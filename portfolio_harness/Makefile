.PHONY: harness

all: harness

harness:
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	mkdir -p target/release && \
	cd ../Portfolio/sandbox && cargo build --release && cd - \
	cp ../Portfolio/sandbox/libportfolio_sandbox_sandbox.so target/release/libportfolio_sandbox_sandbox.so && \
	cargo build --release && \
	cargo run --release

boxed:
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	mkdir -p target/release && \
	cd ../Portfolio/sandbox && cargo build "boxed" --release && cd - \
	cp ../Portfolio/sandbox/libportfolio_sandbox_sandbox.so target/release/libportfolio_sandbox_sandbox.so && \
	cargo build --features "boxed" --release && \
	cargo run --features "boxed" --release


unboxed:
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	mkdir -p target/release && \
	cd ../Portfolio/sandbox && cargo build --release && cd - \
	cp ../Portfolio/sandbox/libportfolio_sandbox_sandbox.so target/release/libportfolio_sandbox_sandbox.so && \
	cargo build --features "unboxed" --release && \
	cargo run --features "unboxed" --release
