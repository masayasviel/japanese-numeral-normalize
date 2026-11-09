// wasm-bindgen が生成した JavaScript の API（wasm/pkg）を、利用者と同じように Node.js から呼ぶテスト。
//
// Rust 側のテスト（tests/wasm.rs）では確かめられない、JavaScript から見た形を検査する:
// 関数名（toArabic / toKanji）、引数の省略、投げられる Error のプロパティ、UTF-16 の位置、型定義（.d.ts）。
//
// 先に `make build-wasm` で wasm/pkg を作っておく（`make test-wasm` は両方まとめて行う）。

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  initSync,
  KanjiFormat,
  toArabic,
  toKanji,
} from "../../pkg/japanese_numeral_normalize.js";

const pkg = new URL("../../pkg/", import.meta.url);

// ブラウザ向け（--target web）の出力を、fetch を使わずにバイト列から初期化する
initSync({ module: readFileSync(new URL("japanese_numeral_normalize_bg.wasm", pkg)) });

/** `fn` が ParseError（`name`・`kind`・`position` を持つ Error）を投げること。 */
function assertParseError(fn, kind, position) {
  assert.throws(fn, (err) => {
    assert.ok(err instanceof Error);
    assert.equal(err.name, "ParseError");
    assert.equal(err.kind, kind);
    assert.equal(err.position, position);
    assert.equal(typeof err.message, "string");
    return true;
  });
}

test("toArabic: 漢数字・混在・大字をアラビア数字にする", () => {
  assert.equal(toArabic("二千二十五"), "2025");
  assert.equal(toArabic("二〇二五"), "2025");
  assert.equal(toArabic("1億2345万6789"), "123456789");
  assert.equal(toArabic("壱萬弐阡参佰肆拾伍"), "12345");
  assert.equal(toArabic("1.5万"), "15000");
  assert.equal(toArabic("-三千万"), "-30000000");
});

test("toKanji: KanjiFormat で書き方を選ぶ", () => {
  const cases = [
    [KanjiFormat.Standard, "一万二千三百四十五", "千万"],
    [KanjiFormat.ExplicitOneBeforeThousand, "一万二千三百四十五", "一千万"],
    [KanjiFormat.Positional, "一二三四五", "一〇〇〇〇〇〇〇"],
    [KanjiFormat.Daiji, "壱万弐千参百四拾五", "壱千万"],
    [KanjiFormat.DaijiFull, "壱萬弐阡参佰肆拾伍", "壱阡萬"],
    [KanjiFormat.Mixed, "1万2345", "1000万"],
  ];
  for (const [format, a, b] of cases) {
    assert.equal(toKanji("12345", format), a, KanjiFormat[format]);
    assert.equal(toKanji("10000000", format), b, KanjiFormat[format]);
  }
});

test("toKanji: format を省略すると Standard", () => {
  assert.equal(toKanji("10000000"), "千万");
  assert.equal(toKanji("10000000", undefined), "千万");
  assert.equal(toKanji("10000000", null), "千万");
});

test("toKanji: KanjiFormat にない値は例外（ParseError ではない）", () => {
  for (const format of [99, -1, 3.5, "Daiji"]) {
    assert.throws(
      () => toKanji("1", format),
      (err) => err instanceof Error && err.name !== "ParseError",
      String(format),
    );
  }
  // 例外の後も使い続けられる
  assert.equal(toKanji("1", KanjiFormat.Daiji), "壱");
});

test("KanjiFormat: Rust の enum と同じ名前と値", () => {
  assert.deepEqual(
    Object.entries(KanjiFormat).filter(([key]) => Number.isNaN(Number(key))),
    [
      ["Standard", 0],
      ["ExplicitOneBeforeThousand", 1],
      ["Positional", 2],
      ["Daiji", 3],
      ["DaijiFull", 4],
      ["Mixed", 5],
    ],
  );
  assert.ok(Object.isFrozen(KanjiFormat));
});

test("エラー: kind と position（UTF-16 のインデックス）を持つ Error を投げる", () => {
  assertParseError(() => toArabic(""), "Empty", 0);
  assertParseError(() => toArabic("十円"), "InvalidCharacter", 1);
  assertParseError(() => toArabic("一億万"), "MisplacedUnit", 2);
  assertParseError(() => toArabic("1万20000"), "MisplacedDigit", 2);
  assertParseError(() => toArabic("1.5千"), "MisplacedSeparator", 1);
  assertParseError(() => toArabic("1.5"), "NotAnInteger", 0);
  assertParseError(() => toArabic("1" + "0".repeat(72)), "OutOfRange", 0);
  assertParseError(() => toKanji("一万"), "InvalidCharacter", 0);
  assertParseError(() => toKanji("1,", KanjiFormat.Daiji), "MisplacedSeparator", 1);
});

test("エラー: position は input.slice() にそのまま使える", () => {
  // [関数, 入力, エラーの位置から始まる残り]。𥝱 はサロゲートペア（JavaScript の文字列で長さ 2）
  const cases = [
    [toArabic, "十円", "円"],
    [toArabic, "　十円", "円"],
    [toArabic, "𥝱円", "円"],
    [toArabic, "一𥝱二𥝱円", "𥝱円"],
    [toArabic, "𥝱 一", " 一"],
    [toKanji, "１２𥝱", "𥝱"],
    // 符号のあとに数字がないまま終わった場合は、入力の末尾（残りは空）
    [toArabic, "  -", ""],
  ];
  for (const [fn, input, rest] of cases) {
    assert.throws(
      () => fn(input),
      (err) => input.slice(err.position) === rest,
      `${fn.name}(${JSON.stringify(input)})`,
    );
  }
});

test("エラー: 孤立したサロゲートは U+FFFD として扱い、位置はずれない", () => {
  assertParseError(() => toArabic("1\uD800"), "InvalidCharacter", 1);
  assertParseError(() => toArabic("二\uDC00三"), "InvalidCharacter", 1);
});

test("ラウンドトリップ: どの形式の漢数字にしても、読み直すと元に戻る", () => {
  const formats = Object.values(KanjiFormat).filter((v) => typeof v === "number");
  const numbers = [
    "0",
    "-1",
    "10",
    "1100",
    "2025",
    "100000001",
    String(Number.MAX_SAFE_INTEGER),
    // Number や BigInt を通らないので、72 桁まで文字列のまま扱える
    (2n ** 128n).toString(),
    "9".repeat(72),
  ];
  for (const n of numbers) {
    for (const format of formats) {
      assert.equal(toArabic(toKanji(n, format)), n, `${n} (${KanjiFormat[format]})`);
    }
  }
});

test("型定義: ParseError と ParseErrorKind が .d.ts に入っている", () => {
  const dts = readFileSync(new URL("japanese_numeral_normalize.d.ts", pkg), "utf8");
  assert.match(dts, /export function toArabic\(input: string\): string;/);
  assert.match(dts, /export function toKanji\(input: string, format\?: KanjiFormat \| null\): string;/);
  assert.match(dts, /export enum KanjiFormat \{/);
  assert.match(dts, /export interface ParseError extends Error \{/);
  for (const kind of [
    "Empty",
    "InvalidCharacter",
    "MisplacedUnit",
    "MisplacedDigit",
    "MisplacedSeparator",
    "NotAnInteger",
    "OutOfRange",
  ]) {
    assert.ok(dts.includes(`| "${kind}"`), kind);
  }
});
