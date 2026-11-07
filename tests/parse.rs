//! 漢数字 → 数値の結合テスト（公開 API だけを使う）。

use japanese_numeral_normalize::{Numeral, ParseErrorKind};

/// 入力と期待値（算用数字の文字列）の表を検査する。
fn check_ok(cases: &[(&str, &str)]) {
    for &(input, expected) in cases {
        match input.parse::<Numeral>() {
            Ok(n) => assert_eq!(n.to_string(), expected, "input: {input:?}"),
            Err(e) => panic!("input: {input:?}, error: {e}"),
        }
    }
}

/// 入力と期待するエラー（種類・バイト位置）の表を検査する。
fn check_err(cases: &[(&str, ParseErrorKind, usize)]) {
    for (input, kind, position) in cases {
        let err = input
            .parse::<Numeral>()
            .expect_err(&format!("input: {input:?} should fail"));
        assert_eq!(err.kind(), kind, "input: {input:?}");
        assert_eq!(err.position(), *position, "input: {input:?}");
    }
}

#[test]
fn kanji_with_small_units() {
    check_ok(&[
        ("〇", "0"),
        ("零", "0"),
        ("一", "1"),
        ("十", "10"),
        ("一十", "10"),
        ("十一", "11"),
        ("二十", "20"),
        ("百", "100"),
        ("百一", "101"),
        ("千", "1000"),
        ("二千二十五", "2025"),
        ("九千九百九十九", "9999"),
    ]);
}

#[test]
fn positional() {
    check_ok(&[
        ("二〇二五", "2025"),
        ("〇〇七", "7"),
        ("2025", "2025"),
        ("２０２５", "2025"),
        ("一二三四五六七八九〇", "1234567890"),
    ]);
}

#[test]
fn large_units() {
    check_ok(&[
        ("万", "10000"),
        ("一万", "10000"),
        ("十万", "100000"),
        ("千万", "10000000"),
        ("一億", "100000000"),
        ("一億一", "100000001"),
        ("一億二千三百四十五万六千七百八十九", "123456789"),
        ("一兆二億三", "1000200000003"),
        ("一無量大数", &format!("1{}", "0".repeat(68))),
    ]);
}

#[test]
fn mixed_with_arabic_digits() {
    check_ok(&[
        ("1万2千", "12000"),
        ("3億5000万", "350000000"),
        ("1億2345万6789", "123456789"),
        ("12345億", "1234500000000"),
        ("223兆4千億4256万6千", "223400042566000"),
        ("223兆4000億4256万6000", "223400042566000"),
        ("千25", "1025"),
        ("十5", "15"),
    ]);
}

#[test]
fn daiji_and_variants() {
    check_ok(&[
        ("壱万弐千参百四拾五", "12345"),
        ("伍阡捌佰漆萬陸阡玖拾玖", "58076099"),
        ("壹貳參", "123"),
        ("廿一", "21"),
        ("卅", "30"),
        ("卌五", "45"),
        ("一那由他", &format!("1{}", "0".repeat(60))),
        ("一秭", &format!("1{}", "0".repeat(24))),
        ("一穰", &format!("1{}", "0".repeat(28))),
        ("一無量大數", &format!("1{}", "0".repeat(68))),
    ]);
}

#[test]
fn signs_and_whitespace() {
    check_ok(&[
        ("-五", "-5"),
        ("−五", "-5"),
        ("－5", "-5"),
        ("ー五", "-5"),
        ("+五", "5"),
        ("＋5", "5"),
        ("-〇", "0"),
        ("  一万 ", "10000"),
        ("　一万　", "10000"),
    ]);
}

#[test]
fn largest_value() {
    let max = "9".repeat(72);
    check_ok(&[(&max, &max)]);
    check_ok(&[("9999無量大数", &format!("9999{}", "0".repeat(68)))]);
}

#[test]
fn errors() {
    use ParseErrorKind::*;
    check_err(&[
        ("", Empty, 0),
        ("   ", Empty, 3),
        ("-", Empty, 1),
        (" + ", Empty, 2),
        ("+ 5", InvalidCharacter(' '), 1),
        ("十円", InvalidCharacter('円'), 3),
        ("一 万", InvalidCharacter(' '), 3),
        ("5-3", InvalidCharacter('-'), 1),
        ("百千", MisplacedUnit, 3),
        ("十十", MisplacedUnit, 3),
        ("一万二億", MisplacedUnit, 9),
        ("一万万", MisplacedUnit, 6),
        ("一億万", MisplacedUnit, 6),
        ("12百", MisplacedDigit, 0),
        ("〇百", MisplacedDigit, 0),
        ("十25", MisplacedDigit, 3),
        ("1億12345万", MisplacedDigit, 4),
        ("1万20000", MisplacedDigit, 4),
        ("10000無量大数", OutOfRange, 0),
    ]);
    let too_long = format!("1{}", "0".repeat(72));
    check_err(&[(&too_long, OutOfRange, 0)]);
}

#[test]
fn error_display() {
    let err = "十円".parse::<Numeral>().unwrap_err();
    assert_eq!(err.to_string(), "invalid character '円' at byte 3");
}
