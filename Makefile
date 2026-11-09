# テスト・ビルドは Docker（compose.yaml）の中で行う。手元に Rust を入れなくてよい。
# 初回は Rust のイメージをダウンロードするので少し時間がかかる。

APP := docker compose run --rm app

.PHONY: lint test build

# 整形（ソースを書き換える）してから clippy で lint する。
# 公式の Rust イメージは最小構成で rustfmt と clippy が入っていないので、先に追加する。
# コンテナは毎回作り直す（--rm）ため、lint のたびに追加し直す（数 MB のダウンロード）。
lint:
	$(APP) sh -c 'rustup component add rustfmt clippy && cargo fmt --all && cargo clippy --all-targets -- -D warnings'

test:
	$(APP) cargo test

# 最適化ありでビルドする（成果物は Docker の volume 内の target/release/ に置かれる）
build:
	$(APP) cargo build --release
