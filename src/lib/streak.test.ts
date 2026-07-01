import { describe, it, expect } from "vitest";
import { computeStreak } from "./streak";

describe("computeStreak（寛容ストリーク）", () => {
  it("記録なしは0", () => {
    expect(computeStreak([], "2026-07-02")).toBe(0);
  });

  it("今日を含む連続3日は3", () => {
    expect(computeStreak(["2026-06-30", "2026-07-01", "2026-07-02"], "2026-07-02")).toBe(3);
  });

  it("昨日までの連続でも継続中(今日未記録でもgrace内)", () => {
    expect(computeStreak(["2026-06-30", "2026-07-01"], "2026-07-02")).toBe(2);
  });

  it("1日サボりは許容(forgiving): 月・水は途切れず2", () => {
    // 6/30(月)・7/2(水)、7/1を飛ばしても継続。
    expect(computeStreak(["2026-06-30", "2026-07-02"], "2026-07-02")).toBe(2);
  });

  it("2日以上の空白で途切れる", () => {
    // 6/28 と 7/2 は4日空き → 継続は今日ぶんの1のみ。
    expect(computeStreak(["2026-06-28", "2026-07-02"], "2026-07-02")).toBe(1);
  });

  it("最新記録が古すぎる(grace超)なら0", () => {
    expect(computeStreak(["2026-06-20"], "2026-07-02")).toBe(0);
  });

  it("重複日は二重計上しない", () => {
    expect(computeStreak(["2026-07-02", "2026-07-02", "2026-07-01"], "2026-07-02")).toBe(2);
  });

  it("ISO日時でも日付部分で判定する", () => {
    expect(
      computeStreak(["2026-07-01T23:10:00", "2026-07-02T08:00:00"], "2026-07-02T09:00:00"),
    ).toBe(2);
  });
});
