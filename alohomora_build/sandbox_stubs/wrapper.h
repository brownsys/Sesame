#include <stddef.h>

#ifdef __cplusplus
extern "C" \{
#endif

void invoke_free_c(char*);

struct __attribute__ ((packed)) SizedSliceC \{
  unsigned char *result;
  unsigned int size;
};

struct sandbox_out \{ 
    char* result; 
    unsigned size;
    unsigned long long setup; 
    unsigned long long teardown; 
};

{{- for sandbox in sandboxes }}
struct sandbox_out invoke_sandbox_{sandbox}_c(void* arg, size_t slot);
{{- endfor }}

void* alloc_mem_in_sandbox(unsigned size, size_t sandbox_index);
void free_mem_in_sandbox(void* ptr, size_t sandbox_index);

void unlock_sandbox(size_t sandbox_index);
size_t get_lock_on_sandbox();

#ifdef __cplusplus
}
#endif
