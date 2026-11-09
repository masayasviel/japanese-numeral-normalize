//! 引数を変換して表示するサンプル。
//!
//! アラビア数字はすべての形式の漢数字に、それ以外（漢数字など）はアラビア数字にする。
//!
//! ```sh
//! cargo run --example convert -- 12345 10000000 一億二千万 1.5万
//! ```

use japanese_numeral_normalize::{KanjiFormat, to_arabic, to_kanji};

const FORMATS: [(&str, KanjiFormat); 6] = [
    ("Standard", KanjiFormat::Standard),
    (
        "ExplicitOneBeforeThousand",
        KanjiFormat::ExplicitOneBeforeThousand,
    ),
    ("Positional", KanjiFormat::Positional),
    ("Daiji", KanjiFormat::Daiji),
    ("DaijiFull", KanjiFormat::DaijiFull),
    ("Mixed", KanjiFormat::Mixed),
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: cargo run --example convert -- <数> [<数> ...]");
        std::process::exit(2);
    }

    for arg in args {
        // to_kanji はアラビア数字しか受け付けないので、通ればアラビア数字の入力
        if to_kanji(&arg, KanjiFormat::Standard).is_ok() {
            println!("{arg}");
            for (label, format) in FORMATS {
                // 上で読めることを確かめているので失敗しない
                let kanji = to_kanji(&arg, format).expect("already checked");
                println!("  {label:<26} {kanji}");
            }
        } else {
            match to_arabic(&arg) {
                Ok(arabic) => println!("{arg}\n  {:<26} {arabic}", "Arabic"),
                Err(e) => eprintln!("{arg}: {e}"),
            }
        }
    }
}
