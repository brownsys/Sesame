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
char* {sandbox}_sandbox(const char*);  // Calls the sandbox function (inside sandbox)
void {sandbox}_sandbox_free(char*);  // Frees allocated data (inside sandbox)
{{endfor}}

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

// Initializes the sandbox containers and create sandboxes within each.
void initialize_sandbox_pool() \{
    for (int i = 0; i < NUM_SANDBOXES; i++) \{
        sandbox_container_t* ptr = new sandbox_container_t;
        sandbox_pool.push_back(ptr);
        sandbox_pool[i]->sandbox.create_sandbox();
    }
}

// Allocates `size` bytes of memory in the sandbox specified by `sandbox_index`.
void* alloc_mem_in_sandbox(unsigned size, size_t sandbox_index) \{
    if (sandbox_pool.size() == 0) initialize_sandbox_pool();
    rlbox_sandbox_{name}* sandbox = &sandbox_pool[sandbox_index]->sandbox;

    // Call malloc in sandbox
    tainted_{name}<char*> result_tainted = sandbox->malloc_in_sandbox<char>(size);

    // Swizzle returned pointer
    void* result = result_tainted.UNSAFE_unverified();
    return result;
}

// Frees the memory pointed to by `ptr` the sandbox specified by `sandbox_index`.
void free_mem_in_sandbox(void* ptr, size_t sandbox_index) \{
    if (sandbox_pool.size() == 0) initialize_sandbox_pool();
    rlbox_sandbox_{name}* sandbox = &sandbox_pool[sandbox_index]->sandbox;

    // Unswizzle ptr to be inside the sandbox
    tainted_{name} <void*> tainted_ptr;
    tainted_ptr.assign_raw_pointer(*sandbox, ptr);

    sandbox->free_in_sandbox(tainted_ptr);
}

// Get a lock on a sandbox from the pool for memory allocation & use.
size_t get_lock_on_sandbox() \{
    // Lock the sandbox pool for accessing
    std::unique_lock<std::mutex> pool_lock(pool_mtx);

    // Initialize the pool if it hasn't been
    if (sandbox_pool.size() == 0) \{
        initialize_sandbox_pool();
    }
    assert(sandbox_pool.size() == NUM_SANDBOXES);

    // Find a free sandbox for the thread to use
    int slot = -1;
    while (slot == -1) \{
        // Loop through all slots in the pool to see if any are available
        for (int i = 0; i < sandbox_pool.size() && slot == -1; i++) \{
            bool found = sandbox_pool[i]->sandbox_mtx.try_lock();
            if (found) slot = i;
        }

        // If none are, wait to be notified that one is done being used
        if (slot == -1) pool_cv.wait(pool_lock);
    }

    return slot;
}

// Unlock a specific sandbox from the pool after use.
void unlock_sandbox(size_t sandbox_index) \{
    // Reset sandbox for next use.
    sandbox_pool[sandbox_index]->sandbox.reset_sandbox();

    std::unique_lock<std::mutex> pool_lock(pool_mtx);  // This might be unneccessary
    // Unlock it, so other threads can access.
    sandbox_pool[sandbox_index]->sandbox_mtx.unlock();

    // Notify a thread that a sandbox slot has opened up.
    pool_cv.notify_one();
}

{{ for sandbox in sandboxes }}
char* invoke_sandbox_{sandbox}_c(void* arg, size_t slot) \{
    printf("in c wrapper invoke()\n");
    fflush(stdout);
    // auto start = high_resolution_clock::now();

    // Get the sandbox
    rlbox_sandbox_{name}* sandbox = &sandbox_pool[slot]->sandbox;

    // END SETUP TIMER HERE
    // auto stop = high_resolution_clock::now();
    // auto duration = duration_cast<nanoseconds>(stop - start);
    // unsigned long long setup = duration.count();

    // Swizzle arg ptr into the sandbox
    char* arg2 = (char*)arg;             // `assign_raw_pointer` only works with char*s so we have to cast it first
    tainted_{name} <char*> tainted_arg;
    tainted_arg.assign_raw_pointer(*sandbox, arg2);

    // Invoke sandbox.
    printf("in c wrapper gonna call preamble\n");
    tainted_{name}<char*> tainted_result = sandbox->invoke_sandbox_function({sandbox}_sandbox, tainted_arg);
    printf("in c wrapper done calling preamble\n");
    // START TEARDOWN TIMER HERE
    char* buffer = tainted_result.INTERNAL_unverified_safe();
    // uint16_t size2 = (((uint16_t)(uint8_t) buffer[0]) * 100) + ((uint16_t)(uint8_t) buffer[1]);
    // start = high_resolution_clock::now();

    // Copy output to our memory.
    // char* result = (char*) malloc(size2);
    // memcpy(result, buffer + 2, size2);

    // END TEARDOWN TIMER HERE
    // stop = high_resolution_clock::now();
    // duration = duration_cast<nanoseconds>(stop - start);
    // unsigned long long teardown = duration.count();

    // Return timing data.
    return buffer;
}

{{ endfor }}

void invoke_free_c(char* buffer) \{
  free(buffer);
}