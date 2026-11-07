//! [`Numeral`] 型: 漢数字で表せる範囲の整数。

use alloc::string::String;
use core::cmp::Ordering;
use core::fmt::{self, Write as _};
use core::ops::Neg;

use crate::error::TryFromNumeralError;

/// 節（4 桁の組）の数。`groups[k]` が 10^(4k) の位を表す。
///
/// k = 0..=17 なので、最上位は無量大数（10^68）の節。表せる絶対値は 10^72 未満。
pub(crate) const GROUPS: usize = 18;

/// 1 つの節の基数。漢数字は万進法なので 10^4。
pub(crate) const BASE: u16 = 10_000;

/// 漢数字で表せる範囲（絶対値が 10^72 未満）の符号付き整数。
///
/// `u128` の最大値は約 3.4×10^38 で無量大数（10^68）に届かないため、
/// 万進法の 4 桁ごとの組（節）を配列で持つ。
///
/// ```
/// use japanese_numeral_normalize::Numeral;
///
/// let n = Numeral::from(-1_2345_6789_i64);
/// assert_eq!(n.to_string(), "-123456789");
/// assert!(n.is_negative());
/// assert_eq!(i64::try_from(n), Ok(-1_2345_6789));
/// ```
// 0 のときは必ず `negative == false` にする（-0 を作らない）ので、表現は値ごとに一意。
// そのため `PartialEq` / `Eq` / `Hash` は derive したもので正しく動く。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Numeral {
    negative: bool,
    /// 節ごとの値（各 0..=9999）。下位の節から並ぶ（リトルエンディアン）。
    groups: [u16; GROUPS],
}

impl Numeral {
    /// 0。
    pub const ZERO: Self = Self {
        negative: false,
        groups: [0; GROUPS],
    };

    /// 符号と節から作る。0 なら符号を正にそろえる（正規化）。
    pub(crate) fn from_groups(negative: bool, groups: [u16; GROUPS]) -> Self {
        debug_assert!(
            groups.iter().all(|&g| g < BASE),
            "each group must be < 10000"
        );
        let is_zero = groups.iter().all(|&g| g == 0);
        Self {
            negative: negative && !is_zero,
            groups,
        }
    }

    /// 節の配列（下位から）。
    pub(crate) const fn groups(&self) -> &[u16; GROUPS] {
        &self.groups
    }

    /// 0 でない最上位の節の位置。0 のときは `None`。
    pub(crate) fn top_group(&self) -> Option<usize> {
        self.groups.iter().rposition(|&g| g != 0)
    }

    /// 負の数なら `true`。0 は負ではない。
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.negative
    }

    /// 0 なら `true`。
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.top_group().is_none()
    }

    /// 絶対値。
    #[must_use]
    pub const fn abs(self) -> Self {
        Self {
            negative: false,
            groups: self.groups,
        }
    }

    /// 符号と絶対値（`u128`）から作る。
    fn from_u128(negative: bool, mut magnitude: u128) -> Self {
        let mut groups = [0; GROUPS];
        // 10000 で割った余りが一番下の節、商を次へ…を繰り返す（基数変換）
        for g in &mut groups {
            if magnitude == 0 {
                break;
            }
            // 余りは 10000 未満なので u16 に必ず収まる
            *g = u16::try_from(magnitude % u128::from(BASE)).expect("remainder < 10000");
            magnitude /= u128::from(BASE);
        }
        Self::from_groups(negative, groups)
    }

    /// 絶対値を `u128` で返す。収まらなければ `None`。
    fn magnitude_u128(&self) -> Option<u128> {
        // 上の節から「×10000 して足す」を繰り返す。溢れたら checked_* が None を返す
        self.groups.iter().rev().try_fold(0_u128, |acc, &g| {
            acc.checked_mul(u128::from(BASE))?
                .checked_add(u128::from(g))
        })
    }
}

/// 算用数字（`-123456789` の形）で書き出す。
///
/// 幅・寄せ・`+` 符号の指定（`{:>10}`, `{:+}` など）にも対応する。
impl fmt::Display for Numeral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut digits = String::new();
        match self.top_group() {
            None => digits.push('0'),
            Some(top) => {
                // 最上位の節はそのまま、それより下の節は 4 桁にゼロ埋めする
                write!(digits, "{}", self.groups[top])?;
                for g in self.groups[..top].iter().rev() {
                    write!(digits, "{g:04}")?;
                }
            }
        }
        // 符号とパディングの処理は標準ライブラリに任せる
        f.pad_integral(!self.negative, "", &digits)
    }
}

impl fmt::Debug for Numeral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Numeral({self})")
    }
}

impl Ord for Numeral {
    fn cmp(&self, other: &Self) -> Ordering {
        // 絶対値の比較: 上の節から順に比べる（配列は長さが同じなので辞書式比較でよい）
        let magnitude = self.groups.iter().rev().cmp(other.groups.iter().rev());
        match (self.negative, other.negative) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => magnitude,
            // 両方負なら絶対値が大きい方が小さい
            (true, true) => magnitude.reverse(),
        }
    }
}

impl PartialOrd for Numeral {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Neg for Numeral {
    type Output = Self;

