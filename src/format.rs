//! アラビア数字 → 漢数字（[`to_kanji`](crate::to_kanji) の書き出し）。

use alloc::string::String;

use crate::numeral::Numeral;
use crate::units::{
    ARABIC_DIGITS, DAIJI_DIGITS, DAIJI_FULL_DIGITS, DAIJI_FULL_SMALL_UNITS, DAIJI_SMALL_UNITS,
    KANJI_DIGITS, LARGE_UNITS, SMALL_UNITS,
};

/// 漢数字の書き方。[`to_kanji`](crate::to_kanji) の引数で選ぶ。
///
/// | 形式 | 12345 | 10000000 | 1100 | 0 |
/// | --- | --- | --- | --- | --- |
/// | [`Standard`](Self::Standard) | 一万二千三百四十五 | 千万 | 千百 | 〇 |
/// | [`ExplicitOneBeforeThousand`](Self::ExplicitOneBeforeThousand) | 一万二千三百四十五 | 一千万 | 一千百 | 〇 |
/// | [`Positional`](Self::Positional) | 一二三四五 | 一〇〇〇〇〇〇〇 | 一一〇〇 | 〇 |
/// | [`Daiji`](Self::Daiji) | 壱万弐千参百四拾五 | 壱千万 | 壱千壱百 | 零 |
/// | [`DaijiFull`](Self::DaijiFull) | 壱萬弐阡参佰肆拾伍 | 壱阡萬 | 壱阡壱佰 | 零 |
/// | [`Mixed`](Self::Mixed) | 1万2345 | 1000万 | 1100 | 0 |
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
    /// 単位式で、千の前にも「一」を書く: `一千万`, `一千百`
    ///
    /// 金額などで使う書き方。十・百の前には書かない（`百十`）。それ以外は [`Standard`](Self::Standard) と同じ。
    ExplicitOneBeforeThousand,
    /// 位取り式: `一二三四五`, `二〇二五`
    Positional,
    /// 大字: `壱万弐千参百四拾五`
    ///
    /// 法令で使う 壱・弐・参・拾 を使い、改ざん防止のため十・百・千の前にも必ず「壱」を書く。
    Daiji,
    /// すべて大字: `壱萬弐阡参佰肆拾伍`
    ///
    /// 祝儀袋などで見られる書き方。数字は 零壱弐参肆伍陸漆捌玖、十百千は 拾佰阡、万は 萬。
    /// 億以上には大字の別の字がないので、標準と同じ字を使う。
    DaijiFull,
    /// 算用数字と単位の混在: `1万2345`, `1億2000万`
    Mixed,
}

impl Numeral {
    /// 指定した形式の漢数字にする。負の数には先頭に `-` を付ける。
    pub(crate) fn to_kanji(&self, format: KanjiFormat) -> String {
        let mut out = String::new();
        if self.is_negative() {
            out.push('-');
        }
        match format {
            KanjiFormat::Standard => write_units(&mut out, self, &STANDARD),
            KanjiFormat::ExplicitOneBeforeThousand => {
                write_units(&mut out, self, &EXPLICIT_ONE_BEFORE_THOUSAND);
            }
            KanjiFormat::Daiji => write_units(&mut out, self, &DAIJI),
            KanjiFormat::DaijiFull => write_units(&mut out, self, &DAIJI_FULL),
            KanjiFormat::Positional => write_positional(&mut out, self),
            KanjiFormat::Mixed => write_mixed(&mut out, self),
        }
        out
    }
}

/// 単位式の書き方の違いをまとめた設定。ロジックは [`write_units`] で共通。
struct UnitStyle {
    /// 0〜9 の字
    digits: [char; 10],
    /// 十・百・千の字
    small_units: [char; 3],
    /// 十・百・千の前に「一」を書くか（`千` か `一千` か）。インデックスは `small_units` と同じ
    explicit_one: [bool; 3],
    /// 万の字（`万` / `萬`）。億以上は大字でも同じ字なので [`LARGE_UNITS`] を使う
    ten_thousand: &'static str,
}

const STANDARD: UnitStyle = UnitStyle {
    digits: KANJI_DIGITS,
    small_units: SMALL_UNITS,
    explicit_one: [false, false, false],
    ten_thousand: "万",
};

const EXPLICIT_ONE_BEFORE_THOUSAND: UnitStyle = UnitStyle {
    // 千の前だけ「一」を書く
    explicit_one: [false, false, true],
    ..STANDARD
};

const DAIJI: UnitStyle = UnitStyle {
    digits: DAIJI_DIGITS,
    small_units: DAIJI_SMALL_UNITS,
    explicit_one: [true, true, true],
    ten_thousand: "万",
};

const DAIJI_FULL: UnitStyle = UnitStyle {
    digits: DAIJI_FULL_DIGITS,
    small_units: DAIJI_FULL_SMALL_UNITS,
    explicit_one: [true, true, true],
    ten_thousand: "萬",
};

/// 単位式: 節ごとに「千百十一」で書き、大きな単位を付ける。
fn write_units(out: &mut String, n: &Numeral, style: &UnitStyle) {
    let Some(top) = n.top_group() else {
        out.push(style.digits[0]);
        return;
    };
    for k in (0..=top).rev() {
        let group = n.group(k);
        // 0 の節は単位ごと省く（100000001 は「一億一」。「一億〇万一」とは書かない）
        if group == [0; 4] {
            continue;
        }
        write_group(out, group, style);
        out.push_str(if k == 1 {
            style.ten_thousand
        } else {
            LARGE_UNITS[k]
        });
    }
}

