// ユーザー向けエラー文言の整形（#398 P1）。
// Tauri invoke は Rust の Result 文字列(多くは日本語)を、JS例外は Error を投げる。
// 生の値をそのまま画面に出すと、英語スタックや技術的詳細がユーザーに漏れる。
// errorText() は「人が読める1行」を返す。呼び出し側で文脈プレフィックスを添える。

const FALLBACK = "原因不明のエラーが発生しました。";

/** unknown なエラー値から、ユーザー表示に適した簡潔な文字列を取り出す。 */
export function errorText(e: unknown): string {
  if (typeof e === "string") return e.trim() || FALLBACK;
  if (e instanceof Error) return e.message.trim() || e.name;
  if (e && typeof e === "object" && "message" in e) {
    const m = (e as { message: unknown }).message;
    if (typeof m === "string") return m.trim() || FALLBACK;
  }
  return e == null ? FALLBACK : String(e);
}
