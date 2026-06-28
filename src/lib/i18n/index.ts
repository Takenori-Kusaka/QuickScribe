// i18n 基盤（#401 Phase1）。svelte-i18n を同期初期化し、$_('key') で参照する。
// 既定は日本語。Phase1 は単言語（見た目不変）でカタログ機構を確立する。
// Phase3 で en/zh/es 追加＋ロケール切替UI＋OS言語検出（getLocaleFromNavigator）を行う。
import { addMessages, init } from "svelte-i18n";
import ja from "./ja.json";

addMessages("ja", ja);

init({
  fallbackLocale: "ja",
  initialLocale: "ja",
});
