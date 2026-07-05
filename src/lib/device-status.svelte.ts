// OS/デバイス連携のランタイム状態（自動起動・録音ソース列挙・whisperモデル一覧 / #392 App.svelte 分割）。
// 永続化される設定（inputDevice/whisperModel/taskbarWidget 等）は App/settings-persist 側に残し、
// ここは「OS から取得する実行時の状態と一覧」のみを持つ runes ストア。
import { invoke } from "@tauri-apps/api/core";
import {
  enable as enableAutostart,
  disable as disableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import { errorText, type Translator } from "./errors";

/** 録音ソース（マイク入力／出力デバイスのループバック）。 */
export type AudioSource = { id: string; label: string; kind: string };

/** whisper モデルの一覧項目。 */
export type WhisperModelInfo = { id: string; label: string; speed: string };

/**
 * デバイス連携の実行時状態モジュールを生成する。
 * @param deps 翻訳関数 `t` と、App の error 状態へ書き戻す `onError`。
 * @returns autoStart/audioSources/whisperModels と各ローダ・自動起動トグル。
 */
export function createDeviceStatus(deps: { t: Translator; onError: (msg: string) => void }) {
  // OSログイン時の自動起動（S6.3）。実体は OS 登録なので状態は OS から取得する。
  let autoStart = $state<boolean>(false);
  let audioSources = $state<AudioSource[]>([]);
  let whisperModels = $state<WhisperModelInfo[]>([]);

  async function loadAutoStart() {
    try {
      autoStart = await isAutostartEnabled();
    } catch (e) {
      console.error("isAutostartEnabled failed", e);
    }
  }

  /** トグル変更時: OS へ登録/解除し、実際の登録状態へ同期する（失敗時は error へ通知）。 */
  async function onAutoStartChange() {
    try {
      if (autoStart) await enableAutostart();
      else await disableAutostart();
      autoStart = await isAutostartEnabled();
    } catch (e) {
      deps.onError(deps.t("errors.autostart_failed", { values: { detail: errorText(e, deps.t) } }));
      autoStart = await isAutostartEnabled().catch(() => autoStart);
    }
  }

  /** 利用可能な録音ソースを列挙する（設定 UI のプルダウン用）。失敗時は空のまま既定運用。 */
  async function loadAudioSources() {
    try {
      audioSources = await invoke<AudioSource[]>("list_audio_sources");
    } catch (e) {
      console.error("list_audio_sources failed", e);
      audioSources = [];
    }
  }

  /** タスクバーウィジェットの表示有効/無効をバックエンドへ反映する（Windowsのみ実体動作）。 */
  async function applyTaskbarWidget(enabled: boolean) {
    try {
      await invoke("set_taskbar_widget", { enabled });
    } catch (e) {
      console.error("set_taskbar_widget failed", e);
    }
  }

  /** ローカル whisper の選択可能モデルを列挙する。 */
  async function loadWhisperModels() {
    try {
      whisperModels = await invoke<WhisperModelInfo[]>("list_whisper_models");
    } catch (e) {
      console.error("list_whisper_models failed", e);
    }
  }

  return {
    get autoStart() {
      return autoStart;
    },
    set autoStart(v: boolean) {
      autoStart = v;
    },
    get audioSources() {
      return audioSources;
    },
    get whisperModels() {
      return whisperModels;
    },
    loadAutoStart,
    onAutoStartChange,
    loadAudioSources,
    loadWhisperModels,
    applyTaskbarWidget,
  };
}
