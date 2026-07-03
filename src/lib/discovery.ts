// 横断発見（複数エントリをAIで読み解く / S4.3）の純粋ロジック（#402）。
// App.svelte の discoverAcross から、対象選択とプロンプト整形を抽出。

export interface DiscoveryItem {
  created: string;
  tags: string[];
  content: string;
}

/**
 * 横断発見の対象エントリを上限件数で切り出す（プロンプト肥大の防止）。
 * @typeParam T エントリ型（本関数は先頭 max 件を取るだけで内容に依存しない）。
 * @param entries 対象候補（絞り込み済みの順序を尊重する）。
 * @param max 取り出す最大件数。
 * @returns `targets`（先頭 max 件）と `truncated`（超過したか）。
 */
export function selectDiscoveryTargets<T>(
  entries: T[],
  max: number,
): { targets: T[]; truncated: boolean } {
  return { targets: entries.slice(0, max), truncated: entries.length > max };
}

/**
 * 対象エントリ群を、AIへ渡す単一テキストへ整形する。
 * 各エントリは `### <日時> #tag...` の見出し＋本文、区切りは `---`。
 * @param items 整形対象のエントリ（日時・タグ・本文）。
 * @returns 連結した単一プロンプトテキスト。
 */
export function buildDiscoveryText(items: DiscoveryItem[]): string {
  return items
    .map((e) => {
      const tagStr = e.tags.map((t) => `#${t}`).join(" ");
      return `### ${e.created} ${tagStr}\n${e.content}`;
    })
    .join("\n\n---\n\n");
}
