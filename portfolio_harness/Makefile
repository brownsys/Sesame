.PHONY: boxed unboxed clean

UNBOXEDPATH = ../portfolio
BOXEDPATH = ../portfolio_boxed

boxed:
# 1. adjust portfolio source in Cargo.toml
	sed -i '' -e 's/\/portfolio\//\/portfolio_boxed\//g' Cargo.toml

# 2. make tmp directory
	mkdir -p target/release

# 3. build the sandboxes manually and move them to our build directory
	cd $(BOXEDPATH)/sandbox && cargo build --release
	cp $(BOXEDPATH)/sandbox/libportfolio_sandbox_sandbox.so target/release/libportfolio_sandbox_sandbox.so

# 4. run harness
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	cargo run --features "boxed" --release


unboxed:
# 1. adjust portfolio source in Cargo.toml
	sed -i '' -e 's/\/portfolio_boxed\//\/portfolio\//g' Cargo.toml

# 2. make tmp directory
	mkdir -p target/release

# 4. run harness
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	cargo run --features "unboxed" --release

clean:
	cargo clean
	cd $(BOXEDPATH) && cargo clean
	cd $(UNBOXEDPATH) && cargo clean