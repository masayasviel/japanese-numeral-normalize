# テスト・ビルド用の Rust 環境。
#
# ソースはイメージにコピーせず、compose.yaml でマウントして使う。
# （ソースを変えるたびにイメージを作り直さなくてよい）
# 使い方は compose.yaml の先頭を参照。

# compose.yaml から stable と MSRV の 2 つのバージョンを渡す
ARG RUST_VERSION=1.97
FROM rust:${RUST_VERSION}-slim

# 公式イメージは最小構成なので、fmt と clippy を追加する
RUN rustup component add rustfmt clippy

WORKDIR /app
