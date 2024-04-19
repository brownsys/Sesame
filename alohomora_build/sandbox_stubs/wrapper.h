#ifdef __cplusplus
extern "C" \{
#endif

void invoke_free_c(char*);
struct sandbox_out \{ 
    char* result; 
    unsigned long long setup; 
    unsigned long long teardown; 
};

{{- for sandbox in sandboxes }}
sandbox_out invoke_sandbox_{sandbox}_c(const char* arg);
{{- endfor }}

#ifdef __cplusplus
}
#endif
