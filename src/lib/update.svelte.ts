// 自動アップデート（起動時に非同期チェック→背景DL→完了後に再起動確認 / #392 App.svelte 分割）。
// Svelte5 runes モジュール。UI から独立した更新ドメインの状態と操作を集約する。
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { errorText, type Translator } from "./errors";

export type UpdateState = "idle" | "downloading" | "ready";

/** 更新ドメインの状態モジュールを生成する。翻訳関数 `t`（svelte-i18n の $_）を注入する。 */
export function createUpdater(t: Translator) {
  let updateState = $state<UpdateState>("idle");
  let updateVersion = $state<string>("");
  let updatePct = $state<number>(0);
  let updateMsg = $state<string>("");

  /** 更新確認→ダウンロード/インストール。manual=true のときだけ結果メッセージを表示する。 */
  async function checkForUpdate(manual = false) {
    try {
      updateMsg = manual ? t("update.checking") : "";
      const update = await check();
      if (!update) {
        updateMsg = manual ? t("update.latest") : "";
        return;
      }
      updateMsg = "";
      updateVersion = update.version;
      updateState = "downloading";
      let downloaded = 0;
      let total = 0;
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          updatePct = total > 0 ? Math.round((downloaded / total) * 100) : 0;
        }
      });
      updateState = "ready";
    } catch (e) {
      console.error("update check failed", e);
      updateMsg = manual ? t("update.check_failed", { values: { detail: errorText(e, t) } }) : "";
    }
  }

  async function restartNow() {
    await relaunch();
  }

  return {
    get updateState() {
      return updateState;
    },
    get updateVersion() {
      return updateVersion;
    },
    get updatePct() {
      return updatePct;
    },
    get updateMsg() {
      return updateMsg;
    },
    checkForUpdate,
    restartNow,
  };
}
