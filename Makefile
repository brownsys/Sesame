.PHONY: prime
prime:
	mariadb -u root -ppassword < data/prime.sql

.PHONY: websubmit
websubmit:
	mkdir -p /tmp/websubmit/css
	mkdir -p /tmp/websubmit/js
	RUST_BACKTRACE=full cargo run --quiet --bin websubmit -- -i alohomora -c websubmit/sample-config.toml

.PHONY: websubmit-boxed
websubmit-boxed:
	mkdir -p /tmp/websubmit-boxed/css
	mkdir -p /tmp/websubmit-boxed/js
	RUST_BACKTRACE=full cargo run --quiet --bin websubmit_boxed -- -i alohomora -c websubmit_boxed/sample-config.toml
