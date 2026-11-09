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
```

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
