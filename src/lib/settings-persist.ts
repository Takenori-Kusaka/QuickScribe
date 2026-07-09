// 設定の永続化（localStorage 読み書き＋スキーマ検証 / S5.3・ADR-0017 / #392 App.svelte 分割）。
// localStorage のキー名・既定値・クランプ（破損耐性）の知識をここへ集約し、App.svelte からは
// readSettings()/writeSettings() の2関数で扱う。秘密情報(API鍵/AWS鍵)は keyring 側(lib/secrets)。
import {
  ALL_PROVIDERS,
  LOCAL_PROVIDERS,
  DEFAULT_SHORTCUT,
  type Provider,
  type SttProvider,
  type CustomStyle,
} from "./constants";
import { clampProvider, clampSttProvider, clampOneOf, isValidRefineStyle } from "./settings";

/** 設定スキーマ版（ADR-0017）。形式が将来変わってもクランプ＋移行で壊れないようにする。 */
export const SETTINGS_VERSION = 1;

/** localStorage に永続化される設定一式（秘密情報は含まない）。 */
export interface AppSettings {
  provider: Provider;
  resolvedModel: Record<Provider, string>;
  recordShortcut: string;
  recordMode: "toggle" | "momentary";
  includeTimestamps: boolean;
  autoPipeline: boolean;
  keepText: boolean;
  saveAudio: boolean;
  audioFormat: string;
  saveDir: string;
  outputFormat: string;
  refineStyle: string;
  translateOutput: boolean;
  outputLang: string;
  /** OpenAI互換エンドポイントの接続先(base_url / #593)。既定は空＝公式 api.openai.com。 */
  openaiBaseUrl: string;
  sttProvider: SttProvider;
  /** GPUで文字起こし(Vulkan変種のみ実効・既定ON=速度最適 / ADR-0028)。 */
  sttUseGpu: boolean;
  offlineMode: boolean;
  sttModel: string;
  sttAzureResource: string;
  whisperModel: string;
  customStyles: CustomStyle[];
  awsRegion: string;
  awsWorkspaceId: string;
  awsAuthMode: "sigv4" | "apikey";
  bedrockModel: string;
  taskbarWidget: boolean;
  inputDevice: string;
  inputDeviceKind: string;
  /** 習慣ナッジ(S9.4 #58): 起動時に継続中ストリークが未記録なら通知で促す。既定OFF(opt-in)。 */
  nudgeEnabled: boolean;
}

function emptyModelMap(): Record<Provider, string> {
  return Object.fromEntries(ALL_PROVIDERS.map((p) => [p, ""])) as Record<Provider, string>;
}

/**
 * localStorage から設定を読み、破損/旧値は既定へクランプして返す（非破壊: 未知キーは保持）。
 * @param localeDefault 出力言語の既定（未保存時に使う起動時UI言語 例 "ja"）。
 * @returns 検証済みの設定一式。offlineMode 時は provider/sttProvider をローカルへ固定する。
 */
