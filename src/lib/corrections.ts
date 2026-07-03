// 用語補正フェーズの純粋ロジック（文字起こし→整形の間 / E2-E3）。
// LLMから返る「原文 ||| 提案 ||| 理由」の区切り行をパースし、置換適用する。

/** 用語補正の1候補（原文→提案。`replace` が true のものだけ適用対象）。 */
export type Correction = {
  /** 置換元の原文（文字起こし中の表記）。 */
  original: string;
  /** 置換後の提案表記。 */
  suggestion: string;
  /** 補正の理由（UI 表示・ユーザー判断の材料）。 */
  reason: string;
  /** この候補を適用するか（既定 true。ユーザーが個別に外せる）。 */
  replace: boolean;
};

/**
 * LLM応答（区切り行）を補正候補配列へパースする。
 * 各行は `原文 ||| 提案 ||| 理由`。原文=提案や空のものは除外し、`replace` は既定 true。
 * @param raw LLM から返る改行区切りの応答テキスト。
 * @returns パースされた補正候補の配列（不正な行は無視）。
 */
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

/**
 * 選択された補正を本文へ全置換適用する（`replace=true` かつ提案が非空のもの）。
 * @param text 置換対象の本文。
 * @param corrections 適用する補正候補（各出現箇所を単純全置換する）。
 * @returns 置換後テキスト（`text`）と適用件数（`applied`）。
 */
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
