import { describe, it, expect } from "vitest";
import { kindLabel, parseTags, filterEntries } from "./entry";

const E = (name: string, preview: string, tags: string[]) => ({ name, preview, tags });
const sample = [
  E("transcript-1.md", "会議の議事メモ", ["仕事", "会議"]),
  E("refined-2.md", "旅行の振り返り", ["旅行"]),
  E("note-3.txt", "買い物リスト", []),
];

describe("filterEntries", () => {
  it("空クエリ・タグ無しは全件", () => {
    expect(filterEntries(sample, "").length).toBe(3);
    expect(filterEntries(sample, "   ").length).toBe(3);
  });
  it("本文/ファイル名/タグの部分一致(大小無視)", () => {
    expect(filterEntries(sample, "議事").map((e) => e.name)).toEqual(["transcript-1.md"]);
    expect(filterEntries(sample, "REFINED").map((e) => e.name)).toEqual(["refined-2.md"]);
    expect(filterEntries(sample, "旅行").map((e) => e.name)).toEqual(["refined-2.md"]);
  });
  it("選択タグはAND(全て含むもののみ)", () => {
    expect(filterEntries(sample, "", ["仕事"]).map((e) => e.name)).toEqual(["transcript-1.md"]);
    expect(filterEntries(sample, "", ["仕事", "会議"]).length).toBe(1);
    expect(filterEntries(sample, "", ["仕事", "旅行"]).length).toBe(0);
  });
  it("タグとクエリの併用", () => {
    expect(filterEntries(sample, "議事", ["会議"]).length).toBe(1);
    expect(filterEntries(sample, "旅行", ["会議"]).length).toBe(0);
  });
  it("一致なしは空", () => {
    expect(filterEntries(sample, "存在しない語XYZ").length).toBe(0);
  });
});

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
