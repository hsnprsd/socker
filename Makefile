.PHONY: build test

build:
	cargo b --bins

test: build
	sudo RUST_LOG=debug ./target/debug/socker ./target/debug/heavy_mem
