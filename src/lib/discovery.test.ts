import { describe, it, expect } from "vitest";
import { selectDiscoveryTargets, buildDiscoveryText, type DiscoveryItem } from "./discovery";

describe("selectDiscoveryTargets", () => {
  it("max 件で切り出し、超過を truncated で示す", () => {
    const r = selectDiscoveryTargets([1, 2, 3, 4], 2);
    expect(r.targets).toEqual([1, 2]);
    expect(r.truncated).toBe(true);
  });
  it("max 以下なら truncated は false", () => {
    const r = selectDiscoveryTargets([1, 2], 5);
    expect(r.targets).toEqual([1, 2]);
    expect(r.truncated).toBe(false);
  });
});

describe("buildDiscoveryText", () => {
  it("見出し＋本文を --- で連結、タグは #付き", () => {
    const items: DiscoveryItem[] = [
      { created: "2026-06-27 10:00", tags: ["仕事", "不安"], content: "本文1" },
      { created: "2026-06-28 11:00", tags: [], content: "本文2" },
    ];
    expect(buildDiscoveryText(items)).toBe(
      "### 2026-06-27 10:00 #仕事 #不安\n本文1\n\n---\n\n### 2026-06-28 11:00 \n本文2",
    );
  });
  it("空配列は空文字列", () => {
    expect(buildDiscoveryText([])).toBe("");
  });
});
