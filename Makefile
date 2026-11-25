WASM_TARGET := wasm32v1-none
WASM_DIR := target/$(WASM_TARGET)/release

.PHONY: all build test fmt fmt-check clippy wasm check clean

all: check

build:
	cargo build --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

# Release Wasm for every contract, ready for `stellar contract deploy`.
wasm:
	cargo build --workspace --target $(WASM_TARGET) --release
	@ls -l $(WASM_DIR)/*.wasm

# What CI runs.
check: fmt-check clippy test wasm

clean:
	cargo clean
