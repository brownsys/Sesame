.PHONY: run

run:
	cd ../websubmit_boxed_sandboxes/ && cargo build --release --features "sandbox_timing"
	cargo run --release

plot:
	. venv/bin/activate; python3 plotting.py; deactivate

clean:
	cargo clean
	cd ../websubmit_boxed_sandboxes && cargo clean && rm -rf /libwebsubmit_boxed_sandboxes_sandbox.so && rm -rf wasm_target
