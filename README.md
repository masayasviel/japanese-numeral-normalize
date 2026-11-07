# japanese-numeral-normalize

アラビア数字 ⇔ 漢数字の相互変換ライブラリ（Rust）。

- 漢数字・算用数字・その混在を読める: `二千二十五` / `二〇二五` / `1億2345万6789` / `1.5万`
- 大字・異体字も読める: `壱萬弐千` / `伍阡捌佰` / `廿一`
- 4 つの形式で書き出せる: `一万二千三百四十五` / `一二三四五` / `壱万弐千参百四拾五` / `1万2345`
- 無量大数の位（10^72 未満）まで扱える
- 依存クレートなし、`no_std`（`alloc` のみ）

## 使い方

```toml
[dependencies]
japanese-numeral-normalize = { git = "https://github.com/masayasviel/japanese-numeral-normalize" }
```

```rust
use japanese_numeral_normalize::{to_arabic, to_kanji, KanjiFormat, Numeral};

// 漢数字 → 算用数字
assert_eq!(to_arabic("二千二十五")?, "2025");
assert_eq!(to_arabic("1億2345万6789")?, "123456789");

// 算用数字 → 漢数字
assert_eq!(to_kanji("12345")?, "一万二千三百四十五");

// Numeral 型: 形式の指定・整数との変換・比較
let n: Numeral = "一億二千万".parse()?;
assert_eq!(n.to_kanji_with(KanjiFormat::Mixed), "1億2000万");
assert_eq!(i64::try_from(n)?, 120_000_000);
assert!(n > Numeral::from(100_000_000));
```

試すだけなら（Docker のみで動きます）:

```sh
docker compose run --rm dev cargo run --example convert -- 一億二千万 1.5万 -42 二〇二五
```

## 出力形式（`KanjiFormat`）

| 形式 | 12345 | 10000000 | 0 |
| --- | --- | --- | --- |
| `Standard`（既定） | 一万二千三百四十五 | 千万 | 〇 |
| `Positional` | 一二三四五 | 一〇〇〇〇〇〇〇 | 〇 |
| `Daiji` | 壱万弐千参百四拾五 | 壱千万 | 零 |
| `Mixed` | 1万2345 | 1000万 | 0 |

## 不正な入力

あいまいな入力は推測せずエラーにします。`ParseError` は原因（`kind()`）と位置（`position()`、バイト単位）を持ちます。

| 入力 | エラー |
| --- | --- |
| `十円` | `InvalidCharacter('円')` |
| `百千`, `一万二億`, `一億万` | `MisplacedUnit` |
| `12百`, `1万20000` | `MisplacedDigit` |
| `1,,2`, `1.5千` | `MisplacedSeparator` |
| `1.5` | `NotAnInteger` |

受け付ける文字・文法の詳細は [docs/design.md](docs/design.md) を参照してください。

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

## 参考

- [geolonia/japanese-numeral](https://github.com/geolonia/japanese-numeral)
- [nagataaaas/Kanjize](https://github.com/nagataaaas/Kanjize)
