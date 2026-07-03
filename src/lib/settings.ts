// 設定値の検証・クランプ（純粋関数 / #402 Phase2・ADR-0017）。
// App.svelte の validateSettings から純ロジックを抽出し、単体テスト可能にする。
// 破損/未知の値は既定へクランプ（破損耐性）。未知キー自体は呼び出し側で保持（非破壊）。
import {
  PROVIDER_LABELS,
  STT_LABELS,
  REFINE_STYLES,
  type Provider,
  type SttProvider,
  type CustomStyle,
} from "./constants";

/**
 * 整形プロバイダを検証する。
 * @param p 検証する値（localStorage 等の生値）。
 * @returns 既知のプロバイダならそのまま、未知は既定 "gemini"。
 */
export function clampProvider(p: string): Provider {
  return (p in PROVIDER_LABELS ? p : "gemini") as Provider;
}

/**
 * STTプロバイダを検証する。
 * @param p 検証する値（localStorage 等の生値）。
 * @returns 既知の STT プロバイダならそのまま、未知は既定 "local"。
 */
export function clampSttProvider(p: string): SttProvider {
  return (p in STT_LABELS ? p : "local") as SttProvider;
}

/**
 * 許容値のいずれかへクランプする。
 * @typeParam T 許容される文字列リテラルの型。
 * @param v 検証する値。
 * @param allowed 許容値の配列。
 * @param def どれにも一致しないときの既定値。
 * @returns `v` が許容値なら `v`、そうでなければ `def`。
 */
export function clampOneOf<T extends string>(v: string, allowed: readonly T[], def: T): T {
  return (allowed as readonly string[]).includes(v) ? (v as T) : def;
}

/**
 * 整形スタイルが「組み込み」または「存在するカスタム」かを判定する。
 * @param style 検証するスタイル値（例 "structured" / "custom:ab12"）。
 * @param customStyles 現在定義済みのカスタムスタイル一覧。
 * @returns 有効なスタイルなら true。
 */
export function isValidRefineStyle(style: string, customStyles: CustomStyle[]): boolean {
  if (REFINE_STYLES.some((s) => s.value === style)) return true;
  return style.startsWith("custom:") && customStyles.some((c) => `custom:${c.id}` === style);
}
