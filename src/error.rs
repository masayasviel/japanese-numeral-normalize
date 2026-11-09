//! エラー型。

use core::fmt;

/// 入力の文字列を数として読めなかったときのエラー。
///
/// [`kind`](Self::kind) で原因を、[`position`](Self::position) で
/// 入力文字列中の位置（バイトオフセット）を取得できる。
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
    /// 受け付けない文字がある（例: `"12a"` の `a`）。
    InvalidCharacter(char),
    /// 桁区切り `,` の位置が正しくない（例: `",1"`, `"1,,2"`, `"1,"`）。
    MisplacedSeparator,
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
            ParseErrorKind::MisplacedSeparator => f.write_str("misplaced separator")?,
            ParseErrorKind::OutOfRange => f.write_str("value is out of range")?,
        }
        write!(f, " at byte {}", self.position)
    }
}

// `core::error::Error` は Rust 1.81 から使える。no_std でも `?` で他のエラーと混ぜて扱える。
impl core::error::Error for ParseError {}
