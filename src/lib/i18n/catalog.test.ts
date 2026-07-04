import { describe, it, expect } from "vitest";
// Rust 側 SSOT(errcode.rs)を Vite の raw import で読み、コード集合を機械的に突合する。
import errcodeSrc from "../../../src-tauri/src/errcode.rs?raw";
import ja from "./ja.json";
import en from "./en.json";
import zh from "./zh.json";
import es from "./es.json";

// #462/#401: 翻訳カタログの構造的整合を機械検証する。
// 1) 全カタログ(287+キー)の再帰的キー集合が 4 言語で完全パリティ（足し忘れ検出）。
// 2) 各キーのプレースホルダ集合({n}等)が ja と一致（補間切れ検出）。
// 3) errors.rust.* / status.rust.* は Rust 側 SSOT(errcode.rs) と機械的に突合。
const catalogs: Record<string, unknown> = { ja, en, zh, es };

/** ネストした JSON をドット区切りキーへ平坦化する。 */
function flatten(obj: unknown, prefix = ""): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(obj as Record<string, unknown>)) {
    const key = prefix ? `${prefix}.${k}` : k;
    if (typeof v === "string") out[key] = v;
    else Object.assign(out, flatten(v, key));
  }
  return out;
}

/** 文言中の ICU プレースホルダ名({x})をソート済み配列で返す。 */
function placeholders(msg: string): string[] {
  return [...msg.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
}

/** errcode.rs から指定プレフィックスの安定コード定数を抽出する（Rust側SSOTとの突合用）。 */
function rustCodes(prefix: string): string[] {
  const codes = [...errcodeSrc.matchAll(/pub const ([A-Z0-9_]+): &str = "([A-Z0-9_]+)";/g)]
    .map((m) => m[2])
    .filter((c) => c.startsWith(prefix));
  return [...new Set(codes)].sort();
}

const flat = Object.fromEntries(
  Object.entries(catalogs).map(([lang, cat]) => [lang, flatten(cat)]),
) as Record<string, Record<string, string>>;
const jaKeys = Object.keys(flat.ja).sort();

describe("i18n カタログ全体のパリティ（4言語）", () => {
  it("十分な規模のカタログである（287キー以上）", () => {
    expect(jaKeys.length).toBeGreaterThanOrEqual(287);
  });

  for (const lang of ["en", "zh", "es"]) {
    it(`${lang} が ja と同一のキー集合を持つ`, () => {
      expect(Object.keys(flat[lang]).sort()).toEqual(jaKeys);
    });

    it(`${lang} の各キーのプレースホルダが ja と一致する`, () => {
      for (const key of jaKeys) {
        expect(placeholders(flat[lang][key]), `${lang}.${key}`).toEqual(placeholders(flat.ja[key]));
      }
    });
  }
});

describe("i18n errors.rust カタログ（Rust SSOT との突合）", () => {
  const jaErrKeys = Object.keys(
    (ja as { errors: { rust: Record<string, string> } }).errors.rust,
  ).sort();

  it("60 個以上のコードを持つ", () => {
    expect(jaErrKeys.length).toBeGreaterThanOrEqual(60);
  });

  it("全コードが E_ 命名規約に従う", () => {
    for (const k of jaErrKeys) expect(k).toMatch(/^E_[A-Z0-9_]+$/);
  });

  it("errcode.rs の E_ コード集合と完全一致する（片側だけの追加を検出）", () => {
    expect(jaErrKeys).toEqual(rustCodes("E_"));
  });

  for (const [lang, cat] of Object.entries(
    catalogs as Record<string, { errors: { rust: Record<string, string> } }>,
  )) {
    it(`${lang} は空文字テンプレートを含まない`, () => {
      for (const [k, v] of Object.entries(cat.errors.rust)) {
        expect(v.trim(), `${lang}.${k}`).not.toBe("");
      }
    });

    it(`${lang} のプレースホルダは {detail} のみ`, () => {
      for (const [k, v] of Object.entries(cat.errors.rust)) {
        for (const p of placeholders(v)) expect(p, `${lang}.${k}`).toBe("detail");
      }
    });
  }
});

describe("i18n status.rust カタログ（Rust SSOT との突合）", () => {
  const jaStatusKeys = Object.keys(
    (ja as { status: { rust: Record<string, string> } }).status.rust,
  ).sort();

  it("全コードが S_ 命名規約に従う", () => {
    for (const k of jaStatusKeys) expect(k).toMatch(/^S_[A-Z0-9_]+$/);
  });

  it("errcode.rs の S_ コード集合と完全一致する", () => {
    expect(jaStatusKeys).toEqual(rustCodes("S_"));
  });
});
