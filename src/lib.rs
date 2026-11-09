//! アラビア数字 ⇔ 漢数字の相互変換ライブラリ。
//!
//! 設計は [`docs/design.md`](https://github.com/masayasviel/japanese-numeral-normalize/blob/main/docs/design.md) を参照。

// 標準ライブラリ（OS 依存の機能）を使わず、`core` と `alloc`（String / Vec）だけで書く。
// こうしておくと組み込みや WASM でもそのまま使える。
#![no_std]

extern crate alloc;

mod error;
mod format;
// 公開 API から使うまでは「使われていない」という警告が出るので、一時的に抑える。
// allow ではなく expect にしておくと、使い始めて警告が出なくなったときに消し忘れを教えてくれる。
// テストからは使っているので、テスト以外のビルドだけに付ける。
#[cfg_attr(not(test), expect(dead_code, reason = "to_arabic（#15）から使う"))]
mod numeral;
mod units;

use alloc::string::String;

pub use error::{ParseError, ParseErrorKind};
pub use format::KanjiFormat;

use numeral::Numeral;

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
