// ユーザー向けエラー文言の整形（#398 P1 / #462 i18n Phase2）。
// Tauri invoke は Rust の Result 文字列、JS例外は Error を投げる。
// 生の値をそのまま画面に出すと、英語スタックや技術的詳細がユーザーに漏れる。
// errorText() は「人が読める1行」を返す。呼び出し側で文脈プレフィックスを添える。
//
// #462: Rust バックエンドは日本語ハードコードをやめ、安定エラーコード
// `E_XXX`（必要なら区切り文字 U+001F に続けて技術詳細）を返す。errorText は
// 翻訳関数 `t`（svelte-i18n の $_）を受け取ると、コードを `errors.rust.<CODE>`
// カタログのローカライズ文言へ写像する。未知コード・生文字列は安全にフォールバックする。

/** 翻訳器が無い場合の最終フォールバック（既定=日本語）。翻訳器があれば `errors.unknown` を使う。 */
const FALLBACK = "原因不明のエラーが発生しました。";
const UNKNOWN_KEY = "errors.unknown";

/** Rust 側のコードと技術詳細を区切る制御文字（U+001F Unit Separator）。errcode.rs と一致。 */
export const ERR_CODE_SEP = "";

/** コード部の形式（`E_` + 大文字/数字/_）。区切り文字は含めず、制御文字を正規表現に持ち込まない。 */
const CODE_RE = /^E_[A-Z0-9_]+$/;

/** svelte-i18n 互換の翻訳関数。未知キーはキー自身を返す前提。 */
export type Translator = (key: string, opts?: { values?: Record<string, string> }) => string;

/** unknown なエラー値から生の文字列を取り出す（空なら ""）。 */
function extract(e: unknown): string {
  if (typeof e === "string") return e.trim();
  if (e instanceof Error) return e.message.trim() || e.name;
  if (e && typeof e === "object" && "message" in e) {
    const m = (e as { message: unknown }).message;
    if (typeof m === "string") return m.trim();
  }
  return e == null ? "" : String(e);
}

/**
 * unknown なエラー値から、ユーザー表示に適した簡潔な文字列を取り出す。
 * `t` を渡すと Rust の安定エラーコード（`E_XXX`）を `errors.rust.<CODE>` へ翻訳する。
 * - 既知コード: ローカライズ文言（`{detail}` に技術詳細を補間）。
 * - 未知コード: 技術詳細（無ければコード）を返す（区切り文字は露出させない）。
 * - コード以外の生文字列: そのまま（空なら翻訳された `errors.unknown` / FALLBACK）。
 */
export function errorText(e: unknown, t?: Translator): string {
  const raw = extract(e);
  const sepIdx = raw.indexOf(ERR_CODE_SEP);
  const code = sepIdx >= 0 ? raw.slice(0, sepIdx) : raw;
  const detail = sepIdx >= 0 ? raw.slice(sepIdx + 1) : "";
  if (CODE_RE.test(code)) {
    if (t) {
      const key = `errors.rust.${code}`;
      const msg = t(key, { values: { detail } });
      if (msg !== key) return msg; // 既知コード → ローカライズ済み
    }
    return detail || code; // 未知コード or 翻訳器なし → 安全なフォールバック
  }
  if (raw) return raw;
  // 空・null: 翻訳器があればローカライズ、無ければ既定文言。
  if (t) {
    const msg = t(UNKNOWN_KEY);
    if (msg !== UNKNOWN_KEY) return msg;
  }
  return FALLBACK;
}
