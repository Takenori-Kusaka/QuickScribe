// 保管庫エントリ／タグ関連の純粋関数（#402）。App.svelte から抽出してテスト可能化。

/**
 * エントリ種別を i18n キーへ（#401）。呼び出し側で `$_(kindLabel(kind))` と翻訳する。
 * @param kind 種別文字列（transcript/refined/note など）。
 * @returns 対応する i18n キー。未知の値はそのまま返す（キー欠如時に素通し表示）。
 */
export function kindLabel(kind: string): string {
  switch (kind) {
    case "transcript":
      return "results.kind_transcript";
    case "refined":
      return "results.kind_refined";
    case "note":
      return "results.kind_note";
    default:
      return kind;
  }
}

/** 検索/絞り込みに必要なエントリの最小構造。 */
export interface FilterableEntry {
  name: string;
  preview: string;
  tags: string[];
}

/**
 * エントリを検索語と選択タグで絞り込む（純粋 / #402・#392）。
 * @typeParam T 少なくとも {@link FilterableEntry} を満たすエントリ型。
 * @param entries 絞り込み対象のエントリ配列。
 * @param query 検索語（空なら全通し。非空ならファイル名/本文プレビュー/タグの部分一致・大小無視）。
 * @param selectedTags AND 条件のタグ（指定タグを全て含むものだけ通す）。既定は空。
 * @returns 条件を満たすエントリのみを含む新しい配列。
 */
export function filterEntries<T extends FilterableEntry>(
  entries: T[],
  query: string,
  selectedTags: string[] = [],
): T[] {
  const q = query.trim().toLowerCase();
  return entries.filter((e) => {
    if (selectedTags.length > 0 && !selectedTags.every((t) => e.tags.includes(t))) return false;
    if (!q) return true;
    const hay = `${e.name} ${e.preview} ${e.tags.join(" ")}`.toLowerCase();
    return hay.includes(q);
  });
}

/**
 * 入力文字列をタグ配列へ変換する。
 * カンマ/全角カンマ/空白区切り、前後空白除去、先頭の # 除去、空・重複は除く。
 * @param s ユーザー入力のタグ文字列。
 * @returns 正規化済みのタグ配列（順序維持・重複排除）。
 */
export function parseTags(s: string): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const raw of s.split(/[,、\s]+/)) {
    const t = raw.trim().replace(/^#+/, "");
    if (t && !seen.has(t)) {
      seen.add(t);
      out.push(t);
    }
  }
  return out;
}
