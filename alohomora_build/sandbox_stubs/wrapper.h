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
sandbox_out invoke_sandbox_{sandbox}_c(void* arg, unsigned size);
{{- endfor }}

void* alloc_mem_in_sandbox(unsigned size, unsigned sandbox_index);

void* alloc_in_sandbox(unsigned size);

#ifdef __cplusplus
}
#endif
