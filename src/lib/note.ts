// 録音メモに関する純粋ロジック。UI/Tauriから分離してテスト可能にする（S7.2）。

/** 録音開始・停止のミリ秒から経過秒を求める（負値は0に丸める）。 */
export function elapsedSeconds(startMs: number, endMs: number): number {
  return Math.max(0, Math.round((endMs - startMs) / 1000));
}

/** Phase 1 のプレースホルダ保存内容を組み立てる（文字起こし導入で置き換え予定）。 */
export function buildNoteContent(seconds: number): string {
  return `QuickScribe メモ (録音 ${seconds}s) — Phase1 プレースホルダ`;
}

/** 経過時間と進捗(0-100)から残り秒数を推定する。 */
export function estimateRemaining(elapsedSec: number, progressPct: number): number {
  if (progressPct <= 0 || progressPct >= 100) return 0;
  return (elapsedSec * (100 - progressPct)) / progressPct;
}

/** 残り秒数を「残り 約N分N秒」形式に整形する（進捗ETA表示用）。 */
export function formatRemaining(seconds: number): string {
  if (!isFinite(seconds) || seconds <= 0) return "";
  const s = Math.round(seconds);
  if (s < 60) return `残り 約${s}秒`;
  const m = Math.floor(s / 60);
  const rem = s % 60;
  return rem === 0 ? `残り 約${m}分` : `残り 約${m}分${rem}秒`;
}
