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
#include <stddef.h>
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
// void* alloc_mem_in_sandbox(unsigned size, unsigned sandbox_index);
// void free_mem_in_sandbox(void* ptr, unsigned sandbox_index);

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

const int NUM_SANDBOXES = 10;
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

void* alloc_mem_in_sandbox(unsigned size, size_t sandbox_index) \{
    if (sandbox_pool.size() == 0) initialize_sandbox_pool();

    // printf("invoking alloc in sandbox %d\n", sandbox_index);
    rlbox_sandbox_{name}* sandbox = &sandbox_pool[sandbox_index]->sandbox;

    // auto result_tainted = sandbox->invoke_sandbox_function(alloc_in_sandbox, size);
    tainted_{name}<char*> result_tainted = sandbox->malloc_in_sandbox<char>(size);
    void* result = result_tainted.UNSAFE_unverified();
    // printf("cpp: done invoking alloc in sandbox %d, got ptr %p for sz %d\n", sandbox_index, result, size);

    return result;
}

void free_mem_in_sandbox(void* ptr, size_t sandbox_index) \{
    // TODO: need synchronization for these fns -- dont think i do should be handled at a higher level by instances.
    if (sandbox_pool.size() == 0) initialize_sandbox_pool();

    rlbox_sandbox_{name}* sandbox = &sandbox_pool[sandbox_index]->sandbox;

    // char* ptr_cha/r = (char*)ptr;
    tainted_{name} <void*> tainted_ptr;
    tainted_ptr.assign_raw_pointer(*sandbox, ptr);

    // printf("cpp: trying to invoke free in sandbox %d, for ptr %p\n", sandbox_index, ptr);

    sandbox->free_in_sandbox(tainted_ptr);

    // printf("\tcpp: free complete\n");
}

// Get a lock on a sandbox from the pool for memory allocation & use.
size_t get_lock_on_sandbox() \{
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

    // printf("cpp: got lock on sandbox %d\n", slot);

    return slot;
}

// Unlock a sandbox from the pool.
void unlock_sandbox(size_t i) \{
    // printf("cpp: unlocking sandbox %d\n", i);
    std::unique_lock<std::mutex> pool_lock(pool_mtx);  // This might be unneccessary
    sandbox_pool[i]->sandbox_mtx.unlock();

    // Notify a thread that a sandbox slot has opened up.
    pool_cv.notify_one();
}

{{ for sandbox in sandboxes }}
sandbox_out invoke_sandbox_{sandbox}_c(void* arg, size_t slot) \{
    auto start = high_resolution_clock::now();

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
        tainted_{name}<char*> tainted_result = sandbox->invoke_sandbox_function({sandbox}_sandbox, tainted_arg, 0);
        // tainted_result = sandbox->invoke_sandbox_function({sandbox}_sandbox, tainted_arg, size);

        // START TEARDOWN TIMER HERE
        char* buffer = tainted_result.INTERNAL_unverified_safe();
        uint16_t size2 = (((uint16_t)(uint8_t) buffer[0]) * 100) + ((uint16_t)(uint8_t) buffer[1]);
        start = high_resolution_clock::now();


        // Copy output to our memory.
        char* result = (char*) malloc(size2);
        memcpy(result, buffer + 2, size2);

        // Reset sandbox for next use.
        // sandbox->free_in_sandbox(tainted_arg); // this call might be redundant but I'm a little spooked to remove it
        sandbox->reset_sandbox();
        // sandbox->destroy_sandbox();
        // sandbox->create_sandbox();

        // Unlock the sandbox now that it's been reset.
        // unlock_sandbox(slot);

        // END TEARDOWN TIMER HERE
        stop = high_resolution_clock::now();
        duration = duration_cast<nanoseconds>(stop - start);
        unsigned long long teardown = duration.count();

        // Return timing data.
        return sandbox_out \{result, size2, setup, teardown};
}

{{ endfor }}

void invoke_free_c(char* buffer) \{
  free(buffer);
}