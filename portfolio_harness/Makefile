.PHONY: harness

all: harness

harness:
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	mkdir -p target/release && \
	cd ../Portfolio/sandbox && cargo build --release && cd - \
	cp ../Portfolio/sandbox/libportfolio_sandbox_sandbox.so target/release/libportfolio_sandbox_sandbox.so && \
	cargo build --release && \
	cargo run --release
