//! アラビア数字 ⇔ 漢数字の相互変換ライブラリ。
//!
//! 設計は [`docs/design.md`](https://github.com/masayasviel/japanese-numeral-normalize/blob/main/docs/design.md) を参照。

// 標準ライブラリ（OS 依存の機能）を使わず、`core` と `alloc`（String / Vec）だけで書く。
// こうしておくと組み込みや WASM でもそのまま使える。
#![no_std]

extern crate alloc;

mod error;
// 公開 API から使うまでは「使われていない」という警告が出るので、一時的に抑える。
// allow ではなく expect にしておくと、使い始めて警告が出なくなったときに消し忘れを教えてくれる。
// テストからは使っているので、テスト以外のビルドだけに付ける。
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "to_kanji（#14）と to_arabic（#15）から使う")
)]
mod numeral;
#[cfg_attr(not(test), expect(dead_code, reason = "to_kanji（#14）から使う"))]
mod units;

pub use error::{ParseError, ParseErrorKind};
