//! エラー型。

use core::fmt;

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
