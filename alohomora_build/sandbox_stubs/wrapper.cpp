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
#include <sys/syscall.h>

// #ifndef SYS_pthread_self
// #error "SYS_pthread_self unavailable on this system"
// #endif
// #define pthread_self() ((pid_t)syscall(SYS_pthread_self)) // get TID macro for sandbox pool debugging

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
char * {sandbox}_sandbox(const char*, unsigned size);  // Calls the sandbox function (inside sandbox)
void {sandbox}_sandbox_free(char*);                        // Frees allocated data (inside sandbox)
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

// Declare global sandbox.
rlbox_sandbox_{name} sandbox;
bool sandbox_initialized = false;

std::mutex pool_mtx;             // used to protect access to the sandbox list (shouldnt really be needed except for using the cv)
std::condition_variable pool_cv;  // used to notify threads when a sandbox is available

std::atomic_int used_slots = 0;

struct sandbox_container \{
    rlbox_sandbox_{name} sandbox;
    std::mutex sandbox_mtx;
} typedef sandbox_container_t;

std::vector<sandbox_container_t*> sandbox_pool;
const int NUM_SANDBOXES = 2;

// TODO: vector for sandbox pool
// with mutex for each sandbox
// try locking it to see if its used

// initializes the sandbox list the first time we need a sandbox
void initialize_sandbox_pool() \{
    printf("%d: initializing pool\n", pthread_self());
    for (int i = 0; i < NUM_SANDBOXES; i++) \{
        sandbox_container_t* ptr = new sandbox_container_t;
        printf("%d: just pushed back to %d there\n", pthread_self(), i);
        sandbox_pool.push_back(ptr);
        printf("%d: now len is %zu\n", pthread_self(), sandbox_pool.size());
        sandbox_pool[i]->sandbox.create_sandbox();
        printf("%d: just created sandbox %zu\n", pthread_self(), sandbox_pool.size());
    }
}

// TODO: should probably handle this at somepoint
void teardown_sandbox_pool() \{

}

void operate_on_sandbox(rlbox_sandbox_{name}* sandbox) \{

}

{{ for sandbox in sandboxes }}
sandbox_out invoke_sandbox_{sandbox}_c(const char* arg, unsigned size) \{
    auto start = high_resolution_clock::now();

    printf("%d: invoking some shit\n", pthread_self());

    std::unique_lock<std::mutex> pool_lock(pool_mtx);

    printf("%d: got unique lock\n", pthread_self());

    // 0. if the pool isn't initialized, initialize it
    if (sandbox_pool.size() == 0) \{
        initialize_sandbox_pool();
    }
    assert(sandbox_pool.size() == NUM_SANDBOXES);

    
    int slot = -1;
    while (slot == -1) \{
        // 1. loop through all sandboxes to find a slot
        for (int i = 0; i < sandbox_pool.size() && slot == -1; i++) \{
            bool found = sandbox_pool[i]->sandbox_mtx.try_lock();
            printf("%d: checking slot %d\n", pthread_self(), i);
            if (found) slot = i;
        }

        // 2b. if none are free, we need to wait until any are freed up and then try again
        if (slot == -1) \{
            printf("%d: waiting (bc %d slots are used)\n", pthread_self(), used_slots.load());
                pool_cv.wait(
                    pool_lock);
            printf("%d: up an at em again\n", pthread_self());
        }
    }

    ++used_slots;

    printf("%d: found slot %d is free!!! (now %d slots are used)\n", pthread_self(), slot, used_slots.load());

    // we're done accessing the pool, so we can unlock this
    pool_lock.unlock();

    // here we do the actual ops within the sandbox
    printf("%d: operating on slot %d!!!\n", pthread_self(), slot);

    rlbox_sandbox_{name}* sandbox = &sandbox_pool[slot]->sandbox;

        // Copy param into sandbox.
        tainted_{name}<char*> tainted_arg = sandbox->malloc_in_sandbox<char>(size);
        memcpy(tainted_arg.unverified_safe_pointer_because(size, "writing to region"), arg, size);

        // END SETUP TIMER HERE
        auto stop = high_resolution_clock::now();
        auto duration = duration_cast<nanoseconds>(stop - start);
        unsigned long long setup = duration.count();

        // Invoke sandbox.
        tainted_{name}<char *> tainted_result = sandbox->invoke_sandbox_function({sandbox}_sandbox, tainted_arg, size);

        // START TEARDOWN TIMER HERE
        char* buffer = tainted_result.INTERNAL_unverified_safe();
        uint16_t size2 = (((uint16_t)(uint8_t) buffer[0]) * 100) + ((uint16_t)(uint8_t) buffer[1]);
        start = high_resolution_clock::now();

        // Copy output to our memory.
        char* result = (char*) malloc(size2);
        memcpy(result, buffer + 2, size2);

        // Destroy sandbox.
        sandbox->free_in_sandbox(tainted_arg);
        printf("%d: resetting slot %d!!!\n", pthread_self(), slot);
        sandbox->reset_sandbox();

    // unlock the sandbox for others to use
    sandbox_pool[slot]->sandbox_mtx.unlock();

    // notify those waiting with the contion variable
    --used_slots;
    pool_cv.notify_one();

    printf("%d: all done (now %d used slots)\n", pthread_self(), used_slots.load());

  // sandbox.destroy_sandbox();

  // END TEARDOWN TIMER HERE
  stop = high_resolution_clock::now();
  duration = duration_cast<nanoseconds>(stop - start);
  unsigned long long teardown = duration.count();
  return sandbox_out \{ result, size2, setup, teardown };
}

{{ endfor }}

void invoke_free_c(char* buffer) \{
  free(buffer);
}