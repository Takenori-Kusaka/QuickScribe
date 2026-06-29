// 保管庫エントリ／タグ関連の純粋関数（#402）。App.svelte から抽出してテスト可能化。

/**
 * エントリ種別を i18n キーへ（#401）。未知の値はそのまま返す。
 * 呼び出し側で `$_(kindLabel(kind))` のように翻訳する。キー欠如時は
 * svelte-i18n がキー文字列を返すため、未知値はそのまま表示される。
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
