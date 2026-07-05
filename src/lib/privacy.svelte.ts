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
  /** OpenAI互換の接続先(base_url / #593)。loopback を指すなら端末内完結扱い。 */
  getBaseUrl: () => string;
  /** STT 設定をバックエンドへ反映する（ローカル切替後に呼ぶ）。 */
  syncStt: () => void;
}

/**
 * URL のホストが loopback（端末内＝オンデバイス完結）かを判定する（#593）。
 * localhost / 127.0.0.0/8 / ::1 / 0.0.0.0 のみ true。LAN・リモートは false。
 * スキーム付きの正しい URL のみを loopback と認める。パース不能は false（安全側＝
 * リモートを誤って「端末内」と表示しない）。判定は「通信の宛先」の事実に限る。
 */
export function isLoopbackUrl(url: string): boolean {
  const s = url.trim();
  if (!s) return false;
  try {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- 使い捨てのURLパース(リアクティブ状態ではない)
    const host = new URL(s).hostname.toLowerCase().replace(/^\[|\]$/g, "");
    return host === "localhost" || host === "::1" || host === "0.0.0.0" || /^127\./.test(host);
  } catch {
    return false;
  }
}

/**
 * 整形の接続先が端末内で完結するか（#593）。ローカルプロバイダ(Ollama)は常に true、
 * OpenAI互換は base_url が loopback を指すときだけ true（self-host ゲートウェイ/ローカルLLM）。
 */
function refineEndpointIsLocal(provider: string, baseUrl: string): boolean {
  if ((LOCAL_PROVIDERS as readonly string[]).includes(provider)) return true;
  if (provider === "openai") return isLoopbackUrl(baseUrl);
  return false;
}

/**
 * プライバシー／オフラインモードの状態モジュールを生成する。
 * @param deps provider/sttProvider の getter/setter と STT 同期。
 * @returns offlineMode/isFullyLocal と makeOffline/setOfflineMode。
 */
export function createPrivacy(deps: PrivacyDeps) {
  // オフライン固定モード(#465): ONの間はクラウドプロバイダ選択を無効化し、常にローカルに固定する。
  let offlineMode = $state<boolean>(false);

  // 「オンデバイス完結」か（整形の宛先がローカル かつ STT=ローカル）。誇張なく現状を可視化する。
  // 判定は provider 名だけでなく、OpenAI互換の base_url が loopback かも見る（#593）。
  const isFullyLocal = $derived(
    refineEndpointIsLocal(deps.getProvider(), deps.getBaseUrl()) &&
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