/// 1 つの節を千・百・十・一の位の順に書く。`group` は一・十・百・千の位の順に並ぶ。
fn write_group(out: &mut String, group: [u8; 4], style: &UnitStyle) {
    for (place, &d) in group.iter().enumerate().rev() {
        let d = usize::from(d);
        if d == 0 {
            continue;
        }
        // 一の位は数字だけ
        let Some(unit) = place.checked_sub(1) else {
            out.push(style.digits[d]);
            continue;
        };
        // 標準は「一千」ではなく「千」と書く（大字は「壱千」）
        if d != 1 || style.explicit_one[unit] {
            out.push(style.digits[d]);
        }
        out.push(style.small_units[unit]);
    }
}

/// 位取り式: 各桁を漢数字に置き換えるだけ。
fn write_positional(out: &mut String, n: &Numeral) {
    let digits = n.significant_digits();
    if digits.is_empty() {
        out.push(KANJI_DIGITS[0]);
    }
    for &d in digits.iter().rev() {
        out.push(KANJI_DIGITS[usize::from(d)]);
    }
}

/// 混在: 節を算用数字で書き、大きな単位を付ける。
fn write_mixed(out: &mut String, n: &Numeral) {
    let Some(top) = n.top_group() else {
        out.push(ARABIC_DIGITS[0]);
        return;
    };
    for k in (0..=top).rev() {
        let group = n.group(k);
        if group == [0; 4] {
            continue;
        }
        // 節の中の上位の 0 は書かない（「1億0200万」ではなく「1億200万」）
        for &d in group.iter().rev().skip_while(|&&d| d == 0) {
            out.push(ARABIC_DIGITS[usize::from(d)]);
        }
        out.push_str(LARGE_UNITS[k]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numeral::MAX_DIGITS;
    use crate::to_kanji;
    use alloc::format;

    fn check(format: KanjiFormat, cases: &[(&str, &str)]) {
        for &(input, expected) in cases {
            assert_eq!(
                to_kanji(input, format).as_deref(),
                Ok(expected),
                "{input} with {format:?}"
            );
        }
    }

    #[test]
    fn standard() {
        check(
            KanjiFormat::Standard,
            &[
                ("0", "〇"),
                ("1", "一"),
                ("10", "十"),
                ("11", "十一"),
                ("20", "二十"),
                ("101", "百一"),
                ("110", "百十"),
                ("1000", "千"),
                ("1010", "千十"),
                ("1100", "千百"),
                ("2025", "二千二十五"),
                ("9999", "九千九百九十九"),
                ("10000", "一万"),
                ("10001", "一万一"),
                ("11000", "一万千"),
                ("10000000", "千万"),
                ("100000000", "一億"),
                ("100000001", "一億一"),
                ("123456789", "一億二千三百四十五万六千七百八十九"),
                ("-5", "-五"),
                ("-0", "〇"),
                ("1,234", "千二百三十四"),
            ],
        );
    }

    #[test]
    fn explicit_one_before_thousand() {
        check(
            KanjiFormat::ExplicitOneBeforeThousand,
            &[
                ("0", "〇"),
                ("10", "十"),
                ("110", "百十"),
                ("1000", "一千"),
                ("1100", "一千百"),
                ("1111", "一千百十一"),
                ("2000", "二千"),
                ("10000000", "一千万"),
                ("11000", "一万一千"),
                ("100010000000", "一千億一千万"),
            ],
        );
    }

    #[test]
    fn positional() {
        check(
            KanjiFormat::Positional,
            &[
                ("0", "〇"),
                ("7", "七"),
                ("2025", "二〇二五"),
                ("-10000", "-一〇〇〇〇"),
            ],
        );
    }

    #[test]
    fn daiji() {
        check(
            KanjiFormat::Daiji,
            &[
                ("0", "零"),
                ("10", "壱拾"),
                ("111", "壱百壱拾壱"),
                ("1000", "壱千"),
                ("1110", "壱千壱百壱拾"),
                ("10000", "壱万"),
                ("12345", "壱万弐千参百四拾五"),
                ("300000000", "参億"),
            ],
        );
    }

    #[test]
    fn daiji_full() {
        check(
            KanjiFormat::DaijiFull,
            &[
                ("0", "零"),
                ("10", "壱拾"),
                ("1110", "壱阡壱佰壱拾"),
                ("10000", "壱萬"),
                ("12345", "壱萬弐阡参佰肆拾伍"),
                ("10000000", "壱阡萬"),
                ("58076099", "伍阡捌佰漆萬陸阡玖拾玖"),
                ("1000000000000", "壱兆"),
                ("100010000", "壱億壱萬"),
            ],
        );
    }

    #[test]
    fn mixed() {
        check(
            KanjiFormat::Mixed,
            &[
                ("0", "0"),
                ("9999", "9999"),
                ("10000", "1万"),
                ("100000001", "1億1"),
                ("102000000", "1億200万"),
                ("123456789", "1億2345万6789"),
                ("-120000000", "-1億2000万"),
            ],
        );
    }

    #[test]
    fn every_large_unit() {
        // 1 の後ろに 0 が 4k 個 = 10^(4k)
        for (k, unit) in LARGE_UNITS.iter().enumerate().skip(1) {
            let input = format!("1{}", "0".repeat(4 * k));
            check(KanjiFormat::Standard, &[(&input, &format!("一{unit}"))]);
        }
    }

    #[test]
    fn largest_value() {
        let max = to_kanji(&"9".repeat(MAX_DIGITS), KanjiFormat::Standard).unwrap();
        assert!(max.starts_with("九千九百九十九無量大数九千九百九十九不可思議"));
        assert!(max.ends_with("九千九百九十九万九千九百九十九"));
    }
}
