APP := docker compose run --rm app

.PHONY: lint test build

lint:
	$(APP) sh -c 'rustup component add rustfmt clippy && cargo fmt --all && cargo clippy --all-targets -- -D warnings'

test:
	$(APP) cargo test

build:
	$(APP) cargo build --release
