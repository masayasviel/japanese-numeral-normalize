//! 内部表現 [`Numeral`] と、アラビア数字の文字列の読み書き。

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::{ParseError, ParseErrorKind};
use crate::units::{ARABIC_DIGITS, arabic_digit_value, is_comma, is_minus, is_plus};

/// 節（4 桁の組）の数。節 k は 10^(4k) の位から始まる 4 桁。
///
/// k = 0..=17 なので、最上位は無量大数（10^68）の節。
pub(crate) const GROUPS: usize = 18;

/// 扱える最大桁数。絶対値は 10^72 未満なので 72 桁。
pub(crate) const MAX_DIGITS: usize = GROUPS * 4;

/// 符号付きの整数を、10 進の桁の並びのまま持つ。
///
/// 入力も出力も文字列なので、数値型（`i64` / `u128` など）には変換しない。
/// 桁の並びで持つので、`u128`（約 3.4×10^38）を超える無量大数の位まで扱える。
// 0 のときは必ず `negative == false` にする（-0 を作らない）。
#[derive(Debug)]
pub(crate) struct Numeral {
    negative: bool,
    /// 各桁（0..=9）。`digits[i]` が 10^i の位（下位から並ぶ）。
    digits: [u8; MAX_DIGITS],
}

impl Numeral {
    /// 符号と各桁から作る。0 なら符号を正にそろえる（正規化）。
    pub(crate) fn new(negative: bool, digits: [u8; MAX_DIGITS]) -> Self {
        debug_assert!(digits.iter().all(|&d| d < 10), "each digit must be < 10");
        let is_zero = digits.iter().all(|&d| d == 0);
        Self {
            negative: negative && !is_zero,
            digits,
        }
    }

    /// 負の数なら `true`。0 は負ではない。
    pub(crate) const fn is_negative(&self) -> bool {
        self.negative
    }

    /// 0 でない最上位の桁までの各桁（下位から）。0 のときは空。
    pub(crate) fn significant_digits(&self) -> &[u8] {
        let len = self
            .digits
            .iter()
            .rposition(|&d| d != 0)
            .map_or(0, |i| i + 1);
        &self.digits[..len]
    }

    /// 0 でない最上位の節の位置。0 のときは `None`。
    pub(crate) fn top_group(&self) -> Option<usize> {
        let len = self.significant_digits().len();
        len.checked_sub(1).map(|i| i / 4)
    }

    /// 節 k の 4 桁。一・十・百・千の位の順に並ぶ。
    pub(crate) fn group(&self, k: usize) -> [u8; 4] {
        core::array::from_fn(|i| self.digits[4 * k + i])
    }

    /// アラビア数字の文字列を読む。
    ///
    /// 前後の空白・先頭の符号・全角数字・桁区切り（数字と数字の間のみ）を受け付ける。
    /// 先頭の 0 は無視する（`007` は 7）。
    pub(crate) fn parse_arabic(input: &str) -> Result<Self, ParseError> {
        let (negative, body) = split_sign(input);

        // 先頭の 0 を除いた数字（上位から）
        let mut significant = Vec::new();
        // 数字を 1 つでも読んだら true
        let mut has_digit = false;
        // 直前に桁区切りを読んだら true。次は数字でなければならない
        let mut pending_comma = false;

        for c in body.chars() {
            if let Some(d) = arabic_digit_value(c) {
                has_digit = true;
                pending_comma = false;
                if !(significant.is_empty() && d == 0) {
                    significant.push(d);
                }
            } else if is_comma(c) {
                // 「,1」「1,,2」: 桁区切りの直前が数字でない
                if !has_digit || pending_comma {
                    return Err(ParseError::new(ParseErrorKind::MisplacedSeparator));
                }
                pending_comma = true;
            } else {
                return Err(ParseError::new(ParseErrorKind::InvalidCharacter(c)));
            }
        }

        if pending_comma {
            // 「1,」: 桁区切りの後ろに数字がない
            return Err(ParseError::new(ParseErrorKind::MisplacedSeparator));
        }
        if !has_digit {
            return Err(ParseError::new(ParseErrorKind::Empty));
        }
        if significant.len() > MAX_DIGITS {
            return Err(ParseError::new(ParseErrorKind::OutOfRange));
        }

        // 上位から並んだ数字を、下位から詰め直す
        let mut digits = [0; MAX_DIGITS];
        for (slot, &d) in digits.iter_mut().zip(significant.iter().rev()) {
            *slot = d;
        }
        Ok(Self::new(negative, digits))
    }

