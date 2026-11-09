//! 漢数字の書き方（JavaScript に公開する enum）。

// #[wasm_bindgen] が enum 用に生成するコードが f64 を `==` で比べるため、このモジュールだけ許可する
#![allow(clippy::float_cmp)]

use japanese_numeral_normalize as jnn;
use wasm_bindgen::prelude::wasm_bindgen;

// ドキュメントコメントは .d.ts の JSDoc にもなるので、Rust のリンク（[`...`](crate::...)）は書かない
/// 漢数字の書き方。`toKanji` の引数で選ぶ。
///
/// | 形式 | 12345 | 10000000 | 0 |
/// | --- | --- | --- | --- |
/// | `Standard`（既定） | 一万二千三百四十五 | 千万 | 〇 |
/// | `ExplicitOneBeforeThousand` | 一万二千三百四十五 | 一千万 | 〇 |
/// | `Positional` | 一二三四五 | 一〇〇〇〇〇〇〇 | 〇 |
/// | `Daiji` | 壱万弐千参百四拾五 | 壱千万 | 零 |
/// | `DaijiFull` | 壱萬弐阡参佰肆拾伍 | 壱阡萬 | 零 |
/// | `Mixed` | 1万2345 | 1000万 | 0 |
// 本体の `KanjiFormat` は別クレートの型なので #[wasm_bindgen] を付けられない。
// 同じ形の enum を用意して、`From` で本体の型に変換する。
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KanjiFormat {
    /// 単位式（既定）: `一万二千三百四十五`
    #[default]
    Standard,
    /// 単位式で、千の前にも「一」を書く: `一千万`
    ExplicitOneBeforeThousand,
    /// 位取り式: `一二三四五`
    Positional,
    /// 大字: `壱万弐千参百四拾五`
    Daiji,
    /// すべて大字: `壱萬弐阡参佰肆拾伍`
    DaijiFull,
    /// 算用数字と単位の混在: `1万2345`
    Mixed,
}

impl From<KanjiFormat> for jnn::KanjiFormat {
    fn from(format: KanjiFormat) -> Self {
        match format {
            KanjiFormat::Standard => Self::Standard,
            KanjiFormat::ExplicitOneBeforeThousand => Self::ExplicitOneBeforeThousand,
            KanjiFormat::Positional => Self::Positional,
            KanjiFormat::Daiji => Self::Daiji,
            KanjiFormat::DaijiFull => Self::DaijiFull,
            KanjiFormat::Mixed => Self::Mixed,
        }
    }
}
