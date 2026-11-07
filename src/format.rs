//! 数値 → 漢数字。

use alloc::string::{String, ToString};
use core::fmt::Write as _;

use crate::numeral::Numeral;
use crate::units::{DAIJI_DIGITS, DAIJI_SMALL_UNITS, KANJI_DIGITS, LARGE_UNITS, SMALL_UNITS};

/// 漢数字の書き方。
///
/// | 形式 | 12345 | 10000000 | 0 |
/// | --- | --- | --- | --- |
/// | [`Standard`](Self::Standard) | 一万二千三百四十五 | 千万 | 〇 |
/// | [`Positional`](Self::Positional) | 一二三四五 | 一〇〇〇〇〇〇〇 | 〇 |
/// | [`Daiji`](Self::Daiji) | 壱万弐千参百四拾五 | 壱千万 | 零 |
/// | [`Mixed`](Self::Mixed) | 1万2345 | 1000万 | 0 |
// `#[non_exhaustive]` を付けると、クレートの外では match に `_ =>` が必須になる。
// 形式を後から追加しても利用側のコードが壊れない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum KanjiFormat {
    /// 単位式（既定）: `一万二千三百四十五`
    ///
    /// 十・百・千の前の「一」は省略し（`千万`）、万以上の単位の前の「一」は書く（`一億`）。
    #[default]
    Standard,
    /// 位取り式: `一二三四五`, `二〇二五`
    Positional,
    /// 大字: `壱万弐千参百四拾五`
    ///
    /// 法令で使う 壱・弐・参・拾 を使い、改ざん防止のため十・百・千の前にも必ず「壱」を書く。
    Daiji,
    /// 算用数字と単位の混在: `1万2345`, `1億2000万`
    Mixed,
}

impl Numeral {
    /// 標準の漢数字（[`KanjiFormat::Standard`]）にする。
    ///
    /// ```
    /// use japanese_numeral_normalize::Numeral;
    ///
    /// assert_eq!(Numeral::from(12345).to_kanji(), "一万二千三百四十五");
    /// assert_eq!(Numeral::from(-1_0000_0000).to_kanji(), "-一億");
    /// ```
    #[must_use]
    pub fn to_kanji(&self) -> String {
        self.to_kanji_with(KanjiFormat::Standard)
    }

    /// 指定した形式の漢数字にする。負の数には先頭に `-` を付ける。
    ///
    /// ```
    /// use japanese_numeral_normalize::{KanjiFormat, Numeral};
    ///
    /// let n = Numeral::from(1_2000_0000);
    /// assert_eq!(n.to_kanji_with(KanjiFormat::Standard), "一億二千万");
    /// assert_eq!(n.to_kanji_with(KanjiFormat::Positional), "一二〇〇〇〇〇〇〇");
    /// assert_eq!(n.to_kanji_with(KanjiFormat::Daiji), "壱億弐千万");
    /// assert_eq!(n.to_kanji_with(KanjiFormat::Mixed), "1億2000万");
    /// ```
    #[must_use]
    pub fn to_kanji_with(&self, format: KanjiFormat) -> String {
        let mut out = String::new();
        if self.is_negative() {
            out.push('-');
        }
        match format {
            KanjiFormat::Standard => write_units(&mut out, self, &STANDARD),
            KanjiFormat::Daiji => write_units(&mut out, self, &DAIJI),
            KanjiFormat::Positional => write_positional(&mut out, self),
            KanjiFormat::Mixed => write_mixed(&mut out, self),
        }
        out
    }
}

/// 単位式の書き方の違い（標準と大字）をまとめた設定。
struct UnitStyle {
    digits: [char; 10],
    /// 十・百・千
    small_units: [char; 3],
    /// 十・百・千の前の「一」を書くか（`千` か `壱千` か）
    explicit_one: bool,
}

const STANDARD: UnitStyle = UnitStyle {
    digits: KANJI_DIGITS,
    small_units: SMALL_UNITS,
    explicit_one: false,
};

const DAIJI: UnitStyle = UnitStyle {
    digits: DAIJI_DIGITS,
    small_units: DAIJI_SMALL_UNITS,
    explicit_one: true,
};

/// 単位式: 節ごとに「千百十一」で書き、大きな単位を付ける。
fn write_units(out: &mut String, n: &Numeral, style: &UnitStyle) {
    let Some(top) = n.top_group() else {
        out.push(style.digits[0]);
        return;
    };
    for k in (0..=top).rev() {
        let group = n.groups()[k];
        // 0 の節は単位ごと省く（1_0000_0001 は「一億一」。「一億〇万一」とは書かない）
        if group == 0 {
            continue;
        }
        write_group(out, group, style);
        out.push_str(LARGE_UNITS[k]);
    }
}

