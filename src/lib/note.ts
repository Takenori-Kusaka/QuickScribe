// 録音メモに関する純粋ロジック。UI/Tauriから分離してテスト可能にする（S7.2）。
import { type Translator } from "./errors";

/**
 * 録音開始・停止のミリ秒から経過秒を求める（負値は0に丸める）。
 * @param startMs 録音開始時刻（ミリ秒）。
 * @param endMs 録音停止時刻（ミリ秒）。
 * @returns 経過秒（四捨五入・0以上）。
 */
export function elapsedSeconds(startMs: number, endMs: number): number {
  return Math.max(0, Math.round((endMs - startMs) / 1000));
}

/**
 * Phase 1 のプレースホルダ保存内容を組み立てる（文字起こし導入で置き換え予定）。
 * @param seconds 録音秒数。
 * @returns 保存本文（プレースホルダ文字列）。
 */
export function buildNoteContent(seconds: number): string {
  return `QuickScribe メモ (録音 ${seconds}s) — Phase1 プレースホルダ`;
}

/**
 * 経過時間と進捗(0-100)から残り秒数を線形推定する。
 * @param elapsedSec これまでの経過秒。
 * @param progressPct 進捗率（0-100）。0以下/100以上は推定不能として0を返す。
 * @returns 残り秒数の推定値。
 */
export function estimateRemaining(elapsedSec: number, progressPct: number): number {
  if (progressPct <= 0 || progressPct >= 100) return 0;
  return (elapsedSec * (100 - progressPct)) / progressPct;
}

/**
 * 残り秒数を「残り 約N分N秒」形式に整形する（進捗ETA表示用）。
 * 文言は i18n カタログ(eta.*)から解決する（#401: 非日本語UIに日本語を露出させない）。
 * @param seconds 残り秒数。非有限・0以下は空文字を返す。
 * @param t 翻訳関数（svelte-i18n の $_）。省略時は日本語既定で整形する。
 * @returns 整形済み ETA 文字列（例「残り 約2分30秒」）。
 */
export function formatRemaining(seconds: number, t?: Translator): string {
  if (!isFinite(seconds) || seconds <= 0) return "";
  const s = Math.round(seconds);
  const ja: Record<string, (v: Record<string, string>) => string> = {
    "eta.seconds": (v) => `残り 約${v.s}秒`,
    "eta.minutes": (v) => `残り 約${v.m}分`,
    "eta.min_sec": (v) => `残り 約${v.m}分${v.s}秒`,
  };
  const tr = (key: string, values: Record<string, string>) =>
    t ? t(key, { values }) : ja[key](values);
  if (s < 60) return tr("eta.seconds", { s: String(s) });
  const m = Math.floor(s / 60);
  const rem = s % 60;
  return rem === 0
    ? tr("eta.minutes", { m: String(m) })
    : tr("eta.min_sec", { m: String(m), s: String(rem) });
}
