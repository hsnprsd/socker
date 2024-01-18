.PHONY: build test

build:
	cargo b --bins

test: build
	sudo RUST_LOG=debug ./target/debug/socker -e ./target/debug/heavy_mem -m 128m
