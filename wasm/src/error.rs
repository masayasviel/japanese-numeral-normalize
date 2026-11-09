//! 本体の [`ParseError`](jnn::ParseError) を JavaScript の `Error` にする。

use japanese_numeral_normalize as jnn;
use js_sys::{Error, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

/// 本体の [`ParseError`](jnn::ParseError) を JavaScript の `Error` にする。
///
/// - `message`: 本体のエラーメッセージ（`Display`。位置はバイト位置で書かれる）
/// - `name`: `"ParseError"`
/// - `kind`: 原因（[`ParseErrorKind`](jnn::ParseErrorKind) のバリアント名）
/// - `position`: エラーの位置。JavaScript の文字列と同じ UTF-16 のインデックス
pub(crate) fn to_js(input: &str, err: &jnn::ParseError) -> Error {
    let js_err = Error::new(&err.to_string());
    js_err.set_name("ParseError");
    let kind = JsValue::from_str(kind_name(err.kind()));
    let position = JsValue::from(utf16_position(input, err.position()));
    // 作ったばかりの普通のオブジェクトへの代入なので失敗しない
    Reflect::set(&js_err, &JsValue::from_str("kind"), &kind).expect("set kind");
    Reflect::set(&js_err, &JsValue::from_str("position"), &position).expect("set position");
    js_err
}

/// [`ParseErrorKind`](jnn::ParseErrorKind) のバリアント名。
fn kind_name(kind: &jnn::ParseErrorKind) -> &'static str {
    use jnn::ParseErrorKind as Kind;
    match kind {
        Kind::Empty => "Empty",
        Kind::InvalidCharacter(_) => "InvalidCharacter",
        Kind::MisplacedUnit => "MisplacedUnit",
        Kind::MisplacedDigit => "MisplacedDigit",
        Kind::MisplacedSeparator => "MisplacedSeparator",
        Kind::NotAnInteger => "NotAnInteger",
        Kind::OutOfRange => "OutOfRange",
        // ParseErrorKind は #[non_exhaustive] なので、別クレートからは `_` が必須。
        // 本体に種類を増やしたら、ここと下の TypeScript の型にも足す
        _ => "Unknown",
    }
}

/// UTF-8 のバイト位置を、UTF-16 のコード単位でのインデックスにする。
///
/// 本体の位置はバイトオフセット（日本語は多くが 1 文字 3 バイト、𥝱 は 4 バイト）だが、
/// JavaScript の文字列は UTF-16 なので、`input.slice(position)` で使える値にそろえる。
/// 例: `"𥝱円"` の「円」はバイト位置 4、UTF-16 では 2（𥝱 はサロゲートペアで 2 単位）。
fn utf16_position(input: &str, byte_position: usize) -> usize {
    input
        .char_indices()
        .take_while(|&(i, _)| i < byte_position)
        .map(|(_, c)| c.len_utf16())
        .sum()
}

// 投げる Error の形を、生成される TypeScript の型定義（.d.ts）に足す。
#[wasm_bindgen(typescript_custom_section)]
const PARSE_ERROR_TS: &str = r#"
/**
 * `toArabic` / `toKanji` が、数として読めない入力に対して投げるエラー。
 *
 * ```ts
 * try {
 *   toArabic("十円");
 * } catch (e) {
 *   const err = e as ParseError;
 *   err.kind;     // "InvalidCharacter"
 *   err.position; // 1
 * }
 * ```
 */
export interface ParseError extends Error {
  name: "ParseError";
  /** エラーの原因。 */
  kind: ParseErrorKind;
  /**
   * エラーが見つかった位置。入力文字列の UTF-16 のインデックスなので、
   * `input.slice(position)` がエラーの見つかった文字から始まる。
   */
  position: number;
}

/**
 * エラーの原因。
 *
 * - `Empty`: 入力が空、または符号だけで数字がない
 * - `InvalidCharacter`: 受け付けない文字がある（例: `"十円"` の `円`）
 * - `MisplacedUnit`: 単位の順序や位置が正しくない（例: `"百千"`, `"一億万"`）
 * - `MisplacedDigit`: 数字列がその位に収まらない（例: `"12百"`, `"1万20000"`）
 * - `MisplacedSeparator`: 桁区切り `,` や小数点 `.` の位置が正しくない（例: `",1"`, `"1.5千"`）
 * - `NotAnInteger`: 小数を含み、整数にならない（例: `"1.5"`）
 * - `OutOfRange`: 表せる範囲（絶対値が 10^72 未満）を超えている
 * - `Unknown`: このバインディングがまだ対応していない原因（ライブラリ本体にだけ追加されたもの）
 */
export type ParseErrorKind =
  | "Empty"
  | "InvalidCharacter"
  | "MisplacedUnit"
  | "MisplacedDigit"
  | "MisplacedSeparator"
  | "NotAnInteger"
  | "OutOfRange"
  | "Unknown";
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_positions() {
        // ASCII・BMP の漢字は UTF-16 で 1 単位
        assert_eq!(utf16_position("12a", 2), 2);
        assert_eq!(utf16_position("十円", "十".len()), 1);
        assert_eq!(utf16_position("一億万", "一億".len()), 2);
        // 𥝱（U+25771）は UTF-8 で 4 バイト、UTF-16 でサロゲートペアの 2 単位
        assert_eq!(utf16_position("𥝱円", "𥝱".len()), 2);
        assert_eq!(utf16_position("1𥝱2𥝱円", "1𥝱2𥝱".len()), 6);
        // 入力の末尾（Empty など）は文字列全体の長さ
        assert_eq!(utf16_position("  ", 2), 2);
        assert_eq!(utf16_position("", 0), 0);
    }

    #[test]
    fn kind_names() {
        use jnn::ParseErrorKind as Kind;
        assert_eq!(kind_name(&Kind::Empty), "Empty");
        assert_eq!(kind_name(&Kind::InvalidCharacter('円')), "InvalidCharacter");
        assert_eq!(kind_name(&Kind::MisplacedUnit), "MisplacedUnit");
        assert_eq!(kind_name(&Kind::MisplacedDigit), "MisplacedDigit");
        assert_eq!(kind_name(&Kind::MisplacedSeparator), "MisplacedSeparator");
        assert_eq!(kind_name(&Kind::NotAnInteger), "NotAnInteger");
        assert_eq!(kind_name(&Kind::OutOfRange), "OutOfRange");
    }
}
