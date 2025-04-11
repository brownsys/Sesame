.PHONY: boxed unboxed k9db flamegraph flamegraph-boxed flamegraph-unboxed plot clean sudo-clean

boxed:
	mkdir -p /tmp/harness/css
	mkdir -p /tmp/harness/js
	cd ../websubmit_boxed_sandboxes/ && cargo build --release
	mkdir -p target/release && cp ../websubmit_boxed_sandboxes/libwebsubmit_boxed_sandboxes_sandbox.so target/release/libwebsubmit_boxed_sandboxes_sandbox.so
	RUST_BACKTRACE=1 cargo run --release --features "boxed" -- -i alohomora -c sample-config.toml

k9db:
	mkdir -p /tmp/harness/css
	mkdir -p /tmp/harness/js
	cd ../websubmit_boxed_sandboxes/ && cargo build --release
	mkdir -p target/release && cp ../websubmit_boxed_sandboxes/libwebsubmit_boxed_sandboxes_sandbox.so target/release/libwebsubmit_boxed_sandboxes_sandbox.so
	RUST_BACKTRACE=1 cargo run --release --features "k9db" -- -i alohomora -c sample-config.toml

unboxed:
	mkdir -p /tmp/harness/css
	mkdir -p /tmp/harness/js
	RUST_BACKTRACE=1 cargo run --release --features "unboxed" -- -i websubmit -c sample-config.toml

plot:
	. venv/bin/activate; python3 plotting.py; deactivate

flamegraph: flamegraph-boxed flamegraph-unboxed

flamegraph-boxed: export CARGO_PROFILE_RELEASE_DEBUG := true
flamegraph-boxed:
	cargo flamegraph -o flamegraph.boxed.svg --root --release --features "boxed" -- -i alohomora

flamegraph-unboxed: export CARGO_PROFILE_RELEASE_DEBUG := true
flamegraph-unboxed:
	cargo flamegraph -o flamegraph.unboxed.svg --root --release --features "unboxed" -- -i alohomora

clean:
	cargo clean; rm -rf benches/ flamegraph.boxed.svg flamegraph.unboxed.svg
	cd ../websubmit_boxed_sandboxes && cargo clean && rm -rf libwebsubmit_boxed_sandboxes_sandbox.so && rm -rf wasm_target

sudo-clean:
	sudo cargo clean; sudo rm -rf benches/ flamegraph.boxed.svg flamegraph.unboxed.svg
	cd ../websubmit_boxed_sandboxes && sudo cargo clean && sudo rm -rf libwebsubmit_boxed_sandboxes_sandbox.so && sudo rm -rf wasm_target
