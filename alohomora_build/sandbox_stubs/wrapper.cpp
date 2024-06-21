#include "wrapper.h"

// ANCHOR: imports
// We're going to use RLBox in a single-threaded environment.
#define RLBOX_USE_EXCEPTIONS
#define RLBOX_ENABLE_DEBUG_ASSERTIONS
#define RLBOX_SINGLE_THREADED_INVOCATIONS

// All calls into the sandbox are resolved statically.
#define RLBOX_USE_STATIC_CALLS() rlbox_wasm2c_sandbox_lookup_symbol
#define RLBOX_WASM2C_MODULE_NAME {name | double_underscore_formatter}

#include <stdio.h>
#include <cstring>
#include <cassert>
#include <string>
#include <memory>
#include <chrono>
#include <unistd.h>
#include <condition_variable>

#include "{name}.wasm.h"

#include "rlbox.hpp"
#include "rlbox_wasm2c_sandbox.hpp"

/*
 * FFI DEFINITIONS.
 * Each function here corresponds to one function in Rust that calls the sandbox preamble with the actual sandbox function.
 */
#ifdef __cplusplus
extern "C" \{
#endif

{{for sandbox in sandboxes }}
char* {sandbox}_sandbox(const char*, unsigned size);  // Calls the sandbox function (inside sandbox)
// void* alloc_in_sandbox(unsigned size);

void {sandbox}_sandbox_free(char*);  // Frees allocated data (inside sandbox)
{{endfor}}

// function for moving args into sandbox memory

#ifdef __cplusplus
}
#endif

/*
 * End of FFI DEFINITIONS.
 */


using namespace std;
using namespace rlbox;
using namespace std::chrono;

// Define base type for {name}
RLBOX_DEFINE_BASE_TYPES_FOR({name}, wasm2c);

// std::unique_ptr<rlbox_sandbox_{name}> sandbox = nullptr;

struct sandbox_container \{
    rlbox_sandbox_{name} sandbox;
    std::mutex sandbox_mtx;
} typedef sandbox_container_t;

const int NUM_SANDBOXES = 1;
std::vector<sandbox_container_t*> sandbox_pool;

std::mutex pool_mtx;              // used to protect access to modifying the sandbox pool
std::condition_variable pool_cv;  // used to notify threads when a sandbox is available

// Initialize the sandbox containers and create sandboxes within each.
void initialize_sandbox_pool() \{
    for (int i = 0; i < NUM_SANDBOXES; i++) \{
        sandbox_container_t* ptr = new sandbox_container_t;
        sandbox_pool.push_back(ptr);
        sandbox_pool[i]->sandbox.create_sandbox();
    }
}

void* alloc_mem_in_sandbox(unsigned size) \{
    if (sandbox_pool.size() == 0) initialize_sandbox_pool();

    printf("invoking alloc in sandbox\n");
    // unsigned arg = size;
    rlbox_sandbox_{name}* sandbox = &sandbox_pool[0]->sandbox;
    int arg_buf = 10;
    // const char* arg = &arg_buf;
    // unsigned size = sizeof(int);

    auto result_tainted = sandbox->invoke_sandbox_function(alloc_in_sandbox, 10);
    void* result = result_tainted.INTERNAL_unverified_safe();

    // tainted_{name}<char*> tainted_arg = sandbox->malloc_in_sandbox<char>(size);
    // memcpy(tainted_arg.unverified_safe_pointer_because(size, "writing to region"), arg, size);

    printf("done invoking alloc in sandbox, got ptr %p\n", result);
    return (void*)result;
}

{{ for sandbox in sandboxes }}
sandbox_out invoke_sandbox_{sandbox}_c(void* arg, unsigned size) \{
    auto start = high_resolution_clock::now();

    // Lock the sandbox pool for accessing.
    std::unique_lock<std::mutex> pool_lock(pool_mtx);

    // Initialize the pool if it hasn't been.
    if (sandbox_pool.size() == 0) \{
        initialize_sandbox_pool();
    }
    assert(sandbox_pool.size() == NUM_SANDBOXES);

    // Find a free sandbox for the thread to use.
    int slot = -1;
    while (slot == -1) \{
        // Loop through all slots to see if any are available
        for (int i = 0; i < sandbox_pool.size() && slot == -1; i++) \{
            bool found = sandbox_pool[i]->sandbox_mtx.try_lock();
            if (found) slot = i;
        }

        // If none are, wait to be notified that one is done being used
        if (slot == -1) pool_cv.wait(pool_lock);
    }

    // We have a sandbox to use and have locked that, so we can unlock the pool now.
    pool_lock.unlock();

    // Do the actual operations on the sandbox.
    rlbox_sandbox_{name}* sandbox = &sandbox_pool[slot]->sandbox;

        // DONT Copy param into sandbox.
        // tainted_{name}<char*> tainted_arg = sandbox->malloc_in_sandbox<char>(size);
        // memcpy(tainted_arg.unverified_safe_pointer_because(size, "writing to region"), arg, size);

        // INSTEAD call down function in sandbox to allocate space
        // sandbox->alloc_in_sandbox(10);
        // sandbox->invoke_sandbox_function(alloc_in_sandbox, 10);
        // then call up function outside of sandbox to copy into that space
        // transfer_arg();

        // END SETUP TIMER HERE
        auto stop = high_resolution_clock::now();
        auto duration = duration_cast<nanoseconds>(stop - start);
        unsigned long long setup = duration.count();

        // tainted_{name}<void*> tainted_arg;
        char* arg2 = (char*)arg;
        tainted_{name} <char*> tainted_arg;
        tainted_arg.assign_raw_pointer(*sandbox, arg2);

        // tainted_{name}<void*> tainted_arg2 = (tainted_{name}<void*>) tainted_arg;

        // Invoke sandbox.
        tainted_{name}<char*> tainted_result = sandbox->invoke_sandbox_function({sandbox}_sandbox, tainted_arg, size);

        // START TEARDOWN TIMER HERE
        char* buffer = tainted_result.INTERNAL_unverified_safe();
        uint16_t size2 = (((uint16_t)(uint8_t) buffer[0]) * 100) + ((uint16_t)(uint8_t) buffer[1]);
        start = high_resolution_clock::now();

        // Copy output to our memory.
        char* result = (char*) malloc(size2);
        memcpy(result, buffer + 2, size2);

        // Reset sandbox for next use.
        sandbox->free_in_sandbox(tainted_arg); // this call might be redundant but I'm a little spooked to remove it
        sandbox->reset_sandbox();

    // Unlock the sandbox now that it's been reset.
    sandbox_pool[slot]->sandbox_mtx.unlock();
    // Notify a thread that a sandbox slot has opened up.
    pool_cv.notify_one();

    // END TEARDOWN TIMER HERE
    stop = high_resolution_clock::now();
    duration = duration_cast<nanoseconds>(stop - start);
    unsigned long long teardown = duration.count();

    // Return timing data.
    return sandbox_out \{ result, size2, setup, teardown };
}

{{ endfor }}

void invoke_free_c(char* buffer) \{
  free(buffer);
}