#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" \{
#endif

// FFI for invoking sandbox function; one per sandboxed function.
{{- for sandbox in sandboxes }}
void* invoke_sandbox_{sandbox}_c(void* arg, size_t sandbox_index);
{{- endfor }}

// Allocate/free memory in sandbox.
void* alloc_mem_in_sandbox(size_t size, size_t sandbox_index);
void free_mem_in_sandbox(void* ptr, size_t sandbox_index);

// Transform addresses from host machine addresses (usually 64bits) to sandbox
// addresses (32 bits with a different offset) and back.
uint32_t get_sandbox_pointer(void* ptr, size_t sandbox_index);
void* get_unsandboxed_pointer(uint32_t ptr, size_t sandbox_index);

// Locking and unlocking sandboxes in sandbox pool.
size_t get_lock_on_sandbox();
void unlock_sandbox(size_t sandbox_index);

#ifdef __cplusplus
}
#endif
