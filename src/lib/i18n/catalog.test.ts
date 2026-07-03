import { describe, it, expect } from "vitest";
import ja from "./ja.json";
import en from "./en.json";
import zh from "./zh.json";
import es from "./es.json";

// #462: バックエンドの安定エラーコード（errors.rust.*）が 4 言語で完全パリティであることを保証する。
// どれか 1 言語にコードを足し忘れると、その言語でだけキーが素通し表示されて露出するため。
const catalogs = { ja, en, zh, es } as Record<string, { errors: { rust: Record<string, string> } }>;

describe("i18n errors.rust カタログ", () => {
  const jaKeys = Object.keys(ja.errors.rust).sort();

  it("60 個以上のコードを持つ", () => {
    expect(jaKeys.length).toBeGreaterThanOrEqual(60);
  });

  it("全コードが E_ 命名規約に従う", () => {
    for (const k of jaKeys) expect(k).toMatch(/^E_[A-Z0-9_]+$/);
  });

  for (const [lang, cat] of Object.entries(catalogs)) {
    it(`${lang} が ja と同一のコード集合を持つ（4言語パリティ）`, () => {
      expect(Object.keys(cat.errors.rust).sort()).toEqual(jaKeys);
    });

    it(`${lang} は空文字テンプレートを含まない`, () => {
      for (const [k, v] of Object.entries(cat.errors.rust)) {
        expect(v.trim(), `${lang}.${k}`).not.toBe("");
      }
    });

    it(`${lang} のプレースホルダは {detail} のみ`, () => {
      for (const [k, v] of Object.entries(cat.errors.rust)) {
        const placeholders = [...v.matchAll(/\{(\w+)\}/g)].map((m) => m[1]);
        for (const p of placeholders) expect(p, `${lang}.${k}`).toBe("detail");
      }
    });
  }
});
