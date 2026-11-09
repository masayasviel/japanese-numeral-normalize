//! [`japanese_numeral_normalize`] の WebAssembly（JavaScript）バインディング。
//!
//! [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) で、次の 3 つを JavaScript に公開する。
//!
//! - [`toArabic`](to_arabic): 漢数字（算用数字との混在も可）→ アラビア数字
//! - [`toKanji`](to_kanji): アラビア数字 → 漢数字（書き方は [`KanjiFormat`] で選ぶ。省略すると `Standard`）
//! - [`KanjiFormat`]: 漢数字の書き方。`toKanji` の 2 つ目の引数
//!
//! 数として読めない入力では、`name` が `"ParseError"` の `Error` を投げる。
//! `kind` に原因、`position` に入力文字列中の位置（UTF-16 のインデックス）を持つ。
//!
//! ```js
//! import init, { toArabic, toKanji, KanjiFormat } from "./pkg/japanese_numeral_normalize.js";
//!
//! await init();
//! toArabic("1億2345万6789");              // "123456789"
//! toKanji("12345");                      // "一万二千三百四十五"
//! toKanji("12345", KanjiFormat.Daiji);   // "壱万弐千参百四拾五"
//!
//! try {
//!   toArabic("十円");
//! } catch (e) {
//!   e.name;     // "ParseError"
//!   e.kind;     // "InvalidCharacter"
//!   e.position; // 1（"十円".slice(1) === "円"）
//! }
//! ```

mod error;
mod format;

use japanese_numeral_normalize as jnn;
use js_sys::Error;
use wasm_bindgen::prelude::wasm_bindgen;

pub use format::KanjiFormat;

/// 漢数字（算用数字との混在も可）をアラビア数字の文字列にする。
///
/// ```js
/// toArabic("二千二十五");     // "2025"
/// toArabic("二〇二五");       // "2025"
/// toArabic("1億2345万6789"); // "123456789"
/// toArabic("1.5万");         // "15000"
/// ```
///
/// # Errors
///
/// 数として読めない場合は `ParseError`（`name` が `"ParseError"` の `Error`）を投げる。
#[wasm_bindgen(js_name = toArabic)]
pub fn to_arabic(input: &str) -> Result<String, Error> {
    jnn::to_arabic(input).map_err(|err| error::to_js(input, &err))
}

/// アラビア数字の文字列を漢数字にする。書き方は `format` で選ぶ（省略すると `KanjiFormat.Standard`）。
///
/// ```js
/// toKanji("12345");                                           // "一万二千三百四十五"
/// toKanji("10000000", KanjiFormat.ExplicitOneBeforeThousand); // "一千万"
/// toKanji("-1,000", KanjiFormat.Daiji);                       // "-壱千"
/// ```
///
/// # Errors
///
/// アラビア数字として読めない場合（漢数字を含む場合も）は `ParseError`（`name` が `"ParseError"` の `Error`）を投げる。
#[wasm_bindgen(js_name = toKanji)]
pub fn to_kanji(input: &str, format: Option<KanjiFormat>) -> Result<String, Error> {
    jnn::to_kanji(input, format.unwrap_or_default().into()).map_err(|err| error::to_js(input, &err))
}
