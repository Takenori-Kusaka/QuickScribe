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

/** 整形プロバイダを検証。未知は既定 "gemini" へ。 */
export function clampProvider(p: string): Provider {
  return (p in PROVIDER_LABELS ? p : "gemini") as Provider;
}

/** STTプロバイダを検証。未知は既定 "local" へ。 */
export function clampSttProvider(p: string): SttProvider {
  return (p in STT_LABELS ? p : "local") as SttProvider;
}

/** 許容値のいずれかへクランプ（含まれなければ既定）。 */
export function clampOneOf<T extends string>(v: string, allowed: readonly T[], def: T): T {
  return (allowed as readonly string[]).includes(v) ? (v as T) : def;
}

/** 整形スタイルが「組み込み」または「存在するカスタム」かを判定。 */
export function isValidRefineStyle(style: string, customStyles: CustomStyle[]): boolean {
  if (REFINE_STYLES.some((s) => s.value === style)) return true;
  return style.startsWith("custom:") && customStyles.some((c) => `custom:${c.id}` === style);
}
