//! 引数の数を読み取り、すべての形式で表示するサンプル。
//!
//! ```sh
//! cargo run --example convert -- 一億二千万 1.5万 -42 二〇二五
//! ```

use japanese_numeral_normalize::{KanjiFormat, KanjiOptions, Numeral};

const FORMATS: [(&str, KanjiFormat); 5] = [
    ("Standard", KanjiFormat::Standard),
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

    // 千の前にも「一」を書くオプション（`一千万`）
    let options = KanjiOptions {
        explicit_one_before_thousand: true,
        ..KanjiOptions::default()
    };

    for arg in args {
        match arg.parse::<Numeral>() {
            Ok(n) => {
                println!("{arg}");
                println!("  {:<10} {n}", "Arabic");
                for (label, format) in FORMATS {
                    println!("  {label:<10} {}", n.to_kanji_with(format));
                }
                println!("  {:<10} {}", "Standard*", n.to_kanji_with(options));
            }
            Err(e) => eprintln!("{arg}: {e}"),
        }
    }
}
