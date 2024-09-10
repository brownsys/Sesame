.PHONY: all boxed unboxed flamegraph flamegraph-boxed flamegraph-unboxed plot clean sudo-clean

all: boxed unboxed

boxed:
	cargo run --release --features "boxed" -- -i alohomora -c sample-config.toml

unboxed:
	cargo run --release --features "unboxed" -- -i alohomora -c sample-config.toml

plot:
	. venv/bin/activate; python3 plotting.py

flamegraph: flamegraph-boxed flamegraph-unboxed

flamegraph-boxed: export CARGO_PROFILE_RELEASE_DEBUG := true
flamegraph-boxed:
	cargo flamegraph -o flamegraph.boxed.svg --root --release --features "boxed" -- -i alohomora

flamegraph-unboxed: export CARGO_PROFILE_RELEASE_DEBUG := true
flamegraph-unboxed:
	cargo flamegraph -o flamegraph.unboxed.svg --root --release --features "unboxed" -- -i alohomora

clean:
	cargo clean; rm -rf benches/ flamegraph.boxed.svg flamegraph.unboxed.svg

sudo-clean:
	sudo cargo clean; sudo rm -rf benches/ flamegraph.boxed.svg flamegraph.unboxed.svg
