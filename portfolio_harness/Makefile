.PHONY: harness

all: harness

harness:
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	cargo build --release && \
	cp ../post-portfolio/sandbox/libportfolio_sandbox_sandbox.so target/release/libportfolio_sandbox_sandbox.so && \
	cargo run --release
