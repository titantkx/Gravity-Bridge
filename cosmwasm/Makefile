
# mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
# current_dir := $(notdir $(patsubst %/,%,$(dir $(mkfile_path))))

ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

build:
	cargo build --target wasm32-unknown-unknown --release

build-optimizer:
	docker run --rm -v "$(ROOT_DIR)":/code \
	--mount type=volume,source="$(shell basename "$(ROOT_DIR)")_cache",target=/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/optimizer:0.16.0
	ls -al $(ROOT_DIR)/artifacts

check: build-optimizer
	cosmwasm-check $(ROOT_DIR)/artifacts/cosmwasm.wasm
	# cosmwasm-check $(ROOT_DIR)/target/wasm32-unknown-unknown/release/cosmwasm.wasm

test:
	cargo test --lib -- --nocapture