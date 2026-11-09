# japanese-numeral-normalize

アラビア数字 ⇔ 漢数字の相互変換ライブラリ

## Requirements

- Docker
- Rust v 1.97

## Run

```sh
# lint / format
make lint
# test
make test
# build
make build

# WASM: wasm/pkg に .js / .d.ts / .wasm を作る
make build-wasm
# WASM のテスト（Rust 側の wasm-bindgen-test と、JavaScript 側の node:test）
make test-wasm
```

WASM 用のイメージ（`compose.yaml` の `wasm` サービス）は、初回の `make build-wasm` / `make test-wasm` / `make lint` で自動的に作られる（wasm-bindgen-cli のビルドに数分かかる）。

## Usage

```rust
use japanese_numeral_normalize::{to_arabic, to_kanji, KanjiFormat};

// 漢数字 → アラビア数字
assert_eq!(to_arabic("二千二十五")?, "2025");
assert_eq!(to_arabic("1億2345万6789")?, "123456789");

// アラビア数字 → 漢数字（書き方は引数で選ぶ）
assert_eq!(to_kanji("12345", KanjiFormat::Standard)?, "一万二千三百四十五");
assert_eq!(to_kanji("10000000", KanjiFormat::ExplicitOneBeforeThousand)?, "一千万");
```

| `KanjiFormat` | 12345 | 10000000 | 0 |
| --- | --- | --- | --- |
| `Standard`（既定） | 一万二千三百四十五 | 千万 | 〇 |
| `ExplicitOneBeforeThousand` | 一万二千三百四十五 | 一千万 | 〇 |
| `Positional` | 一二三四五 | 一〇〇〇〇〇〇〇 | 〇 |
| `Daiji` | 壱万弐千参百四拾五 | 壱千万 | 零 |
| `DaijiFull` | 壱萬弐阡参佰肆拾伍 | 壱阡萬 | 零 |
| `Mixed` | 1万2345 | 1000万 | 0 |

```sh
# サンプル: アラビア数字は全形式の漢数字に、漢数字はアラビア数字にする
docker compose run --rm app cargo run --example convert -- 12345 一億二千万 1.5万
```

## WebAssembly（JavaScript）

`wasm/` に [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) のバインディングがある。
`make build-wasm` で `wasm/pkg` にできる ES モジュール（`--target web`）を読み込んで使う。TypeScript の型定義（`.d.ts`）も付く。

```js
import init, { toArabic, toKanji, KanjiFormat } from "./pkg/japanese_numeral_normalize.js";

await init(); // ブラウザ: .wasm を fetch して初期化する

// 漢数字 → アラビア数字
toArabic("二千二十五");    // "2025"
toArabic("1億2345万6789"); // "123456789"

// アラビア数字 → 漢数字（書き方は省略すると KanjiFormat.Standard）
toKanji("12345");                                           // "一万二千三百四十五"
toKanji("10000000", KanjiFormat.ExplicitOneBeforeThousand); // "一千万"
```

Node.js では、`.wasm` のバイト列を渡して同期的に初期化できる。

```js
import { readFileSync } from "node:fs";
import { initSync, toArabic } from "./pkg/japanese_numeral_normalize.js";

initSync({ module: readFileSync(new URL("./pkg/japanese_numeral_normalize_bg.wasm", import.meta.url)) });
```

数として読めない入力では、`name` が `"ParseError"` の `Error` を投げる。
`kind` は原因（Rust の `ParseErrorKind` のバリアント名）、`position` は位置。
Rust 版の位置はバイト位置だが、JavaScript では文字列と同じ UTF-16 のインデックスになるので、`input.slice(position)` がそのまま使える。

```js
try {
  toArabic("一億万");
} catch (e) {
  e.name;     // "ParseError"
  e.kind;     // "MisplacedUnit"
  e.position; // 2（"一億万".slice(2) === "万"）
}
```
