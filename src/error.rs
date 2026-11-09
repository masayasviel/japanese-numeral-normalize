//! エラー型。

use core::fmt;

/// 入力の文字列を数として読めなかったときのエラー。
///
/// [`kind`](Self::kind) で原因を取得できる。
///
/// ```
/// use japanese_numeral_normalize::{to_arabic, ParseErrorKind};
///
/// let err = to_arabic("一億万").unwrap_err();
/// assert_eq!(err.kind(), &ParseErrorKind::MisplacedUnit);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
}

/// [`ParseError`] の原因。
// 後から種類を増やしても利用側の match が壊れないよう #[non_exhaustive] にする
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseErrorKind {
    /// 入力が空、または符号だけで数字がない。
    Empty,
    /// 受け付けない文字がある（例: `"十円"` の `円`）。
    InvalidCharacter(char),
    /// 単位の順序や位置が正しくない（例: `"百千"`, `"一万二億"`, `"一億万"`）。
    MisplacedUnit,
    /// 数字列がその位に収まらない（例: `"12百"`, `"十25"`, `"1万20000"`）。
    MisplacedDigit,
    /// 桁区切り `,` や小数点 `.` の位置が正しくない（例: `",1"`, `"1,,2"`, `"1.5千"`）。
    MisplacedSeparator,
    /// 小数を含み、整数にならない（例: `"1.5"`, `"1.23456万"`）。
    NotAnInteger,
    /// 表せる範囲（絶対値が 10^72 未満、つまり無量大数の位まで）を超えている。
    OutOfRange,
}

impl ParseError {
    pub(crate) const fn new(kind: ParseErrorKind) -> Self {
        Self { kind }
    }

    /// エラーの原因。
    #[must_use]
    pub const fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::Empty => f.write_str("数字がありません"),
            // 改行などの制御文字も見えるよう escape_debug で `\n` のように書く
            ParseErrorKind::InvalidCharacter(c) => {
                write!(f, "使えない文字「{}」があります", c.escape_debug())
            }
            ParseErrorKind::MisplacedUnit => f.write_str("単位の位置が正しくありません"),
            ParseErrorKind::MisplacedDigit => f.write_str("数字がその位に収まりません"),
            ParseErrorKind::MisplacedSeparator => {
                f.write_str("桁区切りまたは小数点の位置が正しくありません")
            }
            ParseErrorKind::NotAnInteger => f.write_str("整数になりません"),
            ParseErrorKind::OutOfRange => f.write_str("表せる範囲を超えています"),
        }
    }
}

impl core::error::Error for ParseError {}
