// i18n 基盤（#401）。svelte-i18n を同期初期化し、$_('key') で参照する。
// 起動時の言語は「保存済みのユーザー設定 > OSのシステム言語 > 日本語」の優先で決める。
import { addMessages, init, getLocaleFromNavigator } from "svelte-i18n";
import ja from "./ja.json";
import en from "./en.json";
import zh from "./zh.json";
import es from "./es.json";

addMessages("ja", ja);
addMessages("en", en);
addMessages("zh", zh);
addMessages("es", es);

/** 対応ロケール。未対応のOS言語は fallback(ja) で表示される。 */
export const SUPPORTED_LOCALES = ["ja", "en", "zh", "es"] as const;
export const LOCALE_STORAGE_KEY = "locale";

/** 対応ロケールに丸める（未対応は ja）。 */
function clamp(loc: string | null | undefined): string {
  const short = (loc ?? "ja").split("-")[0];
  return (SUPPORTED_LOCALES as readonly string[]).includes(short) ? short : "ja";
}

/** 起動時ロケール: 保存済み設定 → OSシステム言語 → 日本語。 */
function initialLocale(): string {
  const saved =
    typeof localStorage !== "undefined" ? localStorage.getItem(LOCALE_STORAGE_KEY) : null;
  if (saved && (SUPPORTED_LOCALES as readonly string[]).includes(saved)) return saved;
  // OSのシステム言語（例 "ja-JP" / "en-US"）を前方一致で対応ロケールへ。
  return clamp(getLocaleFromNavigator());
}

init({
  fallbackLocale: "ja",
  initialLocale: initialLocale(),
});
