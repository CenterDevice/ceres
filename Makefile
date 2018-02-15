all: check build test

build-lib:
	cargo build

build:
	cargo build

check:
	cargo check

test:
	cargo test

use_case_tests: use_cases
	make -C $<


docs: doctoc
	
doctoc: README.md
	doctoc $<

clippy:
	rustup run nightly cargo clippy --features bin

fmt:
	rustup run nightly cargo fmt

duplicate_libs:
	cargo tree -d

_update-clippy_n_fmt:
	rustup update
	rustup run nightly cargo install clippy --force
	rustup run nightly cargo install rustfmt --force

