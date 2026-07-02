// 録音ソース選択値のパース（#392 / App.svelte から抽出）。
// プルダウンは "kind|id" を値に使う（id がレンダーデバイスIDで '|' を含んでも安全に分解）。

export interface RecordSource {
  /** "input"(マイク) / "mix"(ループバック) 等の種別。 */
  kind: string;
  /** デバイスID/名（空=OS既定）。'|' を含む場合も保持する。 */
  id: string;
}

/** "kind|id" を分解する（純粋）。最初の '|' で区切り、id 側の '|' は温存。区切り無しは kind のみ。 */
export function parseRecordSource(value: string): RecordSource {
  const sep = value.indexOf("|");
  if (sep < 0) return { kind: value, id: "" };
  return { kind: value.slice(0, sep), id: value.slice(sep + 1) };
}
