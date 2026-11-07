//! アラビア数字 ⇔ 漢数字の相互変換ライブラリ。
//!
//! ```
//! use japanese_numeral_normalize::{to_arabic, to_kanji, KanjiFormat, Numeral};
//!
//! // 漢数字（算用数字との混在も可）→ 算用数字
//! assert_eq!(to_arabic("二千二十五").unwrap(), "2025");
//! assert_eq!(to_arabic("二〇二五").unwrap(), "2025");
//! assert_eq!(to_arabic("1億2345万6789").unwrap(), "123456789");
//! assert_eq!(to_arabic("1.5万").unwrap(), "15000");
//!
//! // 算用数字 → 漢数字
//! assert_eq!(to_kanji("12345").unwrap(), "一万二千三百四十五");
//!
//! // Numeral 型を使うと、出力形式の指定や整数型との変換ができる
//! let n: Numeral = "壱萬弐千".parse().unwrap();
//! assert_eq!(n.to_kanji_with(KanjiFormat::Mixed), "1万2000");
//! assert_eq!(i32::try_from(n), Ok(12000));
//! ```
//!
//! 扱える範囲は絶対値が 10^72 未満（無量大数の位まで）の整数。
//! 受け付ける表記・文法の詳細は
//! [`docs/design.md`](https://github.com/masayasviel/japanese-numeral-normalize/blob/main/docs/design.md) を参照。

// 標準ライブラリ（OS 依存の機能）を使わず、`core` と `alloc`（String / Vec）だけで書く。
// こうしておくと組み込みや WASM でもそのまま使える。
#![no_std]

extern crate alloc;

mod error;
mod format;
mod numeral;
mod parse;
mod units;

use alloc::string::{String, ToString};

pub use error::{ParseError, ParseErrorKind, TryFromNumeralError};
pub use format::KanjiFormat;
pub use numeral::Numeral;

/// 漢数字（算用数字との混在も可）を算用数字の文字列にする。
///
/// ```
/// use japanese_numeral_normalize::to_arabic;
///
/// assert_eq!(to_arabic("-三千万").unwrap(), "-30000000");
/// assert_eq!(to_arabic("１，２３４").unwrap(), "1234");
/// assert!(to_arabic("十円").is_err());
/// ```
///
/// # Errors
///
/// 数として読めない場合は [`ParseError`] を返す。
pub fn to_arabic(input: &str) -> Result<String, ParseError> {
    Ok(input.parse::<Numeral>()?.to_string())
}

/// 数（算用数字・漢数字・混在のいずれでも可）を標準の漢数字にする。
///
/// 形式を選びたい場合は [`Numeral::to_kanji_with`] を使う。
///
/// ```
/// use japanese_numeral_normalize::to_kanji;
///
/// assert_eq!(to_kanji("100000001").unwrap(), "一億一");
/// assert_eq!(to_kanji("1億2000万").unwrap(), "一億二千万");
/// ```
///
/// # Errors
///
/// 数として読めない場合は [`ParseError`] を返す。
pub fn to_kanji(input: &str) -> Result<String, ParseError> {
    Ok(input.parse::<Numeral>()?.to_kanji())
}
