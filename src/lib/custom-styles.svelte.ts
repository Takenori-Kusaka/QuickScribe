// ユーザー定義のカスタム整形スタイル（S3.3 / #392 App.svelte 分割）。
// 組み込みスタイル(REFINE_STYLES)とカスタムを統合した選択肢リスト(allStyles)を提供し、
// 追加・削除・localStorage 永続化を担う runes ストア。整形スタイルの選択値(refineStyle)自体は
// App の設定 state に残るため、削除時のフォールバックは onRemovedActive コールバックで委譲する。
import { REFINE_STYLES, type CustomStyle } from "./constants";
import { type Translator } from "./errors";

/** allStyles の1項目（value/label/desc）。組み込み・カスタム共通の形。 */
export type StyleOption = { value: string; label: string; desc: string };

/**
 * カスタム整形スタイルの状態モジュールを生成する。
 * @param deps 翻訳関数・エラー通知・現在選択判定・削除時フォールバック。
 * @returns customStyles/newCustom* と allStyles、add/remove。
 */
export function createCustomStyles(deps: {
  t: Translator;
  onError: (msg: string) => void;
  /** 指定スタイル値が現在選択中か（削除時のフォールバック要否判定）。 */
  isActive: (value: string) => boolean;
  /** 選択中のカスタムを削除したときに既定スタイルへ戻す処理。 */
  onRemovedActive: () => void;
}) {
  let customStyles = $state<CustomStyle[]>([]);
  let newCustomLabel = $state<string>("");
  let newCustomInstruction = $state<string>("");

  // 組み込み＋カスタムを1つの選択肢リストに統合（チップ・設定ドロップダウン・解説で共用）。
  // 組み込みの label/desc は i18n キー(catalog.styles.*)を deps.t で解決する(#401)。
  const allStyles = $derived<StyleOption[]>([
    ...REFINE_STYLES.map((s) => ({
      value: s.value,
      label: deps.t(s.labelKey),
      desc: deps.t(s.descKey),
    })),
    ...customStyles.map((c) => ({
      value: `custom:${c.id}`,
      label: c.label || deps.t("catalog.styles.custom.label"),
      desc: c.instruction.trim() || deps.t("catalog.styles.custom.desc"),
    })),
  ]);

  function persist() {
    localStorage.setItem("customStyles", JSON.stringify(customStyles));
  }

  /** 入力フォームからカスタムを1件追加する。名前・指示のどちらか空なら onError で通知する。 */
  function addCustomStyle() {
    const label = newCustomLabel.trim();
    const instruction = newCustomInstruction.trim();
    if (!label || !instruction) {
      deps.onError(deps.t("errors.custom_need_both"));
      return;
    }
    const id =
      typeof crypto !== "undefined" && crypto.randomUUID
        ? crypto.randomUUID().slice(0, 8)
        : String(Date.now());
    customStyles = [...customStyles, { id, label, instruction }];
    newCustomLabel = "";
    newCustomInstruction = "";
    persist();
  }

  /** カスタムを削除する。削除対象が選択中なら onRemovedActive で既定へ戻す。 */
  function removeCustomStyle(id: string) {
    customStyles = customStyles.filter((c) => c.id !== id);
    if (deps.isActive(`custom:${id}`)) deps.onRemovedActive();
    persist();
  }

  return {
    get customStyles() {
      return customStyles;
    },
    set customStyles(v: CustomStyle[]) {
      customStyles = v;
    },
    get newCustomLabel() {
      return newCustomLabel;
    },
    set newCustomLabel(v: string) {
      newCustomLabel = v;
    },
    get newCustomInstruction() {
      return newCustomInstruction;
    },
    set newCustomInstruction(v: string) {
      newCustomInstruction = v;
    },
    get allStyles() {
      return allStyles;
    },
    addCustomStyle,
    removeCustomStyle,
  };
}
