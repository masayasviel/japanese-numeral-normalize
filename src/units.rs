//! 数字・単位の文字表。

use crate::numeral::GROUPS;

/// 大きな単位の正規表記。インデックス `k` が 10^(4k) を表す（0 は単位なし）。
///
/// 表記ゆれのあるものは 𥝱（秭）・穣（穰）・那由多（那由他）を採用する。
/// 𥝱 は基本多言語面の外（U+25771）にあり、UTF-8 で 4 バイトになる点に注意。
pub(crate) const LARGE_UNITS: [&str; GROUPS] = [
    "",
    "万",
    "億",
    "兆",
    "京",
    "垓",
    "𥝱",
    "穣",
    "溝",
    "澗",
    "正",
    "載",
    "極",
    "恒河沙",
    "阿僧祇",
    "那由多",
    "不可思議",
    "無量大数",
];

/// 標準の漢数字（インデックスが値）。
pub(crate) const KANJI_DIGITS: [char; 10] =
    ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];

/// 大字（インデックスが値）。法令で使う 壱・弐・参 のみ置き換え、それ以外は標準の字。
pub(crate) const DAIJI_DIGITS: [char; 10] =
    ['零', '壱', '弐', '参', '四', '五', '六', '七', '八', '九'];

/// 大字（インデックスが値）。すべての数字を大字にする（出力形式 `DaijiFull` 用）。
pub(crate) const DAIJI_FULL_DIGITS: [char; 10] =
    ['零', '壱', '弐', '参', '肆', '伍', '陸', '漆', '捌', '玖'];

/// 十・百・千（標準）。
pub(crate) const SMALL_UNITS: [char; 3] = ['十', '百', '千'];

/// 十・百・千（大字）。法令で使うのは 拾 のみ。
pub(crate) const DAIJI_SMALL_UNITS: [char; 3] = ['拾', '百', '千'];

/// 十・百・千（大字フル）。古い大字表記の 佰・阡 を使う。
pub(crate) const DAIJI_FULL_SMALL_UNITS: [char; 3] = ['拾', '佰', '阡'];

// --- 入力（漢数字 → 数値）用 ---------------------------------------------------

/// 入力でのみ受け付ける大きな単位の異体字・別表記。
const LARGE_UNIT_ALIASES: [(&str, usize); 5] = [
    ("萬", 1),
    ("秭", 6),
    ("穰", 7),
    ("那由他", 15),
    ("無量大數", 17),
];

/// 1 文字の数字を値にする。算用数字（半角・全角）、漢数字、大字・異体字に対応。
// const fn の中では TryFrom が使えないので `as` で変換する（値は 0〜9 なので切り捨ては起きない）
#[allow(clippy::cast_possible_truncation)]
pub(crate) const fn digit_value(c: char) -> Option<u8> {
    Some(match c {
        // char は Unicode のコードポイントなので、連続した範囲は引き算で値にできる
        '0'..='9' => (c as u32 - '0' as u32) as u8,
        '０'..='９' => (c as u32 - '０' as u32) as u8,
        '〇' | '零' => 0,
        '一' | '壱' | '壹' | '弌' => 1,
        '二' | '弐' | '貳' | '貮' | '弍' => 2,
        '三' | '参' | '參' | '弎' => 3,
        '四' | '肆' | '亖' => 4,
        '五' | '伍' => 5,
        '六' | '陸' => 6,
        '七' | '漆' => 7,
        '八' | '捌' => 8,
        '九' | '玖' => 9,
        _ => return None,
    })
}

/// 十・百・千（と大字・異体字）を 10 の指数（1, 2, 3）にする。
pub(crate) const fn small_unit_power(c: char) -> Option<u8> {
    Some(match c {
        '十' | '拾' | '什' => 1,
        '百' | '陌' | '佰' => 2,
        '千' | '阡' | '仟' => 3,
        _ => return None,
    })
}

/// 廿（二十）・卅（三十）・卌（四十）を十の位の数字にする。
pub(crate) const fn tens_compound(c: char) -> Option<u8> {
    Some(match c {
        '廿' => 2,
        '卅' => 3,
        '卌' => 4,
        _ => return None,
    })
}

/// `s` の先頭にある大きな単位を探し、`(k, バイト長)` を返す。
///
/// 恒河沙・不可思議などは複数文字なので、1 文字ずつではなく前方一致で探す。
pub(crate) fn match_large_unit(s: &str) -> Option<(usize, usize)> {
    let canonical = LARGE_UNITS
        .iter()
        .enumerate()
        .skip(1) // k = 0 は単位なし（空文字列）なので除く
        .map(|(k, name)| (*name, k));
    canonical
        .chain(LARGE_UNIT_ALIASES)
        .find(|(name, _)| s.starts_with(name))
        .map(|(name, k)| (k, name.len()))
}

/// 負号として扱う文字（先頭のみ）。
pub(crate) const fn is_minus(c: char) -> bool {
    matches!(
        c,
        '-' | '－' | '−' | '‐' | '‑' | '–' | '—' | '―' | 'ー' | 'ｰ' | '⁻' | '₋' | '﹣'
    )
}

/// 正号として扱う文字（先頭のみ）。
pub(crate) const fn is_plus(c: char) -> bool {
    matches!(c, '+' | '＋' | '⁺' | '₊' | '﹢')
}

/// 桁区切りとして扱う文字（数字と数字の間のみ）。
pub(crate) const fn is_comma(c: char) -> bool {
    matches!(c, ',' | '，')
}

/// 小数点として扱う文字。
pub(crate) const fn is_point(c: char) -> bool {
    matches!(c, '.' | '．')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits() {
        assert_eq!(digit_value('7'), Some(7));
        assert_eq!(digit_value('７'), Some(7));
        assert_eq!(digit_value('七'), Some(7));
        assert_eq!(digit_value('漆'), Some(7));
        assert_eq!(digit_value('十'), None);
        // 出力用の表と入力用の関数が食い違っていないこと
        for digits in [&KANJI_DIGITS, &DAIJI_DIGITS, &DAIJI_FULL_DIGITS] {
            for (value, &kanji) in (0..).zip(digits) {
                assert_eq!(digit_value(kanji), Some(value));
            }
        }
    }

    #[test]
    fn small_units() {
        for units in [&SMALL_UNITS, &DAIJI_SMALL_UNITS, &DAIJI_FULL_SMALL_UNITS] {
            for (power, &unit) in (1..).zip(units) {
                assert_eq!(small_unit_power(unit), Some(power));
            }
        }
    }

    #[test]
    fn large_units() {
        for (k, name) in LARGE_UNITS.iter().enumerate().skip(1) {
            assert_eq!(match_large_unit(name), Some((k, name.len())));
        }
        assert_eq!(match_large_unit("恒河沙一"), Some((13, "恒河沙".len())));
        assert_eq!(match_large_unit("那由他"), Some((15, "那由他".len())));
        // 𥝱 は UTF-8 で 4 バイト
        assert_eq!(match_large_unit("𥝱"), Some((6, 4)));
        // 大字フル（DaijiFull）で書き出す 萬 も読める
        assert_eq!(match_large_unit("萬"), Some((1, "萬".len())));
        assert_eq!(match_large_unit("円"), None);
        assert_eq!(match_large_unit(""), None);
    }
}
