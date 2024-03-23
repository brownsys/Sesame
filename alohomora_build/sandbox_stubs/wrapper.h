#ifdef __cplusplus
extern "C" \{
#endif

void invoke_free_c(char*);

{{- for sandbox in sandboxes }}
char* invoke_sandbox_{sandbox}_c(const char* arg);
{{- endfor }}

#ifdef __cplusplus
}
#endif
