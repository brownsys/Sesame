.PHONY: prime
prime:
	mariadb -u root -ppassword < data/prime.sql

.PHONY: websubmit
websubmit:
	mkdir -p /tmp/websubmit/css
	mkdir -p /tmp/websubmit/js
	ROCKET_TEMPLATE_DIR="websubmit/templates" RUST_BACKTRACE=full cargo run --quiet --bin websubmit -- -i alohomora -c websubmit/sample-config.toml

.PHONY: websubmit-boxed
websubmit-boxed:
	mkdir -p /tmp/websubmit-boxed/css
	mkdir -p /tmp/websubmit-boxed/js
	ROCKET_TEMPLATE_DIR="websubmit-boxed/templates" RUST_BACKTRACE=full cargo run --quiet --bin websubmit-boxed -- -i alohomora -c websubmit-boxed/sample-config.toml
