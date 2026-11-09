# japanese-numeral-normalize

アラビア数字 ⇔ 漢数字の相互変換ライブラリ（Rust）。

> 作り直し中です。設計は [docs/design.md](docs/design.md) を参照してください。

## 開発

テスト・ビルドは Docker（Compose）の中で行います。Docker と make があれば、手元に Rust を入れる必要はありません。

```sh
make lint    # 整形（ソースを書き換える）してから clippy で lint
make test    # テスト
make build   # 最適化ありでビルド（cargo build --release）
```

- 公式の Rust イメージ（`rust:1.97-slim`）をそのまま使います。初回はイメージのダウンロードに少し時間がかかります
- 公式イメージには rustfmt と clippy が入っていないため、`make lint` は実行のたびにそれらを追加します（数 MB のダウンロード）
- ビルド結果（`target/`）は Docker の volume に置かれます。消すときは `docker compose down -v`
- 手元に Rust がある場合は `cargo test` などを直接実行しても同じです

### エディタ

- VS Code: [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) を入れる
- IntelliJ / RustRover: Rust プラグイン（RustRover は標準）
