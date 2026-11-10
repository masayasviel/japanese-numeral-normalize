APP := docker compose run --rm app
WASM_PACK_VERSION := 0.15.0

.PHONY: lint test build wasm

lint:
	$(APP) sh -c 'rustup component add rustfmt clippy && cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings'

test:
	$(APP) cargo test --workspace

build:
	$(APP) cargo build --release

# wasm/pkg に npm パッケージを作る。wasm-pack は target ボリュームに入れて使い回す
wasm:
	$(APP) sh -c 'rustup target add wasm32-unknown-unknown && cargo install --locked --root target/tools wasm-pack@$(WASM_PACK_VERSION) && target/tools/bin/wasm-pack build wasm --target web'
