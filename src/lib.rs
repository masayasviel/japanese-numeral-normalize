//! アラビア数字 ⇔ 漢数字の相互変換ライブラリ。
//!
//! 設計は [`docs/design.md`](https://github.com/masayasviel/japanese-numeral-normalize/blob/main/docs/design.md) を参照。

// 標準ライブラリ（OS 依存の機能）を使わず、`core` と `alloc`（String / Vec）だけで書く。
// こうしておくと組み込みや WASM でもそのまま使える。
#![no_std]

extern crate alloc;
