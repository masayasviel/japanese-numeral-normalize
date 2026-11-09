//! 数字・単位・符号の文字表。

/// 算用数字（インデックスが値）。
pub(crate) const ARABIC_DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

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
