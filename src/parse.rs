//! 漢数字 → 数値。
//!
//! 1. 先頭の符号を読む
//! 2. 文字列をトークン（数字・十百千・大きな単位）に分ける … [`Tokens`]
//! 3. 状態機械で「節」（大きな単位で区切られたかたまり）ごとに値を組み立てる … [`Parser`]

use alloc::vec::Vec;
use core::str::FromStr;

use crate::error::{ParseError, ParseErrorKind};
use crate::numeral::{GROUPS, Numeral};
use crate::units::{
    digit_value, is_minus, is_plus, match_large_unit, small_unit_power, tens_compound,
};

/// 扱える最大桁数（10^72 未満なので 72 桁）。
const MAX_DIGITS: usize = GROUPS * 4;

/// 漢数字・算用数字・その混在を読む。
///
/// ```
/// use japanese_numeral_normalize::Numeral;
///
/// let a: Numeral = "一億二千三百四十五万六千七百八十九".parse().unwrap();
/// let b: Numeral = "1億2345万6789".parse().unwrap();
/// let c: Numeral = "123456789".parse().unwrap();
/// assert_eq!(a, b);
/// assert_eq!(b, c);
/// ```
impl FromStr for Numeral {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s)
    }
}

fn parse(input: &str) -> Result<Numeral, ParseError> {
    // 前後の空白（全角スペースも含む）は無視する。位置は元の文字列基準で数える
    let mut pos = input.len() - input.trim_start().len();
    let mut rest = input.trim();

    // 1. 符号
    let mut negative = false;
    if let Some(c) = rest.chars().next() {
        if is_minus(c) || is_plus(c) {
            negative = is_minus(c);
            rest = &rest[c.len_utf8()..];
            pos += c.len_utf8();
        }
    }
    let end = pos + rest.len();

    // 2〜3. トークンに分けながら状態機械に流す
    let mut parser = Parser::new();
    for token in Tokens::new(rest, pos) {
        let (pos, token) = token?;
        match token {
            Token::Digit(d) => parser.digit(d, pos),
            Token::SmallUnit(power) => parser.small_unit(power, pos)?,
            // 廿 = 二十 なので、数字と「十」が続けて来たのと同じに扱う
            Token::TensCompound(d) => {
                parser.digit(d, pos);
                parser.small_unit(1, pos)?;
            }
            Token::LargeUnit(k) => parser.large_unit(k, pos)?,
        }
    }
    let groups = parser.finish(end)?;
    Ok(Numeral::from_groups(negative, groups))
}

// --- トークナイザ -------------------------------------------------------------

/// 入力の最小単位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    /// 0〜9（`2`, `２`, `二`, `弐` など）
    Digit(u8),
    /// 十・百・千。値は 10 の指数（1, 2, 3）
    SmallUnit(u8),
    /// 廿・卅・卌。値は十の位の数字（2, 3, 4）
    TensCompound(u8),
    /// 万・億・兆…。値は k（10^(4k)）
    LargeUnit(usize),
}

/// 文字列を `(位置, トークン)` の列にするイテレータ。
struct Tokens<'a> {
    rest: &'a str,
    pos: usize,
}

impl<'a> Tokens<'a> {
    const fn new(input: &'a str, pos: usize) -> Self {
        Self { rest: input, pos }
    }
}

impl Iterator for Tokens<'_> {
    type Item = Result<(usize, Token), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.rest.chars().next()?; // 空なら None で終わり
        let pos = self.pos;
        let (token, len) = if let Some(d) = digit_value(c) {
            (Token::Digit(d), c.len_utf8())
        } else if let Some(power) = small_unit_power(c) {
            (Token::SmallUnit(power), c.len_utf8())
        } else if let Some(d) = tens_compound(c) {
            (Token::TensCompound(d), c.len_utf8())
        } else if let Some((k, len)) = match_large_unit(self.rest) {
            (Token::LargeUnit(k), len)
        } else {
            self.rest = ""; // エラーの後は何も返さない
            return Some(Err(ParseError::new(
                ParseErrorKind::InvalidCharacter(c),
                pos,
            )));
        };
        self.rest = &self.rest[len..];
        self.pos += len;
        Some(Ok((pos, token)))
    }
}

