// 保管庫エントリの一覧・絞り込み・閲覧の状態とロジック（S4.3 / #392 App.svelte 分割）。
// Svelte 5 の runes モジュール（.svelte.ts）。App.svelte から状態ごと抽出して凝集させ、
// コンポーネントは薄いオーケストレータに保つ。横断発見(discovery)は整形設定に依存するため
// App 側に残し、本モジュールは filteredEntries を公開して連携する。
import { invoke } from "@tauri-apps/api/core";
import { filterEntries } from "./entry";
import { computeStreak } from "./streak";
import { errorText, type Translator } from "./errors";

export type EntrySummary = {
  path: string;
  name: string;
  created: string;
  kind: string;
  tags: string[];
  preview: string;
};

/** 依存注入: 翻訳関数と、App 側 error 状態へ書き戻すコールバック。 */
export interface VaultViewDeps {
  t: Translator;
  onError: (msg: string) => void;
}

/**
 * 保管庫閲覧の状態モジュールを生成する。getter/setter 経由で $state を公開し、
 * テンプレートからは `vault.entries` 等で参照・双方向バインドできる。
 */
export function createVaultView(deps: VaultViewDeps) {
  let showEntries = $state(false);
  let entries = $state<EntrySummary[]>([]);
  let entriesLoading = $state(false);
  let entrySearch = $state<string>("");
  let selectedTags = $state<string[]>([]);
  let viewingEntry = $state<{ name: string; content: string } | null>(null);

  // 全エントリのタグ集合（絞り込みチップ用・出現頻度降順）。
  const allTags = $derived.by(() => {
    const count = new Map<string, number>();
    for (const e of entries) for (const t of e.tags) count.set(t, (count.get(t) ?? 0) + 1);
    return [...count.entries()].sort((a, b) => b[1] - a[1]).map(([t]) => t);
  });
  // 検索語(name/preview/tags)＋選択タグ(AND)で絞り込んだ一覧。
  const filteredEntries = $derived.by(() => filterEntries(entries, entrySearch, selectedTags));
  // 習慣ナッジ: 記録日の寛容ストリーク(1日サボりまで許容 / #58)。
  const journalStreak = $derived(
    computeStreak(
      entries.map((e) => e.created),
      new Date().toISOString().slice(0, 10),
    ),
  );

  async function load() {
    entriesLoading = true;
    try {
      entries = await invoke<EntrySummary[]>("list_entries");
    } catch (e) {
      deps.onError(deps.t("errors.journal_load", { values: { detail: errorText(e, deps.t) } }));
      entries = [];
    } finally {
      entriesLoading = false;
    }
  }

  function openPanel() {
    showEntries = true;
    viewingEntry = null;
    void load();
  }

  function toggleTag(tag: string) {
    selectedTags = selectedTags.includes(tag)
      ? selectedTags.filter((t) => t !== tag)
      : [...selectedTags, tag];
  }

  async function openEntry(e: EntrySummary) {
    try {
      const content = await invoke<string>("read_text_file", { path: e.path });
      viewingEntry = { name: e.name, content };
    } catch (err) {
      deps.onError(deps.t("errors.entry_open", { values: { detail: errorText(err, deps.t) } }));
    }
  }

  return {
    get showEntries() {
      return showEntries;
    },
    set showEntries(v: boolean) {
      showEntries = v;
    },
    get entries() {
      return entries;
    },
    get entriesLoading() {
      return entriesLoading;
    },
    get entrySearch() {
      return entrySearch;
    },
    set entrySearch(v: string) {
      entrySearch = v;
    },
    get selectedTags() {
      return selectedTags;
    },
    get viewingEntry() {
      return viewingEntry;
    },
    set viewingEntry(v: { name: string; content: string } | null) {
      viewingEntry = v;
    },
    get allTags() {
      return allTags;
    },
    get filteredEntries() {
      return filteredEntries;
    },
    get journalStreak() {
      return journalStreak;
    },
    load,
    openPanel,
    toggleTag,
    openEntry,
  };
}

export type VaultView = ReturnType<typeof createVaultView>;
