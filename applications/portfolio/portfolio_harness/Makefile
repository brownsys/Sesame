.PHONY: boxed unboxed clean

SHELL=bash

UNBOXEDPATH = ../portfolio
BOXEDPATH = ../portfolio_boxed

boxed:
	# build portfolio sandbox separately to produce .so file
	cd $(BOXEDPATH)/sandbox && cargo build --release
	mkdir -p target/release && cp $(BOXEDPATH)/sandbox/libportfolio_sandbox_sandbox.so target/release/libportfolio_sandbox_sandbox.so
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	cargo run --features "boxed" --release

unboxed:
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	cargo run --features "unboxed" --release

clean:
	cargo clean
	cd $(BOXEDPATH)/sandbox/ && rm -rf libportfolio_sandbox_sandbox.so && cargo clean && rm -rf wasm_target
	number=1 ; while [[ $$number -le 1001 ]] ; do \
        rm -rf $$number ; \
        ((number = number + 1)) ; \
    done