// --- 状態機械 -----------------------------------------------------------------

/// 単位の付いていない数字の並び（ラン）。例: `1万2345` の `2345`
#[derive(Default)]
struct Run {
    /// 先頭の位置（エラー表示用）
    start: usize,
    /// 文字数（先頭の 0 も数える）
    len: usize,
    /// 先頭の 0 を除いた数字（上位から）
    digits: Vec<u8>,
}

impl Run {
    fn push(&mut self, d: u8, pos: usize) {
        if self.len == 0 {
            self.start = pos;
        }
        self.len += 1;
        if !(self.digits.is_empty() && d == 0) {
            self.digits.push(d);
        }
    }

    const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 値。4 桁以下のときだけ呼ぶ。
    fn value(&self) -> u16 {
        debug_assert!(self.digits.len() <= 4);
        self.digits
            .iter()
            .fold(0, |acc, &d| acc * 10 + u16::from(d))
    }
}

/// いま読んでいる節。
#[derive(Default)]
struct Section {
    /// 十百千で確定した小計（0..=9999）。例: `二千二十` まで読んだら 2020
    subtotal: u16,
    /// 直前の十百千の指数。十百千は降順にしか現れない
    last_small: Option<u8>,
    /// 十百千の付いていない数字の並び
    run: Run,
}

impl Section {
    const fn is_empty(&self) -> bool {
        self.last_small.is_none() && self.run.is_empty()
    }
}

struct Parser {
    /// 確定した値の各桁。`digits[i]` が 10^i の位（下位から）。
    ///
    /// 節ごとの配列（`[u16; 18]`）ではなく 1 桁ずつ持つのは、
    /// `12345億` のように節をまたぐ数字列をそのまま書き込めるようにするため。
    digits: [u8; MAX_DIGITS],
    /// 直前に読んだ大きな単位の k。大きな単位は降順にしか現れない
    prev_large: Option<usize>,
    section: Section,
}

impl Parser {
    fn new() -> Self {
        Self {
            digits: [0; MAX_DIGITS],
            prev_large: None,
            section: Section::default(),
        }
    }

    /// 数字: ランに追加するだけ。
    fn digit(&mut self, d: u8, pos: usize) {
        self.section.run.push(d, pos);
    }

    /// 十・百・千: 直前の数字（なければ 1）× 10^power を小計に足す。
    fn small_unit(&mut self, power: u8, pos: usize) -> Result<(), ParseError> {
        // 「百千」「十十」のような順序違反
        if self.section.last_small.is_some_and(|last| power >= last) {
            return Err(ParseError::new(ParseErrorKind::MisplacedUnit, pos));
        }
        let run = &self.section.run;
        let multiplier = match (run.len, run.digits.as_slice()) {
            (0, _) => 1,    // 「百」= 1 × 100
            (1, [d]) => *d, // 「二百」= 2 × 100
            // 「12百」「〇百」は不可
            _ => return Err(ParseError::new(ParseErrorKind::MisplacedDigit, run.start)),
        };
        self.section.subtotal += u16::from(multiplier) * 10_u16.pow(u32::from(power));
        self.section.last_small = Some(power);
        self.section.run = Run::default();
        Ok(())
    }

    /// 大きな単位: いまの節を確定し、10^(4k) の位置に書き込む。
    fn large_unit(&mut self, k: usize, pos: usize) -> Result<(), ParseError> {
        // 「一万二億」「一万万」のような順序違反
        if self.prev_large.is_some_and(|prev| k >= prev) {
            return Err(ParseError::new(ParseErrorKind::MisplacedUnit, pos));
        }
        if self.section.is_empty() {
            if self.prev_large.is_some() {
                // 「一億万」: 2 つ目以降の節が単位だけ
                return Err(ParseError::new(ParseErrorKind::MisplacedUnit, pos));
            }
            // 先頭の「万」「億」は 1 とみなす
            self.digits[4 * k] = 1;
        } else {
            self.commit_section(k)?;
        }
        self.prev_large = Some(k);
        Ok(())
    }

