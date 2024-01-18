.PHONY: build test

build:
	cargo b --bins

test: build
	sudo ./target/debug/socker ./target/debug/heavy_mem
