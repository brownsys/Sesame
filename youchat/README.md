# YouChat
This is a web application built using Rocket as a MySQL backend to test Alohomora. You can run it using `cargo run` or `make youchat`.

### Testing
Use `cargo test -- --test-threads=1` (or `make check`) to run all tests for the app. YouChat was intentionally built with a buggy endpoint that will incorrectly leak users' chats from the MySQL database, so the test suite ensures this behavior is stopped by Alohomora while normal site actions are unaffected.

### Alohomora Changes
The `pre-alohomora` branch contains the state of the repo before switching to use Alohomora. Run `sh .get_diff.sh` (or `make diff`) to record the changes to each source file as a .diff in the `diff` directory.