import { describe, it, expect, vi, beforeEach } from "vitest";
import { flushSync } from "svelte";

// invoke をモックしてバックエンド非依存に検証する（#392 抽出モジュール）。
const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invokeMock(...a) }));

import { createVaultView, type EntrySummary } from "./vault-view.svelte";

const t = (key: string) => key; // 素通し翻訳器（メッセージ内容は本テストの関心外）。
const entry = (over: Partial<EntrySummary>): EntrySummary => ({
  path: "/p",
  name: "n",
  created: "2026-07-01",
  kind: "raw",
  tags: [],
  preview: "",
  ...over,
});

describe("createVaultView", () => {
  beforeEach(() => invokeMock.mockReset());

  it("load() 成功で entries を満たし、loading を戻す", async () => {
    const rows = [entry({ path: "/a" }), entry({ path: "/b" })];
    invokeMock.mockResolvedValueOnce(rows);
    const v = createVaultView({ t, onError: () => {} });
    const p = v.load();
    flushSync();
    expect(v.entriesLoading).toBe(true);
    await p;
    flushSync();
    expect(v.entries).toEqual(rows);
    expect(v.entriesLoading).toBe(false);
  });

  it("load() 失敗で onError を呼び entries を空にする", async () => {
    invokeMock.mockRejectedValueOnce("boom");
    const onError = vi.fn();
    const v = createVaultView({ t, onError });
    await v.load();
    flushSync();
    expect(onError).toHaveBeenCalledTimes(1);
    expect(v.entries).toEqual([]);
  });

  it("toggleTag は選択の追加/解除を行い filteredEntries に反映", async () => {
    invokeMock.mockResolvedValueOnce([
      entry({ path: "/a", tags: ["work"] }),
      entry({ path: "/b", tags: ["life"] }),
    ]);
    const v = createVaultView({ t, onError: () => {} });
    await v.load();
    flushSync();
    v.toggleTag("work");
    flushSync();
    expect(v.selectedTags).toEqual(["work"]);
    expect(v.filteredEntries.map((e) => e.path)).toEqual(["/a"]);
    v.toggleTag("work");
    flushSync();
    expect(v.selectedTags).toEqual([]);
    expect(v.filteredEntries).toHaveLength(2);
  });

  it("openEntry は本文を読み viewingEntry を設定する", async () => {
    invokeMock.mockResolvedValueOnce("本文です");
    const v = createVaultView({ t, onError: () => {} });
    await v.openEntry(entry({ path: "/a", name: "2026-07-01" }));
    flushSync();
    expect(v.viewingEntry).toEqual({ name: "2026-07-01", content: "本文です" });
  });

  it("openPanel は viewingEntry を消し一覧を開く", async () => {
    invokeMock.mockResolvedValue([]);
    const v = createVaultView({ t, onError: () => {} });
    v.viewingEntry = { name: "x", content: "y" };
    v.openPanel();
    flushSync();
    expect(v.showEntries).toBe(true);
    expect(v.viewingEntry).toBeNull();
  });
});
