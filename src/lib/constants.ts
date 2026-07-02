// SSOT(単一の正): プロバイダ定義・整形スタイル・各種定数を一元管理する（#401 Phase0）。
// App.svelte 等に散在していた定義をここへ集約し、二重定義・修正漏れを防ぐ。
// i18n(#401本体) でラベル類を翻訳カタログへ移す際の足場でもある。
//
// 注意: REFINE_STYLES の desc・各プロバイダ識別子は Rust 側(refine.rs/stt.rs)の
// 指示文・分岐と意味的に一致させること（将来は Rust 側も同一の契約から導出する）。

// ===== 言語(UI言語 + 整形出力言語の共通SSOT / #401 #453) =====
// UIロケール(翻訳カタログ有)と「整形出力言語(翻訳)」の選択肢を単一定義に統合する。
// ui:true = UI表示言語として提供(カタログ有)。全件は出力言語の選択肢(LLMが訳せる任意言語)。
// en = LLMへの言語指示に使う英語名(Vietnamese等・UIカタログが無くても出力言語にできる)。
export interface Language {
  code: string;
  /** 自言語表記(endonym)。UIの表示用。 */
  label: string;
  /** プロンプトに差し込む英語名(LLMが確実に解釈できる)。 */
  en: string;
  /** UI表示言語として提供するか(翻訳カタログの有無)。 */
  ui: boolean;
}

export const LANGUAGES: Language[] = [
  { code: "ja", label: "日本語", en: "Japanese", ui: true },
  { code: "en", label: "English", en: "English", ui: true },
  { code: "zh", label: "简体中文", en: "Chinese (Simplified)", ui: true },
  { code: "es", label: "Español", en: "Spanish", ui: true },
  { code: "ko", label: "한국어", en: "Korean", ui: false },
  { code: "vi", label: "Tiếng Việt", en: "Vietnamese", ui: false },
  { code: "fr", label: "Français", en: "French", ui: false },
  { code: "de", label: "Deutsch", en: "German", ui: false },
  { code: "pt", label: "Português", en: "Portuguese", ui: false },
  { code: "it", label: "Italiano", en: "Italian", ui: false },
  { code: "ru", label: "Русский", en: "Russian", ui: false },
  { code: "id", label: "Bahasa Indonesia", en: "Indonesian", ui: false },
  { code: "th", label: "ไทย", en: "Thai", ui: false },
  { code: "hi", label: "हिन्दी", en: "Hindi", ui: false },
  { code: "ar", label: "العربية", en: "Arabic", ui: false },
];

/** UI表示言語(翻訳カタログ有)のコード。i18n の SUPPORTED_LOCALES の唯一の源。 */
export const UI_LOCALES: string[] = LANGUAGES.filter((l) => l.ui).map((l) => l.code);

/** コード→プロンプト用の英語名。未知は undefined。 */
export function languageEnglishName(code: string): string | undefined {
  return LANGUAGES.find((l) => l.code === code)?.en;
}

// ===== 整形(LLM)プロバイダ =====
export type Provider = "gemini" | "anthropic" | "openai" | "ollama" | "bedrock" | "claude-aws";

export const ALL_PROVIDERS: Provider[] = [
  "gemini",
  "anthropic",
  "openai",
  "ollama",
  "bedrock",
  "claude-aws",
];

export const PROVIDER_LABELS: Record<Provider, string> = {
  gemini: "Gemini",
  anthropic: "Anthropic (Claude)",
  openai: "OpenAI",
  ollama: "ローカル (Ollama)",
  bedrock: "AWS Bedrock",
  "claude-aws": "Claude Platform on AWS",
};

// 鍵不要のローカルプロバイダ（端末内完結＝差別化「ローカルプライバシー」/ S3.4）。
export const LOCAL_PROVIDERS: Provider[] = ["ollama"];

// AWS系(region＋認証が必要。SigV4 or APIキー / ADR-0011)。
export const AWS_PROVIDERS: Provider[] = ["bedrock", "claude-aws"];

