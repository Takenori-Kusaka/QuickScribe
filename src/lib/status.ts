// バックエンドの status イベント文言のローカライズ（#462 Phase2）。
// Rust 側は日本語ハードコードをやめ、安定コード `S_XXX`（必要なら U+001F 区切りで
// 補足値）を emit する。フロントは statusText() で `status.rust.<CODE>` カタログ
// （ja/en/zh/es）へ写像して表示する。未知コード・生文字列は安全にフォールバックする。
import { ERR_CODE_SEP, type Translator } from "./errors";

/** status コード部の形式（`S_` + 大文字/数字/_）。errcode.rs の ALL_STATUS と一致。 */
const STATUS_CODE_RE = /^S_[A-Z0-9_]+$/;

/**
 * status イベントの payload をユーザー表示文字列へ変換する。
 * - 既知コード: ローカライズ文言（`{detail}` に補足値を補間）。
 * - 未知コード: 補足値（無ければコード）を返す（区切り文字は露出させない）。
 * - コード以外の生文字列（空含む）: そのまま返す。
 */
export function statusText(raw: string, t?: Translator): string {
  const sepIdx = raw.indexOf(ERR_CODE_SEP);
  const code = sepIdx >= 0 ? raw.slice(0, sepIdx) : raw;
  const detail = sepIdx >= 0 ? raw.slice(sepIdx + 1) : "";
  if (STATUS_CODE_RE.test(code)) {
    if (t) {
      const key = `status.rust.${code}`;
      const msg = t(key, { values: { detail } });
      if (msg !== key) return msg;
    }
    return detail || code;
  }
  return raw;
}
