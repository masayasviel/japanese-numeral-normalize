//! ラウンドトリップテスト: どの形式で書き出しても、読み直すと元の値に戻ること。

use japanese_numeral_normalize::{KanjiFormat, Numeral};

const FORMATS: [KanjiFormat; 4] = [
    KanjiFormat::Standard,
    KanjiFormat::Positional,
    KanjiFormat::Daiji,
    KanjiFormat::Mixed,
];

fn assert_roundtrip(n: Numeral) {
    for format in FORMATS {
        let kanji = n.to_kanji_with(format);
        let back: Numeral = kanji
            .parse()
            .unwrap_or_else(|e| panic!("{n} -> {kanji:?} ({format:?}): {e}"));
        assert_eq!(back, n, "{n} -> {kanji:?} ({format:?})");
    }
    // 算用数字（Display）も読み直せる
    assert_eq!(n.to_string().parse::<Numeral>(), Ok(n));
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
        assert_roundtrip(Numeral::from(v));
    }
}

#[test]
fn integer_limits() {
    for n in [
        Numeral::from(i64::MIN),
        Numeral::from(i64::MAX),
        Numeral::from(i128::MIN),
        Numeral::from(u128::MAX),
    ] {
        assert_roundtrip(n);
    }
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
        let n: Numeral = format!("{sign}{digits}").parse().unwrap();
        assert_roundtrip(n);
    }
}

#[test]
fn integers_survive_numeral() {
    let mut rng = XorShift(42);
    for _ in 0..10_000 {
        let v = i64::from_ne_bytes(rng.next().to_ne_bytes());
        let n = Numeral::from(v);
        assert_eq!(i64::try_from(n), Ok(v));
        assert_eq!(n.to_string(), v.to_string());
    }
}