    /// アラビア数字の文字列にする。負の数には先頭に `-` を付ける。
    pub(crate) fn to_arabic(&self) -> String {
        let mut out = String::new();
        if self.negative {
            out.push('-');
        }
        let digits = self.significant_digits();
        if digits.is_empty() {
            out.push(ARABIC_DIGITS[0]);
        }
        for &d in digits.iter().rev() {
            out.push(ARABIC_DIGITS[usize::from(d)]);
        }
        out
    }
}

/// 前後の空白を除き、先頭の符号を読む。
///
/// `(負か, 符号より後ろの文字列)` を返す。
pub(crate) fn split_sign(input: &str) -> (bool, &str) {
    // 前後の空白（全角スペースも含む）は無視する
    let rest = input.trim();
    if let Some(c) = rest.chars().next()
        && (is_minus(c) || is_plus(c))
    {
        return (is_minus(c), &rest[c.len_utf8()..]);
    }
    (false, rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::ToString;

    fn arabic(input: &str) -> String {
        Numeral::parse_arabic(input)
            .unwrap_or_else(|e| panic!("input: {input:?}, error: {e}"))
            .to_arabic()
    }

    fn error(input: &str) -> ParseErrorKind {
        Numeral::parse_arabic(input).unwrap_err().kind().clone()
    }

    #[test]
    fn read_and_write_arabic() {
        let cases = [
            ("0", "0"),
            ("7", "7"),
            ("10000", "10000"),
            ("100000001", "100000001"),
            ("-12345", "-12345"),
            ("+5", "5"),
            ("−5", "-5"),
            ("007", "7"),
            ("１２３", "123"),
            ("1,234", "1234"),
            ("１，２３４", "1234"),
            ("1,0000", "10000"),
            ("  42 ", "42"),
            ("　42　", "42"),
        ];
        for (input, expected) in cases {
            assert_eq!(arabic(input), expected, "input: {input:?}");
        }
    }

    #[test]
    fn zero_is_never_negative() {
        assert_eq!(arabic("-0"), "0");
        assert_eq!(arabic("-000"), "0");
        assert!(!Numeral::new(true, [0; MAX_DIGITS]).is_negative());
    }

    #[test]
    fn largest_value() {
        let max = "9".repeat(MAX_DIGITS);
        assert_eq!(arabic(&max), max);
        // 先頭の 0 は桁数に数えない
        assert_eq!(arabic(&format!("{}1", "0".repeat(100))), "1");
        let too_long = format!("1{}", "0".repeat(MAX_DIGITS));
        assert_eq!(error(&too_long), ParseErrorKind::OutOfRange);
        assert_eq!(error(&format!("-{too_long}")), ParseErrorKind::OutOfRange);
    }

    #[test]
    fn errors() {
        use ParseErrorKind::*;
        let cases = [
            ("", Empty),
            ("   ", Empty),
            ("-", Empty),
            (" + ", Empty),
            ("12a", InvalidCharacter('a')),
            ("一", InvalidCharacter('一')),
            ("1万", InvalidCharacter('万')),
            ("1 2", InvalidCharacter(' ')),
            ("1.5", InvalidCharacter('.')),
            (",1", MisplacedSeparator),
            ("1,,2", MisplacedSeparator),
            ("1,", MisplacedSeparator),
            ("-,1", MisplacedSeparator),
        ];
        for (input, expected) in cases {
            assert_eq!(error(input), expected, "input: {input:?}");
        }
    }

    #[test]
    fn groups_are_little_endian() {
        let n = Numeral::parse_arabic("123456789").unwrap();
        assert_eq!(n.group(0), [9, 8, 7, 6]);
        assert_eq!(n.group(1), [5, 4, 3, 2]);
        assert_eq!(n.group(2), [1, 0, 0, 0]);
        assert_eq!(n.top_group(), Some(2));
        assert_eq!(Numeral::parse_arabic("0").unwrap().top_group(), None);
    }

    #[test]
    fn error_display() {
        let err = Numeral::parse_arabic("12a").unwrap_err();
        assert_eq!(err.to_string(), "使えない文字「a」があります");
    }
}
