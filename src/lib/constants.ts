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

// 表示文言は翻訳カタログ(ja/en/zh/es)に一本化し、ここではキー名のみを持つ(SSOT / #401)。
// 使用側は $_(PROVIDER_LABEL_KEYS[p]) 等で解決する（ハードコード日本語をUIに露出させない）。
export const PROVIDER_LABEL_KEYS: Record<Provider, string> = Object.fromEntries(
  ALL_PROVIDERS.map((p) => [p, `catalog.providers.${p}`]),
) as Record<Provider, string>;

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

// APIキー入力のプレースホルダ（i18nキー。ollamaは鍵不要のためUIには出ない）。
export const KEY_PLACEHOLDER_KEYS: Record<Provider, string> = Object.fromEntries(
  ALL_PROVIDERS.map((p) => [p, `catalog.key_ph.${p}`]),
) as Record<Provider, string>;

// ===== 文字起こし(STT)プロバイダ =====
export type SttProvider = "local" | "groq" | "openai" | "deepgram" | "azure";

/** STTプロバイダの表示順(SSOT)。ラベルは catalog.stt.* で解決する。 */
export const STT_PROVIDERS: SttProvider[] = ["local", "groq", "openai", "deepgram", "azure"];

export const STT_LABEL_KEYS: Record<SttProvider, string> = Object.fromEntries(
  STT_PROVIDERS.map((p) => [p, `catalog.stt.${p}`]),
) as Record<SttProvider, string>;

export const STT_CLOUD: SttProvider[] = ["groq", "openai", "deepgram", "azure"];

export const STT_KEY_PLACEHOLDER_KEYS: Record<string, string> = Object.fromEntries(
  STT_CLOUD.map((p) => [p, `catalog.stt_key_ph.${p}`]),
);

export const STT_MODEL_PLACEHOLDER_KEYS: Record<string, string> = Object.fromEntries(
  STT_CLOUD.map((p) => [p, `catalog.stt_model_ph.${p}`]),
);

// ===== 整形スタイル =====
// label/desc は翻訳カタログ(catalog.styles.*)のキー。desc は各モードの短い解説
// (設定のtips・処理画面のツールチップに使う。refine.rs の指示と一致)。
export type RefineStyle = { value: string; labelKey: string; descKey: string };

// ブレストは vision のコア価値「逐語⇄要約⇄ブレストを行き来」に含まれる整形スタイル。
// 対話型Q&A(ツールの役割外)ではなく「発想を広げる再整形」であることを desc で明確化する(#514)。
export const REFINE_STYLES: RefineStyle[] = ["structured", "verbatim", "summary", "brainstorm"].map(
  (value) => ({
    value,
    labelKey: `catalog.styles.${value}.label`,
    descKey: `catalog.styles.${value}.desc`,
  }),
);

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
export const SUPPORTED_AUDIO_EXTS = ["mp3", "wav", "m4a", "flac", "ogg", "opus", "aac"];
