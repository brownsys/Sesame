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
char * {sandbox}_sandbox(const char*);  // Calls the sandbox function (inside sandbox)
void {sandbox}_sandbox_free(char*);     // Frees allocated data (inside sandbox)
{{endfor}}

#ifdef __cplusplus
}
#endif

/*
 * End of FFI DEFINITIONS.
 */


using namespace std;
using namespace rlbox;

// Define base type for {name}
RLBOX_DEFINE_BASE_TYPES_FOR({name}, wasm2c);

{{ for sandbox in sandboxes }}
char* invoke_sandbox_{sandbox}_c(const char* arg) \{
  // Declare and create a new sandbox
  rlbox_sandbox_{name} sandbox;
  sandbox.create_sandbox();

  // Copy param into sandbox.
  size_t size = strlen(arg) + 1;
  tainted_myapp_lib<char*> tainted_arg = sandbox.malloc_in_sandbox<char>(size);
  strncpy(tainted_arg.unverified_safe_pointer_because(size, "writing to region"), arg, size);

  // Invoke sandbox.
  tainted_myapp_lib<char*> tainted_result = sandbox.invoke_sandbox_function({sandbox}_sandbox, tainted_arg);
  char* buffer = tainted_result.INTERNAL_unverified_safe();

  // Copy output to our memory.
  size = strlen(buffer) + 1;
  char* result = (char*) malloc(size);
  strncpy(result, buffer, size);

  // destroy sandbox
  sandbox.invoke_sandbox_function({sandbox}_sandbox_free, tainted_result);
  sandbox.free_in_sandbox(tainted_arg);
  sandbox.destroy_sandbox();

  return result;
}

{{ endfor }}

void invoke_free_c(char* buffer) \{
  free(buffer);
}