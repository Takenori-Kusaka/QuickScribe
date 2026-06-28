// 保管庫エントリ／タグ関連の純粋関数（#402）。App.svelte から抽出してテスト可能化。

/** エントリ種別を日本語ラベルへ。未知の値はそのまま返す。 */
export function kindLabel(kind: string): string {
  switch (kind) {
    case "transcript":
      return "文字起こし";
    case "refined":
      return "整形済み";
    case "note":
      return "メモ";
    default:
      return kind;
  }
}

/**
 * 入力文字列をタグ配列へ変換する。
 * カンマ/全角カンマ/空白区切り、前後空白除去、先頭の # 除去、空・重複は除く。
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
