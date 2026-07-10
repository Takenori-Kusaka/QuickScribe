// 話者特定(S2.5 / ADR-0031)の純ロジック。話者ラベル `[話者N]` の検出と、
// 話者名リネーム(=既存の一括置換 applyCorrections への橋渡し)を提供する。
// リネームは新規置換ロジックを作らず corrections.ts を再利用する（仕様 R9）。
import type { Correction } from "./corrections";

/** バックエンド(stt.rs)が出力する話者ラベルの接頭辞。ラベルは `[話者N]`（N=1,2,…）。 */
export const SPEAKER_PREFIX = "話者";

/** 正規表現メタ文字をエスケープ（接頭辞を安全に正規表現へ埋め込むため）。 */
function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** 話者トークン `話者N` から番号 N を取り出す（昇順ソート用）。 */
function speakerIndex(token: string, prefix: string): number {
  const n = parseInt(token.slice(prefix.length), 10);
  return Number.isNaN(n) ? 0 : n;
}

/**
 * 文字起こし本文から話者ラベルを重複なく昇順で抽出する（仕様 R8 / detectSpeakers）。
 * タイムスタンプ `[HH:MM:SS]` は接頭辞不一致で拾わない。話者ラベルが無ければ空配列。
 * @param transcript 文字起こし本文。
 * @param prefix 話者ラベルの接頭辞（既定 `話者`）。
 * @returns 例 `["話者1","話者2"]`（番号昇順・一意）。
 */
export function detectSpeakers(transcript: string, prefix: string = SPEAKER_PREFIX): string[] {
  const re = new RegExp(`\\[(${escapeRegExp(prefix)}\\d+)\\]`, "g");
  const seen = new Set<string>();
  for (const m of transcript.matchAll(re)) seen.add(m[1]);
  return [...seen].sort((a, b) => speakerIndex(a, prefix) - speakerIndex(b, prefix));
}

/**
 * 話者→名前の割当を一括置換の補正候補へ変換する（仕様 R9）。空欄の名前はスキップ。
 * `[話者N]` → `[入力名]` の全置換になるよう括弧付きで組み立て、applyCorrections に渡す。
 * @param names 話者トークン(例 `話者1`)→ 入力名(例 `田中`) のマップ。
 * @param prefix 話者ラベルの接頭辞（未使用トークンはそのまま original に使う）。
 * @returns applyCorrections に渡せる Correction 配列（名前が空のものは含めない）。
 */
export function buildSpeakerRenames(
  names: Record<string, string>,
  _prefix: string = SPEAKER_PREFIX,
): Correction[] {
  const out: Correction[] = [];
  for (const [token, name] of Object.entries(names)) {
    const trimmed = name.trim();
    if (!trimmed) continue; // 空欄の話者は変更しない（R8）
    out.push({ original: `[${token}]`, suggestion: `[${trimmed}]`, reason: "", replace: true });
  }
  return out;
}
