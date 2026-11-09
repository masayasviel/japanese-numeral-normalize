//! 漢数字 → アラビア数字の結合テスト（公開 API だけを使う）。

use japanese_numeral_normalize::{ParseErrorKind, to_arabic};

/// 入力と期待値（アラビア数字の文字列）の表を検査する。
fn check_ok(cases: &[(&str, &str)]) {
    for &(input, expected) in cases {
        match to_arabic(input) {
            Ok(arabic) => assert_eq!(arabic, expected, "input: {input:?}"),
            Err(e) => panic!("input: {input:?}, error: {e}"),
        }
    }
}

/// 入力と期待するエラーの種類の表を検査する。
fn check_err(cases: &[(&str, ParseErrorKind)]) {
    for (input, kind) in cases {
        let err = to_arabic(input).expect_err(&format!("input: {input:?} should fail"));
        assert_eq!(err.kind(), kind, "input: {input:?}");
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
        ("一千", "1000"),
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
        ("一千万", "10000000"),
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
        ("1億200万", "102000000"),
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
        ("壱萬弐阡参佰肆拾伍", "12345"),
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
fn separators() {
    check_ok(&[
        ("1,234", "1234"),
        ("１，２３４", "1234"),
        ("1,234,567", "1234567"),
        ("1,234万5,678", "12345678"),
        ("-12,345", "-12345"),
        // 区切り位置の 3 桁ごとは検査しない（「1,0000」のような万区切りも通す）
        ("1,0000", "10000"),
    ]);
}

#[test]
fn decimals_with_large_units() {
    check_ok(&[
        ("1.5万", "15000"),
        ("１．５万", "15000"),
        ("3.14億", "314000000"),
        ("1.50万", "15000"),
        ("1億2.5万", "100025000"),
        ("0.5万", "5000"),
        ("1.0", "1"),
        ("12.0000", "12"),
        ("-2.5兆", "-2500000000000"),
        ("1.23456789億", "123456789"),
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
        ("", Empty),
        ("   ", Empty),
        ("-", Empty),
        (" + ", Empty),
        ("+ 5", InvalidCharacter(' ')),
        ("十円", InvalidCharacter('円')),
        ("一 万", InvalidCharacter(' ')),
        ("5-3", InvalidCharacter('-')),
        ("百千", MisplacedUnit),
        ("十十", MisplacedUnit),
        ("一万二億", MisplacedUnit),
        ("一万万", MisplacedUnit),
        ("一億万", MisplacedUnit),
        ("12百", MisplacedDigit),
        ("〇百", MisplacedDigit),
        ("十25", MisplacedDigit),
        ("1億12345万", MisplacedDigit),
        ("1万20000", MisplacedDigit),
        ("10000無量大数", OutOfRange),
        // 桁区切り
        (",1", MisplacedSeparator),
        ("1,,2", MisplacedSeparator),
        ("1,", MisplacedSeparator),
        ("1,万", MisplacedSeparator),
        ("千,5", MisplacedSeparator),
        ("1.2,3万", MisplacedSeparator),
        // 小数点
        (".5万", MisplacedSeparator),
        ("1.万", MisplacedSeparator),
        ("1.", MisplacedSeparator),
        ("1..5万", MisplacedSeparator),
        ("1.5千", MisplacedSeparator),
        ("千2.5万", MisplacedSeparator),
        ("1.5万2000", MisplacedDigit),
        ("1.5万千", MisplacedUnit),
        ("1.5億3万", MisplacedDigit),
        ("1.5", NotAnInteger),
        ("1.23456万", NotAnInteger),
    ]);
    let too_long = format!("1{}", "0".repeat(72));
    check_err(&[(&too_long, OutOfRange)]);
}

#[test]
fn error_display() {
    let err = to_arabic("十円").unwrap_err();
    assert_eq!(err.to_string(), "使えない文字「円」があります");
}