/// 1 つの節（1〜9999）を千・百・十・一の位に分けて書く。
fn write_group(out: &mut String, group: u16, style: &UnitStyle) {
    // (位の大きさ, 単位の文字)。一の位は単位なし
    let places = [
        (1000, Some(style.small_units[2])),
        (100, Some(style.small_units[1])),
        (10, Some(style.small_units[0])),
        (1, None),
    ];
    for (place, unit) in places {
        let d = usize::from(group / place % 10);
        if d == 0 {
            continue;
        }
        match unit {
            Some(unit) => {
                // 「一千」ではなく「千」と書く（大字は「壱千」）
                if d != 1 || style.explicit_one {
                    out.push(style.digits[d]);
                }
                out.push(unit);
            }
            None => out.push(style.digits[d]),
        }
    }
}

/// 位取り式: 算用数字の各桁を漢数字に置き換えるだけ。
fn write_positional(out: &mut String, n: &Numeral) {
    for c in n.abs().to_string().chars() {
        let d = c
            .to_digit(10)
            .expect("Display of Numeral writes only ASCII digits");
        out.push(KANJI_DIGITS[d as usize]);
    }
}

/// 混在: 節の値を算用数字で書き、大きな単位を付ける。
fn write_mixed(out: &mut String, n: &Numeral) {
    let Some(top) = n.top_group() else {
        out.push('0');
        return;
    };
    for k in (0..=top).rev() {
        let group = n.groups()[k];
        if group != 0 {
            // String への write! は失敗しない
            let _ = write!(out, "{group}{}", LARGE_UNITS[k]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numeral::GROUPS;

    fn check(format: KanjiFormat, cases: &[(i64, &str)]) {
        for &(value, expected) in cases {
            assert_eq!(
                Numeral::from(value).to_kanji_with(format),
                expected,
                "{value} with {format:?}"
            );
        }
    }

    #[test]
    fn standard() {
        check(
            KanjiFormat::Standard,
            &[
                (0, "〇"),
                (1, "一"),
                (10, "十"),
                (11, "十一"),
                (20, "二十"),
                (101, "百一"),
                (110, "百十"),
                (1000, "千"),
                (1010, "千十"),
                (2025, "二千二十五"),
                (9999, "九千九百九十九"),
                (1_0000, "一万"),
                (1_0001, "一万一"),
                (1_1000, "一万千"),
                (1000_0000, "千万"),
                (1_0000_0000, "一億"),
                (1_0000_0001, "一億一"),
                (1_2345_6789, "一億二千三百四十五万六千七百八十九"),
                (-5, "-五"),
            ],
        );
    }

    #[test]
    fn positional() {
        check(
            KanjiFormat::Positional,
            &[
                (0, "〇"),
                (7, "七"),
                (2025, "二〇二五"),
                (-1_0000, "-一〇〇〇〇"),
            ],
        );
    }

    #[test]
    fn daiji() {
        check(
            KanjiFormat::Daiji,
            &[
                (0, "零"),
                (10, "壱拾"),
                (111, "壱百壱拾壱"),
                (1000, "壱千"),
                (1_2345, "壱万弐千参百四拾五"),
                (3_0000_0000, "参億"),
            ],
        );
    }

    #[test]
    fn mixed() {
        check(
            KanjiFormat::Mixed,
            &[
                (0, "0"),
                (9999, "9999"),
                (1_0000, "1万"),
                (1_0000_0001, "1億1"),
                (1_2345_6789, "1億2345万6789"),
                (-1_2000_0000, "-1億2000万"),
            ],
        );
    }

    #[test]
    fn every_large_unit() {
        // k 番目の節だけ 1 の数 = 10^(4k)
        for (k, unit) in LARGE_UNITS.iter().enumerate().skip(1) {
            let mut groups = [0; GROUPS];
            groups[k] = 1;
            let n = Numeral::from_groups(false, groups);
            assert_eq!(n.to_kanji(), alloc::format!("一{unit}"));
        }
    }

    #[test]
    fn largest_value() {
        let max = Numeral::from_groups(false, [9999; GROUPS]);
        let kanji = max.to_kanji();
        assert!(kanji.starts_with("九千九百九十九無量大数九千九百九十九不可思議"));
        assert!(kanji.ends_with("九千九百九十九万九千九百九十九"));
    }
}
