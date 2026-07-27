import { describe, it, expect } from "vitest";
import { kindLabel, parseTags, filterEntries, visibleEntries, hiddenEntryCount } from "./entry";

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

describe("visibleEntries / hiddenEntryCount", () => {
  const rows = Array.from({ length: 5 }, (_, i) => E(`n${i}`, "", []));

  it("showAll=false なら先頭 limit 件だけを返す", () => {
    expect(visibleEntries(rows, 2, false).map((e) => e.name)).toEqual(["n0", "n1"]);
  });

  it("並び順を変えない（一覧は既に新しい順で渡される）", () => {
    expect(visibleEntries(rows, 5, false).map((e) => e.name)).toEqual([
      "n0",
      "n1",
      "n2",
      "n3",
      "n4",
    ]);
  });

  it("showAll=true なら limit を無視して全件返す", () => {
    expect(visibleEntries(rows, 2, true)).toHaveLength(5);
  });

  it("limit が件数以上なら全件返す（切り詰めない）", () => {
    expect(visibleEntries(rows, 99, false)).toHaveLength(5);
  });

  it("limit<=0 は 0 件扱い", () => {
    expect(visibleEntries(rows, 0, false)).toEqual([]);
    expect(visibleEntries(rows, -1, false)).toEqual([]);
  });

  it("入力配列を破壊しない", () => {
    const src = [...rows];
    visibleEntries(src, 2, false);
    expect(src.map((e) => e.name)).toEqual(rows.map((e) => e.name));
  });

  it("hiddenEntryCount は隠れている件数を返す", () => {
    expect(hiddenEntryCount(rows, 2)).toBe(3);
    expect(hiddenEntryCount(rows, 5)).toBe(0);
    expect(hiddenEntryCount(rows, 99)).toBe(0);
    expect(hiddenEntryCount(rows, -1)).toBe(5);
  });
});
