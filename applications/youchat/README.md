# YouChat
This is a web application built using Rocket as a MySQL backend to test Alohomora. You can run it using `cargo run`.

### Testing
Use `cargo test -- --test-threads=1` to run all tests for the app.
YouChat was intentionally built with a buggy endpoint that will incorrectly leak users' chats from the MySQL database, so the test suite ensures this behavior is stopped by Alohomora while normal site actions are unaffected.