export function readSettings(localeDefault: string): AppSettings {
  const ls = localStorage;
  // 整形プロバイダの既定はローカルファースト = ollama(#465/ADR-0021)。鍵不要でプライバシー既定。
  // 保存済みは優先。破損値も安全側でローカル(ollama)へ寄せる。
  let provider = (ls.getItem("provider") as Provider) || "ollama";
  if (!(ALL_PROVIDERS as string[]).includes(provider)) provider = "ollama";

  const resolvedModel = emptyModelMap();
  for (const p of ALL_PROVIDERS) resolvedModel[p] = ls.getItem(`resolvedModel:${p}`) ?? "";

  let sttProvider = (ls.getItem("sttProvider") as SttProvider) || "local";
  const sttUseGpu = ls.getItem("sttUseGpu") !== "false"; // 既定ON(ADR-0027)
  const offlineMode = ls.getItem("offlineMode") === "true";
  // オフライン固定モード(#465): ON なら起動時からローカルに固定（クラウド設定が残っていても無視）。
  if (offlineMode) {
    if (!LOCAL_PROVIDERS.includes(provider)) provider = "ollama";
    sttProvider = "local";
  }

  let customStyles: CustomStyle[] = [];
  try {
    customStyles = JSON.parse(ls.getItem("customStyles") || "[]");
  } catch {
    // 破損した JSON は空扱い（初期値 [] を維持）。
  }

  let refineStyle = ls.getItem("refineStyle") || "structured";
  if (!isValidRefineStyle(refineStyle, customStyles)) refineStyle = "structured";

  const s: AppSettings = {
    // enum 的な値はクランプ（破損耐性 / validateSettings 相当）。
    provider: clampProvider(provider),
    resolvedModel,
    recordShortcut: ls.getItem("recordShortcut") || DEFAULT_SHORTCUT,
    recordMode: clampOneOf(
      ls.getItem("recordMode") === "momentary" ? "momentary" : "toggle",
      ["toggle", "momentary"] as const,
      "toggle",
    ),
    includeTimestamps: ls.getItem("includeTimestamps") !== "false",
    autoPipeline: ls.getItem("autoPipeline") === "true",
    keepText: ls.getItem("keepText") !== "false",
    saveAudio: ls.getItem("saveAudio") === "true",
    audioFormat: clampOneOf(ls.getItem("audioFormat") || "opus", ["opus", "wav"] as const, "opus"),
    saveDir: ls.getItem("saveDir") || "",
    outputFormat: clampOneOf(ls.getItem("outputFormat") || "txt", ["txt", "md"] as const, "txt"),
    refineStyle,
    translateOutput: ls.getItem("translateOutput") === "true",
    outputLang: ls.getItem("outputLang") || localeDefault,
    openaiBaseUrl: ls.getItem("openaiBaseUrl") || "",
    sttProvider: clampSttProvider(sttProvider),
    sttUseGpu,
    offlineMode,
    sttModel: ls.getItem("sttModel") || "",
    sttAzureResource: ls.getItem("sttAzureResource") || "",
    // ローカルwhisperの既定: 日本語UIは large-v3-turbo、他は汎用 base（ADR-0025・実測に基づく見直し）。
    // 旧既定 kotoba-q5 は実録音で「長尺の末尾欠落＋自発発話で崩壊」が確認され既定から降格（ADR-0021改訂）。
    // モデルは非同梱・初回自動DL（tip_model_download で明示）。保存済みは優先。
    whisperModel:
      ls.getItem("whisperModel") || (localeDefault === "ja" ? "large-v3-turbo" : "base"),
    customStyles,
    awsRegion: ls.getItem("awsRegion") || "us-east-1",
    awsWorkspaceId: ls.getItem("awsWorkspaceId") || "",
    awsAuthMode: clampOneOf(
      (ls.getItem("awsAuthMode") as "sigv4" | "apikey") || "sigv4",
      ["sigv4", "apikey"] as const,
      "sigv4",
    ),
    bedrockModel: ls.getItem("bedrockModel") || "",
    taskbarWidget: ls.getItem("taskbarWidget") !== "false",
    inputDevice: ls.getItem("inputDevice") || "",
    inputDeviceKind: ls.getItem("inputDeviceKind") || "input",
    // 習慣ナッジ(#58)は opt-in（既定OFF）。プライバシー/簡便さ優先で明示的にONにした人だけ促す。
    nudgeEnabled: ls.getItem("nudgeEnabled") === "true",
  };
  // スキーマ版を記録（検証を通過した証跡 / ADR-0017）。
  ls.setItem("settingsVersion", String(SETTINGS_VERSION));
  return s;
}

/**
 * 設定フォームの値を localStorage へ書き戻す（秘密情報・resolvedModel・customStyles を除く。
 * それらは別経路 keyring / resolveCurrentModel / persistCustomStyles で永続化される）。
 */
export function writeSettings(s: AppSettings): void {
  const ls = localStorage;
  ls.setItem("provider", s.provider);
  ls.setItem("includeTimestamps", String(s.includeTimestamps));
  ls.setItem("autoPipeline", String(s.autoPipeline));
  ls.setItem("keepText", String(s.keepText));
  ls.setItem("saveAudio", String(s.saveAudio));
  ls.setItem("audioFormat", s.audioFormat);
  ls.setItem("saveDir", s.saveDir);
  ls.setItem("outputFormat", s.outputFormat);
  ls.setItem("refineStyle", s.refineStyle);
  ls.setItem("translateOutput", String(s.translateOutput));
  ls.setItem("outputLang", s.outputLang);
  ls.setItem("openaiBaseUrl", s.openaiBaseUrl);
  ls.setItem("sttProvider", s.sttProvider);
  ls.setItem("sttUseGpu", String(s.sttUseGpu));
  ls.setItem("sttModel", s.sttModel);
  ls.setItem("sttAzureResource", s.sttAzureResource);
  ls.setItem("whisperModel", s.whisperModel);
  ls.setItem("awsRegion", s.awsRegion);
  ls.setItem("awsWorkspaceId", s.awsWorkspaceId);
  ls.setItem("awsAuthMode", s.awsAuthMode);
  ls.setItem("bedrockModel", s.bedrockModel);
  ls.setItem("taskbarWidget", String(s.taskbarWidget));
  ls.setItem("recordMode", s.recordMode);
  ls.setItem("inputDevice", s.inputDevice);
  ls.setItem("inputDeviceKind", s.inputDeviceKind);
  ls.setItem("nudgeEnabled", String(s.nudgeEnabled));
}
