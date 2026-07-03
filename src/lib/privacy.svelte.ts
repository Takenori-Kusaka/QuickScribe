// ローカル完結（オフライン／プライバシー）モードの状態とロジック（#465 / #392 App.svelte 分割）。
// 「整形=ローカル(Ollama) かつ 文字起こし=ローカルwhisper」のときだけオンデバイス完結。
// オフライン固定モードは永続化し、ONの間はローカルへ固定する。provider/sttProvider 自体は
// App の設定 state に残るため、getter/setter とSTT同期をDIで受ける。
import { LOCAL_PROVIDERS } from "./constants";

/** privacy ストアが App の provider/sttProvider 状態を読み書きするための依存。 */
export interface PrivacyDeps {
  getProvider: () => string;
  setProvider: (v: string) => void;
  getSttProvider: () => string;
  setSttProvider: (v: string) => void;
  /** STT 設定をバックエンドへ反映する（ローカル切替後に呼ぶ）。 */
  syncStt: () => void;
}

/**
 * プライバシー／オフラインモードの状態モジュールを生成する。
 * @param deps provider/sttProvider の getter/setter と STT 同期。
 * @returns offlineMode/isFullyLocal と makeOffline/setOfflineMode。
 */
export function createPrivacy(deps: PrivacyDeps) {
  // オフライン固定モード(#465): ONの間はクラウドプロバイダ選択を無効化し、常にローカルに固定する。
  let offlineMode = $state<boolean>(false);

  // 「オンデバイス完結」か（整形=ローカル かつ STT=ローカル）。誇張なく現状を可視化する。
  const isFullyLocal = $derived(
    (LOCAL_PROVIDERS as readonly string[]).includes(deps.getProvider()) &&
      deps.getSttProvider() === "local",
  );

  /** ワンクリックでローカルAI(整形=Ollama / STT=ローカルwhisper)へ切り替える。 */
  function makeOffline() {
    deps.setProvider("ollama");
    deps.setSttProvider("local");
    deps.syncStt();
  }

  /** オフラインモードの切替。ONにするとローカルへ固定＋永続化する。 */
  function setOfflineMode(on: boolean) {
    offlineMode = on;
    try {
      localStorage.setItem("offlineMode", String(on));
    } catch {
      /* localStorage 不可環境では状態のみ */
    }
    if (on) makeOffline();
  }

  return {
    get offlineMode() {
      return offlineMode;
    },
    set offlineMode(v: boolean) {
      offlineMode = v;
    },
    get isFullyLocal() {
      return isFullyLocal;
    },
    makeOffline,
    setOfflineMode,
  };
}
