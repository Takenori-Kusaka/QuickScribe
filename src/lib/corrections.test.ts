import { describe, it, expect } from "vitest";
import { parseCorrections, applyCorrections } from "./corrections";

describe("parseCorrections", () => {
  it("区切り行を候補にパースする", () => {
    const raw = "レーメン ||| LLM ||| 文脈は大規模言語モデルの話\nテンソル ||| tensor ||| 専門用語";
    const c = parseCorrections(raw);
    expect(c).toHaveLength(2);
    expect(c[0]).toMatchObject({ original: "レーメン", suggestion: "LLM", replace: true });
    expect(c[1].suggestion).toBe("tensor");
  });

  it("理由なし(2要素)も許容、原文=提案や空行は除外", () => {
    const raw = "あ ||| い\n \nx ||| x ||| 同一は除外\n空だけ";
    const c = parseCorrections(raw);
    expect(c).toHaveLength(1);
    expect(c[0]).toMatchObject({ original: "あ", suggestion: "い", reason: "" });
  });
});

describe("applyCorrections", () => {
  it("replace=trueのみ全置換し件数を返す", () => {
    const corr = [
      { original: "レーメン", suggestion: "LLM", reason: "", replace: true },
      { original: "猫", suggestion: "犬", reason: "", replace: false },
    ];
    const { text, applied } = applyCorrections("レーメンとレーメン、猫", corr);
    expect(text).toBe("LLMとLLM、猫"); // レーメンは全置換、猫は置換しない
    expect(applied).toBe(1);
  });

  it("空提案は適用しない", () => {
    const { text, applied } = applyCorrections("x", [
      { original: "x", suggestion: "  ", reason: "", replace: true },
    ]);
    expect(text).toBe("x");
    expect(applied).toBe(0);
  });
});
