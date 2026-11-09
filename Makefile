APP := docker compose run --rm app
WASM := docker compose run --rm wasm

WASM_CRATE := japanese-numeral-normalize-wasm
WASM_TARGET := wasm32-unknown-unknown

.PHONY: lint test build build-wasm test-wasm

lint:
	$(APP) sh -c 'rustup component add rustfmt clippy && cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings'
	$(WASM) cargo clippy -p $(WASM_CRATE) --target $(WASM_TARGET) --all-targets -- -D warnings

test:
	$(APP) cargo test --workspace

build:
	$(APP) cargo build --release

# wasm/pkg に、JavaScript から読み込むファイル一式（.js / .d.ts / _bg.wasm）を作る
build-wasm:
	$(WASM) sh -c 'cargo build -p $(WASM_CRATE) --target $(WASM_TARGET) --release \
		&& wasm-bindgen --target web --out-dir wasm/pkg --out-name japanese_numeral_normalize \
			target/$(WASM_TARGET)/release/japanese_numeral_normalize_wasm.wasm'

# WASM のテスト。Rust 側（wasm-bindgen-test を Node.js で実行）と、JavaScript 側（wasm/pkg を node:test で）
test-wasm: build-wasm
	$(WASM) sh -c 'cargo test -p $(WASM_CRATE) --target $(WASM_TARGET) \
		&& node --test wasm/tests/js/api.test.mjs'
