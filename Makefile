all: check build test tests docs

todos:
	rg --vimgrep -g '!Makefile' -i todo 

check:
	cargo check

build:
	cargo build

test:
	cargo test

tests:
	cd $@ && $(MAKE)

clean-package:
	cargo clean -p $$(cargo read-manifest | jq -r .name)

use_case_tests: use_cases
	make -C $<

docs: man
	
man:
	$(MAKE) -C docs

release: clean-package release-test release-bump all
	git commit -am "Bump to version $$(cargo read-manifest | jq .version)"
	git tag v$$(cargo read-manifest | jq -r .version)

release-test: check test clippy
	cargo audit
	cargo +nightly fmt -- --check
	cargo publish --dry-run

release-bump:
	cargo bump

publish:
	git push && git push --tags

install:
	cargo install --force

clippy:
	cargo clippy --all --all-targets -- -D warnings $$(source ".clippy.args")

fmt:
	cargo +nightly fmt

duplicate_libs:
	cargo tree -d

_update-clippy_n_fmt:
	rustup update
	rustup component add clippy
	rustup component add rustfmt --toolchain=nightly

_cargo_install:
	cargo install -f cargo-tree
	cargo install -f cargo-bump

.PHONY: tests

