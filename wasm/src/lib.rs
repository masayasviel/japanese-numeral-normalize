//! [`japanese_numeral_normalize`] の WebAssembly バインディング
//!
//! JavaScript からは `toArabic` / `toKanji` / `KanjiFormat` として使う
//!
//! ```js
//! import init, { toArabic, toKanji, KanjiFormat } from "japanese-numeral-normalize-wasm";
//!
//! await init();
//! toArabic("二千二十五");                  // "2025"
//! toKanji("12345");                        // "一万二千三百四十五"
//! toKanji("12345", KanjiFormat.Daiji);     // "壱万弐千参百四拾五"
//! toArabic("十円");                        // Error: 使えない文字「円」があります
//! ```

// #[wasm_bindgen] が enum に生成するコード（JS の数値との比較）で出るため
#![allow(clippy::float_cmp)]

use japanese_numeral_normalize as jnn;
use wasm_bindgen::prelude::*;

/// 漢数字の書き方。`toKanji` の引数で選ぶ
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// 漢数字（算用数字との混在も可）をアラビア数字の文字列にする
///
/// # Errors
///
/// 数として読めない場合は `Error` を投げる（原因はメッセージに入る）
#[wasm_bindgen(js_name = toArabic)]
pub fn to_arabic(input: &str) -> Result<String, JsError> {
    Ok(jnn::to_arabic(input)?)
}

/// アラビア数字の文字列を漢数字にする。書き方は `format` で選ぶ（省略時は `Standard`）
///
/// # Errors
///
/// アラビア数字として読めない場合は `Error` を投げる（原因はメッセージに入る）
#[wasm_bindgen(js_name = toKanji)]
pub fn to_kanji(input: &str, format: Option<KanjiFormat>) -> Result<String, JsError> {
    Ok(jnn::to_kanji(input, format.unwrap_or_default().into())?)
}