    /// 終端: 最後の節を単位なし（k = 0）として確定し、節ごとの配列にして返す。
    fn finish(mut self, end: usize) -> Result<[u16; GROUPS], ParseError> {
        if !self.section.is_empty() {
            self.commit_section(0)?;
        } else if self.prev_large.is_none() {
            // 数字も単位も 1 つもなかった
            return Err(ParseError::new(ParseErrorKind::Empty, end));
        }
        // 4 桁ずつまとめて節にする
        let mut groups = [0; GROUPS];
        for (group, chunk) in groups.iter_mut().zip(self.digits.chunks_exact(4)) {
            *group = chunk
                .iter()
                .rev()
                .fold(0, |acc, &d| acc * 10 + u16::from(d));
        }
        Ok(groups)
    }

    /// いまの節を 10^(4k) の位置に書き込み、次の節の準備をする。
    fn commit_section(&mut self, k: usize) -> Result<(), ParseError> {
        let section = core::mem::take(&mut self.section);
        let run = &section.run;
        let shift = 4 * k;

        if let Some(last) = section.last_small {
            // 十百千を含む節: 末尾の数字はその位より下に収まること（「千25」は可、「十25」は不可）
            if run.digits.len() > usize::from(last) {
                return Err(ParseError::new(ParseErrorKind::MisplacedDigit, run.start));
            }
            // 値は 9999 以下なので、どの節の位置にも必ず収まる
            let value = section.subtotal + run.value();
            let digits = [value / 1000, value / 100 % 10, value / 10 % 10, value % 10];
            self.write(
                shift,
                digits.map(|d| u8::try_from(d).expect("single digit")),
            );
        } else {
            // 数字だけの節（位取り式・算用数字）
            let (limit, kind) = match self.prev_large {
                // 上の節があれば、その単位の位に収まること（「1億12345万」は不可）
                Some(prev) => (4 * (prev - k), ParseErrorKind::MisplacedDigit),
                // 最上位の節は何桁でもよいが、全体で 72 桁まで（「12345億」は可）
                None => (MAX_DIGITS - shift, ParseErrorKind::OutOfRange),
            };
            if run.digits.len() > limit {
                return Err(ParseError::new(kind, run.start));
            }
            self.write(shift, &run.digits);
        }
        Ok(())
    }

    /// 上位から並んだ数字 `digits` を、最下位が 10^shift の位になるように書き込む。
    fn write(&mut self, shift: usize, digits: impl AsRef<[u8]>) {
        for (i, &d) in digits.as_ref().iter().rev().enumerate() {
            // 節の大きさを検査済みなので、すでに書いた上の節とは重ならない
            debug_assert_eq!(self.digits[shift + i], 0);
            self.digits[shift + i] = d;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn tokens(s: &str) -> Vec<Token> {
        Tokens::new(s, 0).map(|t| t.unwrap().1).collect()
    }

    #[test]
    fn tokenize() {
        use Token::{Digit, LargeUnit, SmallUnit, TensCompound};
        assert_eq!(
            tokens("二千廿1万"),
            vec![
                Digit(2),
                SmallUnit(3),
                TensCompound(2),
                Digit(1),
                LargeUnit(1)
            ]
        );
        assert_eq!(tokens("五不可思議"), vec![Digit(5), LargeUnit(16)]);
    }

    #[test]
    fn token_positions_are_byte_offsets() {
        let positions: Vec<usize> = Tokens::new("1万𥝱二", 10).map(|t| t.unwrap().0).collect();
        // "1" は 1 バイト、"万" は 3 バイト、"𥝱" は 4 バイト
        assert_eq!(positions, vec![10, 11, 14, 18]);
    }

    #[test]
    fn tokenize_stops_at_invalid_character() {
        let mut it = Tokens::new("一円二", 0);
        assert!(it.next().unwrap().is_ok());
        let err = it.next().unwrap().unwrap_err();
        assert_eq!(err.kind(), &ParseErrorKind::InvalidCharacter('円'));
        assert_eq!(err.position(), 3);
        assert!(it.next().is_none());
    }
}
