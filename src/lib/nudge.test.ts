import { describe, it, expect } from "vitest";
import { maybeNudge, type NudgeDeps } from "./nudge";

// 副作用依存をスタブし、maybeNudge のゲーティング/重複防止を検証する。
function deps(over: Partial<NudgeDeps> = {}): NudgeDeps & { sent: string[]; last: { v: string | null } } {
  const state = { sent: [] as string[], last: { v: null as string | null } };
  return {
    notify: (t, b) => {
      state.sent.push(`${t}|${b}`);
    },
    isGranted: async () => true,
    getLastNudge: () => state.last.v,
    setLastNudge: (d) => {
      state.last.v = d;
    },
    ...over,
    ...state,
  };
}

const base = {
  enabled: true,
  dates: ["2026-07-01"], // 昨日記録・今日未記録＝継続中
  today: "2026-07-02",
  title: "T",
  body: "B",
};

describe("maybeNudge（習慣ナッジ発火 #58）", () => {
  it("条件を満たすと通知し、当日を記録する", async () => {
    const d = deps();
    const fired = await maybeNudge(base, d);
    expect(fired).toBe(true);
    expect(d.sent).toEqual(["T|B"]);
    expect(d.last.v).toBe("2026-07-02");
  });

  it("opt-in OFF なら何もしない", async () => {
    const d = deps();
    expect(await maybeNudge({ ...base, enabled: false }, d)).toBe(false);
    expect(d.sent).toEqual([]);
  });

  it("今日すでにナッジ済みなら鳴らさない（1日1回）", async () => {
    const d = deps({ getLastNudge: () => "2026-07-02" });
    expect(await maybeNudge(base, d)).toBe(false);
    expect(d.sent).toEqual([]);
  });

  it("今日すでに記録済みなら鳴らさない（shouldNudge=false）", async () => {
    const d = deps();
    expect(await maybeNudge({ ...base, dates: ["2026-07-02"] }, d)).toBe(false);
    expect(d.sent).toEqual([]);
  });

  it("通知権限が無ければ鳴らさない", async () => {
    const d = deps({ isGranted: async () => false });
    expect(await maybeNudge(base, d)).toBe(false);
    expect(d.sent).toEqual([]);
    expect(d.last.v).toBe(null); // 未記録のまま（次回権限付与後に再挑戦できる）
  });
});
