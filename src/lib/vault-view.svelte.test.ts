import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { flushSync } from "svelte";

// invoke をモックしてバックエンド非依存に検証する（#392 抽出モジュール）。
const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invokeMock(...a) }));

import { createVaultView, type EntrySummary } from "./vault-view.svelte";
import { ENTRY_SEARCH_DEBOUNCE_MS, ENTRY_VISIBLE } from "./constants";

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

describe("createVaultView 検索デバウンス（#666）", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    vi.useFakeTimers();
  });
  afterEach(() => vi.useRealTimers());

  const loaded = async () => {
    invokeMock.mockResolvedValueOnce([
      entry({ path: "/a", name: "alpha" }),
      entry({ path: "/b", name: "beta" }),
    ]);
    const v = createVaultView({ t, onError: () => {} });
    await v.load();
    flushSync();
    return v;
  };

  it("打鍵直後は filteredEntries を再計算しない（デバウンス前）", async () => {
    const v = await loaded();
    v.entrySearch = "alpha";
    flushSync();
    expect(v.entrySearch).toBe("alpha"); // 入力欄の表示は即時
    expect(v.filteredEntries).toHaveLength(2); // 絞り込みはまだ走っていない
  });

  it("デバウンス時間の経過後に絞り込みが反映される", async () => {
    const v = await loaded();
    v.entrySearch = "alpha";
    vi.advanceTimersByTime(ENTRY_SEARCH_DEBOUNCE_MS);
    flushSync();
    expect(v.filteredEntries.map((e) => e.path)).toEqual(["/a"]);
  });

  it("連打しても最後の1回だけが適用される（中間の打鍵で再計算しない）", async () => {
    const v = await loaded();
    v.entrySearch = "a";
    vi.advanceTimersByTime(ENTRY_SEARCH_DEBOUNCE_MS - 1);
    v.entrySearch = "al";
    vi.advanceTimersByTime(ENTRY_SEARCH_DEBOUNCE_MS - 1);
    v.entrySearch = "beta";
    flushSync();
    expect(v.filteredEntries).toHaveLength(2); // ここまで一度も適用されない
    vi.advanceTimersByTime(ENTRY_SEARCH_DEBOUNCE_MS);
    flushSync();
    expect(v.filteredEntries.map((e) => e.path)).toEqual(["/b"]);
  });

  it("dispose() 後は保留中のデバウンスが発火しない", async () => {
    const v = await loaded();
    v.entrySearch = "alpha";
    v.dispose();
    vi.advanceTimersByTime(ENTRY_SEARCH_DEBOUNCE_MS * 10);
    flushSync();
    expect(v.filteredEntries).toHaveLength(2);
  });
});

describe("createVaultView 段階表示（#666）", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    vi.useFakeTimers();
  });
  afterEach(() => vi.useRealTimers());

  const many = (n: number) =>
    Array.from({ length: n }, (_, i) => entry({ path: `/p${i}`, name: `n${i}` }));

  const loadedMany = async (n: number) => {
    invokeMock.mockResolvedValueOnce(many(n));
    const v = createVaultView({ t, onError: () => {} });
    await v.load();
    flushSync();
    return v;
  };

  it("既定では ENTRY_VISIBLE 件までしか描画対象にしない", async () => {
    const v = await loadedMany(ENTRY_VISIBLE + 10);
    expect(v.filteredEntries).toHaveLength(ENTRY_VISIBLE + 10); // 絞り込み結果は全件
    expect(v.visibleEntries).toHaveLength(ENTRY_VISIBLE); // 描画は上限まで
    expect(v.hiddenEntryCount).toBe(10);
  });

  it("showAllEntries=true で全件に到達できる（全件へのアクセスを失わない）", async () => {
    const v = await loadedMany(ENTRY_VISIBLE + 10);
    v.showAllEntries = true;
    flushSync();
    expect(v.visibleEntries).toHaveLength(ENTRY_VISIBLE + 10);
    expect(v.hiddenEntryCount).toBe(10); // 隠れ件数の表示自体は展開状態に依らない
  });

  it("検索の適用で展開状態が畳まれる（前の検索の展開が残らない）", async () => {
    const v = await loadedMany(ENTRY_VISIBLE + 10);
    v.showAllEntries = true;
    v.entrySearch = "n";
    vi.advanceTimersByTime(ENTRY_SEARCH_DEBOUNCE_MS);
    flushSync();
    expect(v.showAllEntries).toBe(false);
    expect(v.visibleEntries).toHaveLength(ENTRY_VISIBLE);
  });

  it("タグ絞り込みの変更でも展開状態が畳まれる", async () => {
    const v = await loadedMany(ENTRY_VISIBLE + 10);
    v.showAllEntries = true;
    flushSync();
    v.toggleTag("work");
    flushSync();
    expect(v.showAllEntries).toBe(false);
  });

  it("openPanel で展開状態が初期化される", async () => {
    invokeMock.mockResolvedValue(many(ENTRY_VISIBLE + 1));
    const v = createVaultView({ t, onError: () => {} });
    await v.load();
    flushSync();
    v.showAllEntries = true;
    v.openPanel();
    flushSync();
    expect(v.showAllEntries).toBe(false);
  });
});
