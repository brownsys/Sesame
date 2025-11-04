## Sesame build instructions

To build a Sesame-protected application, you need to:
1. Add this repo as a build dependency in Cargo.toml
<!--- Make code --->
[build-dependencies]
sesame_build = { path = "<repo>/sesame/build" }
2. Add/modify the `build.rs` file to construct a Sesame builder and invoke functions required by your application.
<!--- Make code --->
use sesame_build::{Options, SesameBuilder};
fn main() {
    let builder = SesameBuilder::new(Options::new()).unwrap();
    // If the application uses a sandbox lib
    builder.link_sandbox("<path/to/sandbox/lib>");
    // If instead the current crate is the sandbox lib
    builder.build_sandbox();
}
3. For production/release binaries, you should also instruct the SesameBuilder to run scrutinizer and the sesame lints.
We will provide updated instructions on how to do this soon as we are currently improving that API.
4. In the virtual manifest of the workspace, add
<!--- Make code --->
    [workspace.metadata.dylint]
    libraries = [
       { path = "<path/to/sesame/lints>" },
    ]
