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

// Sandbox pool.
#define NUM_SANDBOXES 10

class SandboxPool \{
  public:
    // Create sandboxes on initialization.
    SandboxPool() : pool_(), mtxs_(), mtx_(), cv_() \{
        for (auto &sandbox : this->pool_) \{
          sandbox.create_sandbox();
        }
    }

    // Get sandbox at index (should have acquired lock first).
    rlbox_sandbox_{name}& GetSandbox(size_t index) \{
      return this->pool_[index];
    }

    // Get a lock on some sandbox.
    size_t Lock() \{
        // Lock the sandbox pool for accessing
        std::unique_lock<std::mutex> lock(this->mtx_);

        // Repeat until a sandbox is free.
        while (true) \{
            // Find a free sandbox for the thread to use
            for (size_t i = 0; i < NUM_SANDBOXES; i++) \{
                if (this->mtxs_[i].try_lock()) \{
                  return i;
                }
            }

            // Wait on condition variable.
            this->cv_.wait(lock);
        }
    }

    // Release the sandbox.
    void Unlock(size_t index) \{
        // Reset sandbox for next use.
        this->pool_[index].reset_sandbox();

        // Lock the sandbox pool for accessing
        this->mtxs_[index].unlock();

        // Notify a thread that a sandbox slot has opened up.
        this->cv_.notify_one();
    }

  private:
    // Sandbox pool (already initialized in the constructor).
    std::array<rlbox_sandbox_{name}, NUM_SANDBOXES> pool_;
    // Mutexes for each sandbox in the pool.
    std::array<std::mutex, NUM_SANDBOXES> mtxs_;
    // used to protect access to modifying the sandbox pool.
    std::mutex mtx_;
    // used to notify threads when a sandbox is available.
    std::condition_variable cv_;
};

// The sandbox pool
SandboxPool sandbox_pool;

// Allocates `size` bytes of memory in the sandbox specified by `sandbox_index`.
void* alloc_mem_in_sandbox(size_t size, size_t sandbox_index) \{
    rlbox_sandbox_{name}& sandbox = sandbox_pool.GetSandbox(sandbox_index);

    // Call malloc in sandbox
    tainted_{name}<char*> result_tainted = sandbox.malloc_in_sandbox<char>(size);

    // Swizzle returned pointer -> pointer is in correct 64 bit arch.
    void* result = result_tainted.UNSAFE_unverified();
    return result;
}

// Frees the memory pointed to by `ptr` the sandbox specified by `sandbox_index`.
void free_mem_in_sandbox(void* ptr, size_t sandbox_index) \{
    rlbox_sandbox_{name}& sandbox = sandbox_pool.GetSandbox(sandbox_index);

    // Unswizzle ptr to be inside the sandbox
    tainted_{name} <void*> tainted_ptr;
    tainted_ptr.assign_raw_pointer(sandbox, ptr);

    sandbox.free_in_sandbox(tainted_ptr);
}

// Transform addresses from host machine addresses (usually 64bits) to sandbox
// addresses (32 bits with a different offset) and back.
uint32_t get_sandbox_pointer(void* ptr, size_t sandbox_index) \{
    rlbox_sandbox_{name}& sandbox = sandbox_pool.GetSandbox(sandbox_index);
    return sandbox.get_sandboxed_pointer<void*>(ptr);
}
void* get_unsandboxed_pointer(uint32_t ptr, size_t sandbox_index) \{
    rlbox_sandbox_{name}& sandbox = sandbox_pool.GetSandbox(sandbox_index);
    return sandbox.get_unsandboxed_pointer<void*>(ptr);
}

// Locking and unlocking sandboxes in sandbox pool.
size_t get_lock_on_sandbox() \{
    return sandbox_pool.Lock();
}
void unlock_sandbox(size_t sandbox_index) \{
    sandbox_pool.Unlock(sandbox_index);
}


{{ for sandbox in sandboxes }}
void* invoke_sandbox_{sandbox}_c(void* arg, size_t sandbox_index) \{
    // Get the sandbox
    rlbox_sandbox_{name}& sandbox = sandbox_pool.GetSandbox(sandbox_index);

    // Unswizzle the argument ptr into the sandbox
    char* char_arg = reinterpret_cast<char*>(arg);
    tainted_{name} <char*> tainted_arg;
    tainted_arg.assign_raw_pointer(sandbox, char_arg);

    // Invoke sandbox.
    tainted_{name}<char*> tainted_result = sandbox.invoke_sandbox_function({sandbox}_sandbox, tainted_arg);

    // Swizzle returned result.
    char* buffer = tainted_result.INTERNAL_unverified_safe();

    // Return timing data.
    return reinterpret_cast<void*>(buffer);
}
{{ endfor }}
