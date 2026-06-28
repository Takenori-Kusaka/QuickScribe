import { describe, it, expect } from "vitest";
import { kindLabel, parseTags } from "./entry";

describe("kindLabel", () => {
  it("既知の種別を日本語に", () => {
    expect(kindLabel("transcript")).toBe("文字起こし");
    expect(kindLabel("refined")).toBe("整形済み");
    expect(kindLabel("note")).toBe("メモ");
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