    fn neg(self) -> Self {
        Self::from_groups(!self.negative, self.groups)
    }
}

// --- 整数型との相互変換 ---------------------------------------------------
// 同じ実装を型ごとに書くのは大変なので、宣言的マクロ（macro_rules!）で生成する。

macro_rules! impl_from_unsigned {
    ($($t:ty),*) => {$(
        impl From<$t> for Numeral {
            // usize は u128 への From 実装がないので `as` を使う（どの型も u128 に収まり損失はない）
            #[allow(clippy::cast_lossless)]
            fn from(value: $t) -> Self {
                Self::from_u128(false, value as u128)
            }
        }

        impl TryFrom<Numeral> for $t {
            type Error = TryFromNumeralError;

            fn try_from(n: Numeral) -> Result<Self, Self::Error> {
                if n.negative {
                    return Err(TryFromNumeralError(()));
                }
                let magnitude = n.magnitude_u128().ok_or(TryFromNumeralError(()))?;
                <$t>::try_from(magnitude).map_err(|_| TryFromNumeralError(()))
            }
        }
    )*};
}

macro_rules! impl_from_signed {
    ($($t:ty),*) => {$(
        impl From<$t> for Numeral {
            #[allow(clippy::cast_lossless)]
            fn from(value: $t) -> Self {
                // unsigned_abs は i8::MIN のような「正にすると溢れる値」も正しく扱える
                Self::from_u128(value < 0, value.unsigned_abs() as u128)
            }
        }

        impl TryFrom<Numeral> for $t {
            type Error = TryFromNumeralError;

            fn try_from(n: Numeral) -> Result<Self, Self::Error> {
                let magnitude = n.magnitude_u128().ok_or(TryFromNumeralError(()))?;
                // いったん i128 にしてから目的の型へ。
                // 負の場合 0 - magnitude を計算する（i128::MIN の絶対値も扱える）
                let value = if n.negative {
                    0_i128.checked_sub_unsigned(magnitude)
                } else {
                    i128::try_from(magnitude).ok()
                }
                .ok_or(TryFromNumeralError(()))?;
                <$t>::try_from(value).map_err(|_| TryFromNumeralError(()))
            }
        }
    )*};
}

impl_from_unsigned!(u8, u16, u32, u64, u128, usize);
impl_from_signed!(i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::ToString;

    #[test]
    fn groups_are_little_endian() {
        let n = Numeral::from(1_2345_6789_u64);
        assert_eq!(n.groups[..4], [6789, 2345, 1, 0]);
        assert_eq!(n.top_group(), Some(2));
    }

    #[test]
    fn display() {
        assert_eq!(Numeral::ZERO.to_string(), "0");
        assert_eq!(Numeral::from(7).to_string(), "7");
        assert_eq!(Numeral::from(10_000).to_string(), "10000");
        assert_eq!(Numeral::from(1_0000_0001).to_string(), "100000001");
        assert_eq!(Numeral::from(-12_345).to_string(), "-12345");
        assert_eq!(Numeral::from(u128::MAX).to_string(), u128::MAX.to_string());
        assert_eq!(Numeral::from(i128::MIN).to_string(), i128::MIN.to_string());
    }

    #[test]
    fn display_supports_width_and_sign() {
        assert_eq!(format!("{:>6}", Numeral::from(-42)), "   -42");
        assert_eq!(format!("{:+}", Numeral::from(42)), "+42");
        assert_eq!(format!("{:06}", Numeral::from(-42)), "-00042");
    }

    #[test]
    fn largest_value() {
        let max = Numeral::from_groups(false, [9999; GROUPS]);
        assert_eq!(max.to_string(), "9".repeat(72));
        assert_eq!(u128::try_from(max), Err(TryFromNumeralError(())));
    }

    #[test]
    fn zero_is_never_negative() {
        assert!(!Numeral::from(0).is_negative());
        assert!(!(-Numeral::ZERO).is_negative());
        assert!(!Numeral::from_groups(true, [0; GROUPS]).is_negative());
        assert_eq!(-Numeral::ZERO, Numeral::ZERO);
    }

    #[test]
    fn try_from_integers() {
        assert_eq!(u8::try_from(Numeral::from(255)), Ok(255));
        assert!(u8::try_from(Numeral::from(256)).is_err());
        assert!(u64::try_from(Numeral::from(-1)).is_err());
        assert_eq!(i8::try_from(Numeral::from(-128)), Ok(i8::MIN));
        assert!(i8::try_from(Numeral::from(-129)).is_err());
        assert_eq!(i128::try_from(Numeral::from(i128::MIN)), Ok(i128::MIN));
        assert_eq!(i128::try_from(Numeral::from(i128::MAX)), Ok(i128::MAX));
        assert!(i128::try_from(Numeral::from(u128::MAX)).is_err());
        assert_eq!(u128::try_from(Numeral::from(u128::MAX)), Ok(u128::MAX));
    }

    #[test]
    fn ordering() {
        let mut values = [5_i64, -3, 0, 1_0000, -1_0000, 9999, -2];
        let mut numerals = values.map(Numeral::from);
        values.sort_unstable();
        numerals.sort_unstable();
        assert_eq!(numerals, values.map(Numeral::from));
    }

    #[test]
    fn neg_and_abs() {
        let n = Numeral::from(123);
        assert_eq!(-n, Numeral::from(-123));
        assert_eq!((-n).abs(), n);
        assert!(!n.is_zero());
        assert!(Numeral::ZERO.is_zero());
    }
}
