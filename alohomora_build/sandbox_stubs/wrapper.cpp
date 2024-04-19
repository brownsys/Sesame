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

// void zero_memory() \{
//   void *heap = sandbox->get_memory_location();
//   uint64_t size = sandbox->get_total_memory();
//   memset(heap, 0, size);
// }

{{ for sandbox in sandboxes }}
sandbox_out invoke_sandbox_{sandbox}_c(const char* arg, unsigned size) \{
  // Declare and create a new sandbox
  // WANT TO TIME CREATION + MEM MALLOC, START TIMER HERE 
  auto start = high_resolution_clock::now();
  rlbox_sandbox_{name} sandbox;
  sandbox.create_sandbox();

  // Copy param into sandbox.
  // size_t size = strlen(arg) + 1;
  tainted_{name}<char*> tainted_arg = sandbox.malloc_in_sandbox<char>(size);
  // strncpy(tainted_arg.unverified_safe_pointer_because(size, "writing to region"), arg, size);
  memcpy(tainted_arg.unverified_safe_pointer_because(size, "writing to region"), arg, size);

  // END TIMER HERE
  auto stop = high_resolution_clock::now();
  auto duration = duration_cast<nanoseconds>(stop - start);
  unsigned long long setup = duration.count();

  // Invoke sandbox.
  tainted_{name}<char *> tainted_result = sandbox.invoke_sandbox_function({sandbox}_sandbox, tainted_arg, size);
  //START TIMER HERE FOR TEARDOWN
  char* buffer = tainted_result.INTERNAL_unverified_safe();
  uint16_t size2 = (((uint16_t)(uint8_t) buffer[0]) * 100) + ((uint16_t)(uint8_t) buffer[1]);
  start = high_resolution_clock::now();
  // Copy output to our memory.
  char* result = (char*) malloc(size2);
  // strncpy(result, buffer, size);
  memcpy(result, buffer + 2, size2);
  // destroy sandbox
  // sandbox.invoke_sandbox_function({sandbox}_sandbox_free, tainted_result);
  sandbox.free_in_sandbox(tainted_arg);
  sandbox.destroy_sandbox();
  // END TIMER HERE FOR TEARDOWN
  stop = high_resolution_clock::now();
  duration = duration_cast<nanoseconds>(stop - start);
  unsigned long long teardown = duration.count();
  return sandbox_out \{ result, size2, setup, teardown };
}

{{ endfor }}

void invoke_free_c(char* buffer) \{
  free(buffer);
}