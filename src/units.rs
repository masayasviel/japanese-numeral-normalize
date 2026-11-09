//! 数字・単位・符号の文字表。

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

/// 算用数字（インデックスが値）。
pub(crate) const ARABIC_DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// 標準の漢数字（インデックスが値）。
pub(crate) const KANJI_DIGITS: [char; 10] =
    ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];

/// 大字（インデックスが値）。法令で使う 壱・弐・参 のみ置き換え、それ以外は標準の字。
pub(crate) const DAIJI_DIGITS: [char; 10] =
    ['零', '壱', '弐', '参', '四', '五', '六', '七', '八', '九'];

/// 大字（インデックスが値）。すべての数字を大字にする（`DaijiFull` 用）。
pub(crate) const DAIJI_FULL_DIGITS: [char; 10] =
    ['零', '壱', '弐', '参', '肆', '伍', '陸', '漆', '捌', '玖'];

/// 十・百・千（標準）。
pub(crate) const SMALL_UNITS: [char; 3] = ['十', '百', '千'];

/// 十・百・千（大字）。法令で使うのは 拾 のみ。
pub(crate) const DAIJI_SMALL_UNITS: [char; 3] = ['拾', '百', '千'];

/// 十・百・千（すべて大字。`DaijiFull` 用）。
pub(crate) const DAIJI_FULL_SMALL_UNITS: [char; 3] = ['拾', '佰', '阡'];

// --- 入力用 ---------------------------------------------------------------------

/// 算用数字（半角・全角）1 文字を値にする。
// const fn の中では TryFrom が使えないので `as` で変換する（値は 0〜9 なので切り捨ては起きない）
#[allow(clippy::cast_possible_truncation)]
pub(crate) const fn arabic_digit_value(c: char) -> Option<u8> {
    Some(match c {
        // char は Unicode のコードポイントなので、連続した範囲は引き算で値にできる
        '0'..='9' => (c as u32 - '0' as u32) as u8,
        '０'..='９' => (c as u32 - '０' as u32) as u8,
        _ => return None,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_digits() {
        assert_eq!(arabic_digit_value('7'), Some(7));
        assert_eq!(arabic_digit_value('７'), Some(7));
        assert_eq!(arabic_digit_value('七'), None);
        // 出力用の表と入力用の関数が食い違っていないこと
        for (value, &c) in (0..).zip(&ARABIC_DIGITS) {
            assert_eq!(arabic_digit_value(c), Some(value));
        }
    }
}
