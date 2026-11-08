# japanese-numeral-normalize

アラビア数字 ⇔ 漢数字の相互変換ライブラリ（Rust）。

> 作り直し中です。設計は [docs/design.md](docs/design.md) を参照してください。

## 開発

テスト・ビルドは Docker（Compose）で行います。手元に Rust を入れる必要はありません。

```sh
docker compose run --rm check                 # fmt / clippy / test をまとめて実行
docker compose run --rm msrv                  # MSRV（Rust 1.85）でテスト
docker compose run --rm dev cargo test        # 好きな cargo コマンドを実行
docker compose run --rm dev cargo build --release
docker compose run --rm dev cargo fmt         # 整形（ソースを書き換える）
docker compose run --rm dev                   # コンテナのシェルに入る
```

- 初回はイメージの作成に少し時間がかかります
- ビルド結果（`target/`）は Docker の volume に置かれます。消すときは `docker compose down -v`
- 手元に Rust がある場合は `cargo test` などを直接実行しても同じです

### エディタ

- VS Code: [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) を入れる
- IntelliJ / RustRover: Rust プラグイン（RustRover は標準）