// モデルは実行時に各社のモデル一覧APIから最新ミドルレンジを解決する（ビルド時固定にしない）。
// 取得失敗時のフォールバック表示（ローリングlatestエイリアス優先 / ADR-0007）。
// AWSはモデル一覧APIが別系統のため自動解決せず、フォールバック/手入力を使う。
export const FALLBACK_MODELS: Record<Provider, string> = {
  gemini: "gemini-flash-latest",
  anthropic: "claude-sonnet-4-6",
  openai: "gpt-4o",
  ollama: "llama3.1",
  bedrock: "anthropic.claude-sonnet-4-6",
  "claude-aws": "claude-sonnet-4-6",
};

export const KEY_PLACEHOLDERS: Record<Provider, string> = {
  gemini: "AIza...",
  anthropic: "sk-ant-...",
  openai: "sk-...",
  ollama: "",
  bedrock: "Bedrock APIキー(Bearer) ※SigV4時は不要",
  "claude-aws": "Anthropic APIキー ※SigV4時は不要",
};

// ===== 文字起こし(STT)プロバイダ =====
export type SttProvider = "local" | "groq" | "openai" | "deepgram" | "azure";

export const STT_LABELS: Record<SttProvider, string> = {
  local: "ローカル (whisper・端末内完結)",
  groq: "Groq (whisper-large-v3-turbo・高速・安価)",
  openai: "OpenAI (gpt-4o-transcribe)",
  deepgram: "Deepgram (nova-3)",
  azure: "Azure AI Speech (fast transcription)",
};

export const STT_CLOUD: SttProvider[] = ["groq", "openai", "deepgram", "azure"];

export const STT_KEY_PLACEHOLDERS: Record<string, string> = {
  groq: "gsk_...",
  openai: "sk-...",
  deepgram: "Deepgram APIキー",
  azure: "Azure Speech リソースキー",
};

export const STT_MODEL_PLACEHOLDERS: Record<string, string> = {
  groq: "whisper-large-v3-turbo",
  openai: "gpt-4o-transcribe",
  deepgram: "nova-3",
  azure: "（不要）",
};

// ===== 整形スタイル =====
// desc は各モードの短い解説(設定のtips・処理画面のツールチップに使う。refine.rs の指示と一致)。
export type RefineStyle = { value: string; label: string; desc: string };

export const REFINE_STYLES: RefineStyle[] = [
  {
    value: "structured",
    label: "構造化",
    desc: "見出しと箇条書きで要点を整理。ニュアンスは残します（既定）。",
  },
  {
    value: "verbatim",
    label: "逐語",
    desc: "言い淀みや繰り返しも極力そのまま。最小限の読みやすさ調整のみ。",
  },
  { value: "summary", label: "要約", desc: "全体を短く要約し、重要な要点を3〜5個に絞ります。" },
  // ブレストは vision のコア価値「逐語⇄要約⇄ブレストを行き来」に含まれる整形スタイル。
  // 対話型Q&A(ツールの役割外)ではなく「発想を広げる再整形」であることを desc で明確化する(#514)。
  {
    value: "brainstorm",
    label: "発想ひろげ",
    desc: "内容から問い・観点・次の一歩を広げる“再整形”。対話ではなく一度きりの書き換えです。",
  },
];

// ユーザー定義のカスタム整形パターン（S3.3）。value は "custom:<id>"。
export type CustomStyle = { id: string; label: string; instruction: string };

// ===== その他の定数 =====
// 解決済みモデルのキャッシュ寿命（24時間）。これを過ぎたら再取得する。
export const MODEL_TTL_MS = 24 * 60 * 60 * 1000;

// 横断発見でAIへ渡す過去エントリの上限（プロンプト肥大を避ける / S4.3）。
export const DISCOVERY_MAX = 30;

// 録音トグルの既定グローバルショートカット（ADR-0014）。
export const DEFAULT_SHORTCUT = "CommandOrControl+Shift+R";

// 入力ファイルのサイズ上限(MB)。Rust側 lib.rs::MAX_INPUT_BYTES と一致させること（#397）。
export const MAX_INPUT_MB = 500;

// 対応する音声ファイル形式（ファイル選択フィルタ・UI通知で共用 / S1.6）。
export const SUPPORTED_AUDIO_EXTS = ["mp3", "wav", "m4a", "flac", "ogg", "aac"];
