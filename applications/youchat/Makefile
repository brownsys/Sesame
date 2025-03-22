.PHONY: youchat check diff clean deep-clean

youchat:
	RUST_BACKTRACE=full cargo run 

check:
	cargo test --no-fail-fast -- --test-threads=1

diff: $(MYDIR)/*
	sh .get_diff.sh

clean:
	rm -rf diff

deep-clean: clean
	cargo clean