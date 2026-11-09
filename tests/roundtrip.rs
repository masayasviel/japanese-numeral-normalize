//! ラウンドトリップテスト: アラビア数字をどの形式の漢数字にしても、読み直すと元に戻ること。

use japanese_numeral_normalize::{KanjiFormat, to_arabic, to_kanji};

const FORMATS: [KanjiFormat; 6] = [
    KanjiFormat::Standard,
    KanjiFormat::ExplicitOneBeforeThousand,
    KanjiFormat::Positional,
    KanjiFormat::Daiji,
    KanjiFormat::DaijiFull,
    KanjiFormat::Mixed,
];

/// `arabic` は正規形（先頭の 0 なし、`-0` なし）のアラビア数字。
fn assert_roundtrip(arabic: &str) {
    for format in FORMATS {
        let kanji = to_kanji(arabic, format)
            .unwrap_or_else(|e| panic!("to_kanji({arabic:?}, {format:?}): {e}"));
        assert_eq!(
            to_arabic(&kanji).as_deref(),
            Ok(arabic),
            "{arabic} -> {kanji:?} ({format:?})"
        );
    }
    // アラビア数字のままでも読める
    assert_eq!(to_arabic(arabic).as_deref(), Ok(arabic));
}

/// 外部クレートを使わない簡単な擬似乱数（xorshift64）。シード固定なので毎回同じ列になる。
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// 0..n の値
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[test]
fn small_numbers() {
    for v in -1000..=20_000_i32 {
        assert_roundtrip(&v.to_string());
    }
}

#[test]
fn large_numbers() {
    assert_roundtrip(&i64::MIN.to_string());
    assert_roundtrip(&u128::MAX.to_string());
    assert_roundtrip(&"9".repeat(72));
    assert_roundtrip(&format!("-1{}", "0".repeat(71)));
}

#[test]
fn random_numbers_up_to_72_digits() {
    let mut rng = XorShift(0x2025_1107);
    for _ in 0..20_000 {
        // 桁数をばらつかせ、0 を多めに混ぜて「0 の節」が出やすくする
        let len = rng.below(72) + 1;
        let digits: String = (0..len)
            .map(|_| match rng.below(4) {
                0 => '0',
                _ => char::from_digit(u32::try_from(rng.below(10)).unwrap(), 10).unwrap(),
            })
            .collect();
        let sign = if rng.below(2) == 0 { "-" } else { "" };
        // 先頭の 0 や -0 を正規形にそろえてから試す
        let arabic = to_arabic(&format!("{sign}{digits}")).unwrap();
        assert_roundtrip(&arabic);
    }
}
