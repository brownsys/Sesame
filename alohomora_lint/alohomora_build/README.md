## Alohomora build instructions

To build a release version of an Alohomora-protected application: 

1. Add a build.rs file in each crate to check with a call to `alohomora_build::privacy_check_build()` in the main() function. 
2. Then in the Cargo.toml of each of these crates, add
<!--- Make code --->
    [build-dependencies]
    alohomora_build = { path = "../alohomora_lint/alohomora_build" } 

3. Then in the virtual manifest of the workspace, add 
<!--- Make code --->
    [workspace.metadata.dylint]
    libraries = [  
       { path = "./alohomora_lint/alohomora_type_derived" },
    ]

The name of the linting library file must follow the dylint specification (form of `DLL_PREFIX LIBRARY_NAME '@' TOOLCHAIN DLL_SUFFIX`)
even though the lib name specified in the workspace is just the plain library name. 

TODO: The relative paths should be swapped for packages or git repos