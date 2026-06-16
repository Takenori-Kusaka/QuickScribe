// 録音メモに関する純粋ロジック。UI/Tauriから分離してテスト可能にする（S7.2）。

/** 録音開始・停止のミリ秒から経過秒を求める（負値は0に丸める）。 */
export function elapsedSeconds(startMs: number, endMs: number): number {
  return Math.max(0, Math.round((endMs - startMs) / 1000));
}

/** Phase 1 のプレースホルダ保存内容を組み立てる（文字起こし導入で置き換え予定）。 */
export function buildNoteContent(seconds: number): string {
  return `QuickScribe メモ (録音 ${seconds}s) — Phase1 プレースホルダ`;
}
