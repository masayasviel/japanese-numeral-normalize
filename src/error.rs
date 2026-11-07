//! エラー型。

use core::fmt;

/// 文字列を数値として読めなかったときのエラー。
///
/// [`kind`](Self::kind) で原因を、[`position`](Self::position) で
/// 入力文字列中の位置（バイトオフセット）を取得できる。
///
/// ```
/// use japanese_numeral_normalize::{Numeral, ParseErrorKind};
///
/// let err = "一億万".parse::<Numeral>().unwrap_err();
/// assert_eq!(err.kind(), &ParseErrorKind::MisplacedUnit);
/// assert_eq!(err.position(), "一億".len()); // 「万」の位置
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    position: usize,
}

/// [`ParseError`] の原因。
// 後から種類を増やしても利用側の match が壊れないよう #[non_exhaustive] にする
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseErrorKind {
    /// 入力が空、または符号だけで数字がない。
    Empty,
    /// 数字・単位・符号・区切りのいずれでもない文字がある（例: `"十円"` の `円`）。
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
    pub(crate) const fn new(kind: ParseErrorKind, position: usize) -> Self {
        Self { kind, position }
    }

    /// エラーの原因。
    #[must_use]
    pub const fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    /// エラーが見つかった位置。元の入力文字列におけるバイトオフセット。
    ///
    /// 日本語は 1 文字が複数バイト（多くは 3 バイト）なので、文字数ではない点に注意。
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ParseErrorKind::Empty => f.write_str("no digits in input")?,
            ParseErrorKind::InvalidCharacter(c) => write!(f, "invalid character {c:?}")?,
            ParseErrorKind::MisplacedUnit => f.write_str("misplaced unit")?,
            ParseErrorKind::MisplacedDigit => f.write_str("digits do not fit in their place")?,
            ParseErrorKind::MisplacedSeparator => f.write_str("misplaced separator")?,
            ParseErrorKind::NotAnInteger => f.write_str("value is not an integer")?,
            ParseErrorKind::OutOfRange => f.write_str("value is out of range")?,
        }
        write!(f, " at byte {}", self.position)
    }
}

impl core::error::Error for ParseError {}

/// [`Numeral`](crate::Numeral) を整数型に変換できなかった（範囲外だった）ときのエラー。
///
/// ```
/// use japanese_numeral_normalize::Numeral;
///
/// let n = Numeral::from(300);
/// assert!(u8::try_from(n).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// `()` を private なフィールドとして持たせると、クレートの外からは値を作れなくなる。
// （標準ライブラリの `TryFromIntError` と同じ手法）
pub struct TryFromNumeralError(pub(crate) ());

impl fmt::Display for TryFromNumeralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("numeral out of range for the target integer type")
    }
}

// `core::error::Error` は Rust 1.81 から使える。no_std でも `?` で他のエラーと混ぜて扱える。
impl core::error::Error for TryFromNumeralError {}
