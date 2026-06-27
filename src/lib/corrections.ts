// 用語補正フェーズの純粋ロジック（文字起こし→整形の間 / E2-E3）。
// LLMから返る「原文 ||| 提案 ||| 理由」の区切り行をパースし、置換適用する。

export type Correction = {
  original: string;
  suggestion: string;
  reason: string;
  replace: boolean;
};

/// LLM応答（区切り行）を補正候補配列へパースする。
/// 各行 `原文 ||| 提案 ||| 理由`。原文=提案や空のものは除外。replace は既定 true。
export function parseCorrections(raw: string): Correction[] {
  const out: Correction[] = [];
  for (const line of raw.split(/\r?\n/)) {
    const parts = line.split("|||").map((s) => s.trim());
    if (parts.length >= 2 && parts[0] && parts[1] && parts[0] !== parts[1]) {
      out.push({
        original: parts[0],
        suggestion: parts[1],
        reason: parts[2] ?? "",
        replace: true,
      });
    }
  }
  return out;
}

/// 選択された補正を本文へ全置換適用する（replace=true かつ提案が非空のもの）。
/// 戻り値: 置換後テキストと適用件数。
export function applyCorrections(
  text: string,
  corrections: Correction[],
): { text: string; applied: number } {
  let t = text;
  let applied = 0;
  for (const c of corrections) {
    const sugg = c.suggestion.trim();
    if (c.replace && sugg && c.original) {
      t = t.split(c.original).join(sugg);
      applied++;
    }
  }
  return { text: t, applied };
}
