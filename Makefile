# テスト・ビルドは Docker（compose.yaml）の中で行う。手元に Rust を入れなくてよい。
# 初回は Rust のイメージを作るので少し時間がかかる。

STABLE := docker compose run --rm stable
MSRV   := docker compose run --rm msrv

.PHONY: lint test build

# 整形（ソースを書き換える）してから clippy で lint する
lint:
	$(STABLE) cargo fmt --all
	$(STABLE) cargo clippy --all-targets -- -D warnings

# stable と MSRV（Rust 1.85）の両方でテストする
test:
	$(STABLE) cargo test
	$(MSRV) cargo test

# 最適化ありでビルドする（成果物は Docker の volume 内の target/release/ に置かれる）
build:
	$(STABLE) cargo build --release
