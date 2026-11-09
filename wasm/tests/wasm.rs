//! WebAssembly 上の結合テスト（wasm-bindgen-test）。
//!
//! wasm32 にビルドし、wasm-bindgen-test-runner で Node.js 上で実行する（`make test-wasm`）。
//! 関数は Rust から呼ぶが、境界を越えて JavaScript の値（`Error` など）としてやり取りしたものを検査する。

#![cfg(target_arch = "wasm32")]

use japanese_numeral_normalize_wasm::{KanjiFormat, to_arabic, to_kanji};
use js_sys::{Error, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::wasm_bindgen_test;

const FORMATS: [KanjiFormat; 6] = [
    KanjiFormat::Standard,
    KanjiFormat::ExplicitOneBeforeThousand,
    KanjiFormat::Positional,
    KanjiFormat::Daiji,
    KanjiFormat::DaijiFull,
    KanjiFormat::Mixed,
];

/// JavaScript のオブジェクトのプロパティを読む。
fn prop(target: &JsValue, key: &str) -> JsValue {
    Reflect::get(target, &JsValue::from_str(key)).unwrap_or_else(|_| panic!("get {key}"))
}

/// `err` が `ParseError` の形（`name`・`kind`・`position` を持つ `Error`）で、
/// 原因が `kind`、位置が `position`（UTF-16 のインデックス）であること。
fn assert_parse_error(input: &str, err: &Error, kind: &str, position: usize) {
    let value = JsValue::from(err);
    assert!(value.is_instance_of::<Error>(), "input: {input:?}");
    assert_eq!(String::from(err.name()), "ParseError", "input: {input:?}");
    assert_eq!(
        prop(&value, "kind").as_string().as_deref(),
        Some(kind),
        "input: {input:?}"
    );
    assert_eq!(prop(&value, "position"), position, "input: {input:?}");
}

#[wasm_bindgen_test]
fn to_arabic_reads_kanji() {
    let cases = [
        ("〇", "0"),
        ("二千二十五", "2025"),
        ("二〇二五", "2025"),
        ("一億二千三百四十五万六千七百八十九", "123456789"),
        ("1億2345万6789", "123456789"),
        ("223兆4千億4256万6千", "223400042566000"),
        ("壱萬弐阡参佰肆拾伍", "12345"),
        ("廿一", "21"),
        ("-三千万", "-30000000"),
        ("　一万　", "10000"),
        ("1,234万", "12340000"),
        ("1.5万", "15000"),
        ("一𥝱", "1000000000000000000000000"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            to_arabic(input).as_deref(),
            Ok(expected),
            "input: {input:?}"
        );
    }
}

#[wasm_bindgen_test]
fn to_kanji_every_format() {
    // README の表と同じ
    let cases = [
        (KanjiFormat::Standard, ["一万二千三百四十五", "千万", "〇"]),
        (
            KanjiFormat::ExplicitOneBeforeThousand,
            ["一万二千三百四十五", "一千万", "〇"],
        ),
        (
            KanjiFormat::Positional,
            ["一二三四五", "一〇〇〇〇〇〇〇", "〇"],
        ),
        (KanjiFormat::Daiji, ["壱万弐千参百四拾五", "壱千万", "零"]),
        (
            KanjiFormat::DaijiFull,
            ["壱萬弐阡参佰肆拾伍", "壱阡萬", "零"],
        ),
        (KanjiFormat::Mixed, ["1万2345", "1000万", "0"]),
    ];
    for (format, expected) in cases {
        for (input, expected) in ["12345", "10000000", "0"].into_iter().zip(expected) {
            assert_eq!(
                to_kanji(input, Some(format)).as_deref(),
                Ok(expected),
                "{input} with {format:?}"
            );
        }
    }
}

#[wasm_bindgen_test]
fn to_kanji_accepts_signs_and_separators() {
    assert_eq!(
        to_kanji("-1,000", Some(KanjiFormat::Daiji)).as_deref(),
        Ok("-壱千")
    );
    assert_eq!(
        to_kanji("２０２５", Some(KanjiFormat::Positional)).as_deref(),
        Ok("二〇二五")
    );
    assert_eq!(to_kanji(" +007 ", None).as_deref(), Ok("七"));
}

#[wasm_bindgen_test]
fn to_kanji_defaults_to_standard() {
    for input in ["0", "1100", "10000000", "123456789", "-5"] {
        assert_eq!(
            to_kanji(input, None),
            to_kanji(input, Some(KanjiFormat::Standard)),
            "input: {input:?}"
        );
    }
    assert_eq!(KanjiFormat::default(), KanjiFormat::Standard);
}

#[wasm_bindgen_test]
fn to_arabic_errors() {
    // 位置は UTF-16 のインデックス（JavaScript の文字列の添字）。本体のバイト位置とは違う
    let cases = [
        ("", "Empty", 0),
        ("   ", "Empty", 3),
        ("-", "Empty", 1),
        ("十円", "InvalidCharacter", 1),
        ("　十円", "InvalidCharacter", 2),
        ("百千", "MisplacedUnit", 1),
        ("一億万", "MisplacedUnit", 2),
        ("一万二億", "MisplacedUnit", 3),
        ("十25", "MisplacedDigit", 1),
        ("1億12345万", "MisplacedDigit", 2),
        ("1,,2", "MisplacedSeparator", 2),
        ("千2.5万", "MisplacedSeparator", 2),
        ("1.5", "NotAnInteger", 0),
        ("10000無量大数", "OutOfRange", 0),
        // 𥝱 は UTF-8 で 4 バイトだが、UTF-16 ではサロゲートペアの 2 単位
        ("𥝱円", "InvalidCharacter", 2),
        ("一𥝱一𥝱", "MisplacedUnit", 4),
    ];
    for (input, kind, position) in cases {
        let err = to_arabic(input).expect_err(input);
        assert_parse_error(input, &err, kind, position);
        // メッセージは本体の Display のまま
        let expected = japanese_numeral_normalize::to_arabic(input).unwrap_err();
        assert_eq!(
            String::from(err.message()),
            expected.to_string(),
            "input: {input:?}"
        );
    }
}

#[wasm_bindgen_test]
fn to_kanji_errors() {
    let cases = [
        ("", "Empty", 0),
        ("一万", "InvalidCharacter", 0),
        ("1万", "InvalidCharacter", 1),
        ("1.5", "InvalidCharacter", 1),
        ("１２𥝱", "InvalidCharacter", 2),
        ("1,", "MisplacedSeparator", 1),
    ];
    for (input, kind, position) in cases {
        for format in FORMATS {
            let err = to_kanji(input, Some(format)).expect_err(input);
            assert_parse_error(input, &err, kind, position);
        }
    }
    let too_long = format!("1{}", "0".repeat(72));
    let err = to_kanji(&too_long, None).expect_err("73 digits");
    assert_parse_error(&too_long, &err, "OutOfRange", 0);
}

/// どの形式の漢数字にしても、読み直すと元に戻ること。
fn assert_roundtrip(arabic: &str) {
    for format in FORMATS {
        let kanji = to_kanji(arabic, Some(format))
            .unwrap_or_else(|_| panic!("to_kanji({arabic:?}, {format:?})"));
        assert_eq!(
            to_arabic(&kanji).as_deref(),
            Ok(arabic),
            "{arabic} -> {kanji:?} ({format:?})"
        );
    }
}

#[wasm_bindgen_test]
fn roundtrip_small_numbers() {
    for v in -1000..=20_000_i32 {
        assert_roundtrip(&v.to_string());
    }
}

#[wasm_bindgen_test]
fn roundtrip_large_numbers() {
    // wasm32 でも usize（32 ビット）や f64 を通らず、72 桁まで扱えること
    assert_roundtrip(&u64::MAX.to_string());
    assert_roundtrip(&i128::MIN.to_string());
    assert_roundtrip(&u128::MAX.to_string());
    assert_roundtrip(&"9".repeat(72));
    assert_roundtrip(&format!("-1{}", "0".repeat(71)));
}
