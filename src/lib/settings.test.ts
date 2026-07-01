import { describe, it, expect } from "vitest";
import { clampProvider, clampSttProvider, clampOneOf, isValidRefineStyle } from "./settings";

describe("clampProvider", () => {
  it("既知プロバイダはそのまま", () => {
    expect(clampProvider("anthropic")).toBe("anthropic");
    expect(clampProvider("ollama")).toBe("ollama");
  });
  it("未知/空は既定 gemini へ", () => {
    expect(clampProvider("unknown")).toBe("gemini");
    expect(clampProvider("")).toBe("gemini");
  });
});

describe("clampSttProvider", () => {
  it("既知はそのまま、未知は local", () => {
    expect(clampSttProvider("openai")).toBe("openai");
    expect(clampSttProvider("bogus")).toBe("local");
  });
});

describe("clampOneOf", () => {
  it("許容値はそのまま、外れは既定", () => {
    expect(clampOneOf("md", ["txt", "md"] as const, "txt")).toBe("md");
    expect(clampOneOf("xml", ["txt", "md"] as const, "txt")).toBe("txt");
    expect(clampOneOf("momentary", ["toggle", "momentary"] as const, "toggle")).toBe("momentary");
    expect(clampOneOf("", ["opus", "wav"] as const, "opus")).toBe("opus");
  });
});

describe("isValidRefineStyle", () => {
  it("組み込みスタイルは有効", () => {
    expect(isValidRefineStyle("structured", [])).toBe(true);
    expect(isValidRefineStyle("verbatim", [])).toBe(true);
  });
  it("存在するカスタムは有効、存在しないカスタムは無効", () => {
    const custom = [{ id: "x", label: "X", instruction: "..." }];
    expect(isValidRefineStyle("custom:x", custom)).toBe(true);
    expect(isValidRefineStyle("custom:none", custom)).toBe(false);
    expect(isValidRefineStyle("custom:x", [])).toBe(false);
  });
  it("未知の値は無効", () => {
    expect(isValidRefineStyle("bogus", [])).toBe(false);
  });
});
