// 録音トグルのグローバルホットキー：状態＋キャプチャ／登録操作（#392 App.svelte 分割 / ADR-0014）。
// 表示・accelerator 変換の純粋関数は lib/shortcut.ts に既出。ここはその上に UI 状態と
// バックエンド登録（Tauri invoke）を載せる runes ストア。
import { invoke } from "@tauri-apps/api/core";
import { displayShortcut, accelFromEvent } from "./shortcut";
import { errorText, type Translator } from "./errors";
import { DEFAULT_SHORTCUT } from "./constants";

/**
 * ホットキー状態モジュールを生成する。翻訳関数 `t`（svelte-i18n の $_）を注入する。
 * @param t 翻訳関数。
 * @returns recordShortcut/shortcutMsg/capturing と操作（capture/apply/reset/display）。
 */
export function createShortcut(t: Translator) {
  // 実行環境(OS)。修飾キーを親しみやすい表記(Cmd/Ctrl)に出し分けるために保持。
  const isMac =
    typeof navigator !== "undefined" &&
    /mac/i.test(`${navigator.userAgent} ${navigator.platform ?? ""}`);

  let recordShortcut = $state<string>(DEFAULT_SHORTCUT);
  let shortcutMsg = $state<string>("");
  let capturing = $state<boolean>(false);

  /** 現在のショートカットを人が読む表記へ（`displayShortcut(recordShortcut, isMac)` の糖衣）。 */
  function display(): string {
    return displayShortcut(recordShortcut, isMac);
  }

  function startCapture() {
    capturing = true;
    shortcutMsg = "";
  }
  function cancelCapture() {
    capturing = false;
  }

  /** キャプチャ中の keydown を解釈し、有効な組合せならショートカットを更新・登録する。 */
  function onCaptureKeydown(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    if (e.key === "Escape") {
      // ホットキー取得のキャンセルに留め、設定ダイアログ全体を閉じさせない(#395)。
      e.stopPropagation();
      cancelCapture();
      return;
    }
    // 修飾キー単体は待機を継続（組合せの確定を待つ）。
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
    const accel = accelFromEvent(e);
    if (!accel) {
      // 修飾キー無しは誤爆防止のため不可。ヒントを出して待機継続。
      shortcutMsg = t("settings.shortcut_modifier_required");
      return;
    }
    recordShortcut = accel;
    capturing = false;
    void applyShortcut();
  }

  function resetShortcut() {
    recordShortcut = DEFAULT_SHORTCUT;
    void applyShortcut();
  }

  /** 現在の表記でグローバルホットキーを再登録する（localStorage 永続化＋タスクバー同期）。 */
  async function applyShortcut() {
    try {
      await invoke("set_record_shortcut", { accelerator: recordShortcut });
      localStorage.setItem("recordShortcut", recordShortcut);
      void invoke("set_taskbar_shortcut", { display: displayShortcut(recordShortcut, isMac) });
      shortcutMsg = t("errors.hotkey_set", {
        values: { key: displayShortcut(recordShortcut, isMac) },
      });
    } catch (e) {
      shortcutMsg = t("errors.hotkey_failed", { values: { detail: errorText(e, t) } });
    }
  }

  return {
    get recordShortcut() {
      return recordShortcut;
    },
    set recordShortcut(v: string) {
      recordShortcut = v;
    },
    get shortcutMsg() {
      return shortcutMsg;
    },
    get capturing() {
      return capturing;
    },
    get isMac() {
      return isMac;
    },
    display,
    startCapture,
    cancelCapture,
    onCaptureKeydown,
    resetShortcut,
    applyShortcut,
  };
}
