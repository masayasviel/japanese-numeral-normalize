//! アラビア数字 ⇔ 漢数字の相互変換ライブラリ
//!
//! - [`to_arabic`]: 漢数字（算用数字との混在も可）→ アラビア数字
//! - [`to_kanji`]: アラビア数字 → 漢数字（書き方は [`KanjiFormat`] で選ぶ）
//!
//! ```
//! use japanese_numeral_normalize::{to_arabic, to_kanji, KanjiFormat};
//!
//! // 漢数字（算用数字との混在も可）→ アラビア数字
//! assert_eq!(to_arabic("二千二十五").unwrap(), "2025");
//! assert_eq!(to_arabic("二〇二五").unwrap(), "2025");
//! assert_eq!(to_arabic("1億2345万6789").unwrap(), "123456789");
//! assert_eq!(to_arabic("1.5万").unwrap(), "15000");
//!
//! // アラビア数字 → 漢数字（書き方は引数で選ぶ）
//! assert_eq!(to_kanji("12345", KanjiFormat::Standard).unwrap(), "一万二千三百四十五");
//! assert_eq!(to_kanji("12345", KanjiFormat::Daiji).unwrap(), "壱万弐千参百四拾五");
//! assert_eq!(to_kanji("12345", KanjiFormat::Mixed).unwrap(), "1万2345");
//! ```

#![no_std]

extern crate alloc;

mod error;
mod format;
mod numeral;
mod parse;
mod units;

use alloc::string::String;

pub use error::{ParseError, ParseErrorKind};
pub use format::KanjiFormat;

use numeral::Numeral;

/// 漢数字（算用数字との混在も可）をアラビア数字の文字列にする。
///
/// ```
/// use japanese_numeral_normalize::to_arabic;
///
/// assert_eq!(to_arabic("二千二十五").unwrap(), "2025");
/// assert_eq!(to_arabic("二〇二五").unwrap(), "2025");
/// assert_eq!(to_arabic("1億2345万6789").unwrap(), "123456789");
/// assert_eq!(to_arabic("1,234万").unwrap(), "12340000");
/// assert_eq!(to_arabic("1.5万").unwrap(), "15000");
/// assert_eq!(to_arabic("-三千万").unwrap(), "-30000000");
/// assert_eq!(to_arabic("壱萬弐千").unwrap(), "12000");
/// assert!(to_arabic("十円").is_err());
/// ```
///
/// # Errors
///
/// 数として読めない場合は [`ParseError`] を返す。
pub fn to_arabic(input: &str) -> Result<String, ParseError> {
    Ok(parse::parse(input)?.to_arabic())
}

/// アラビア数字の文字列を漢数字にする。書き方は `format` で選ぶ。
///
/// ```
/// use japanese_numeral_normalize::{to_kanji, KanjiFormat};
///
/// assert_eq!(to_kanji("12345", KanjiFormat::Standard).unwrap(), "一万二千三百四十五");
/// assert_eq!(to_kanji("10000000", KanjiFormat::ExplicitOneBeforeThousand).unwrap(), "一千万");
/// assert_eq!(to_kanji("-1,000", KanjiFormat::Daiji).unwrap(), "-壱千");
/// assert_eq!(to_kanji("２０２５", KanjiFormat::Positional).unwrap(), "二〇二五");
/// // 漢数字は受け付けない
/// assert!(to_kanji("一万", KanjiFormat::Standard).is_err());
/// ```
///
/// # Errors
///
/// アラビア数字として読めない場合は [`ParseError`] を返す。
pub fn to_kanji(input: &str, format: KanjiFormat) -> Result<String, ParseError> {
    Ok(Numeral::parse_arabic(input)?.to_kanji(format))
}
