import { describe, it, expect } from "vitest";
import { kindLabel, parseTags } from "./entry";

describe("kindLabel", () => {
  it("既知の種別を i18n キーに", () => {
    expect(kindLabel("transcript")).toBe("results.kind_transcript");
    expect(kindLabel("refined")).toBe("results.kind_refined");
    expect(kindLabel("note")).toBe("results.kind_note");
  });
  it("未知はそのまま", () => {
    expect(kindLabel("other")).toBe("other");
  });
});

describe("parseTags", () => {
  it("カンマ/全角カンマ/空白で分割し前後空白を除去", () => {
    expect(parseTags("仕事, 不安　アイデア、開発")).toEqual(["仕事", "不安", "アイデア", "開発"]);
  });
  it("先頭の # を除去", () => {
    expect(parseTags("#tag1 ##tag2")).toEqual(["tag1", "tag2"]);
  });
  it("重複と空を除く", () => {
    expect(parseTags("a, a, , b")).toEqual(["a", "b"]);
  });
  it("空文字列は空配列", () => {
    expect(parseTags("   ")).toEqual([]);
  });
});
