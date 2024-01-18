.PHONY: build test

build:
	cargo b --bins

client: build
	sudo RUST_LOG=debug ./target/debug/socker -e ./target/debug/client -m 128m

server: build
	sudo RUST_LOG=debug ./target/debug/socker -e ./target/debug/server -m 128m
