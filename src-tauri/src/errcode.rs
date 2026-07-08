// 安定エラーコード（#462 i18n Phase2）。
//
// バックエンドはユーザー向けエラーを日本語ハードコードで返さず、言語非依存の
// 安定コード `E_XXX` を返す。技術詳細が必要な場合は区切り文字 U+001F に続けて付す。
// フロント（src/lib/errors.ts の errorText）が `errors.rust.<CODE>` カタログ
// （ja/en/zh/es）へ写像してローカライズする。これにより非日本語UIでバックエンドの
// 日本語が露出しない。
//
// 命名規約: `E_` + 大文字/数字/アンダースコア。コード自体は SSOT（下の ALL に集約）。

/// コードと技術詳細を区切る制御文字（U+001F Unit Separator）。errors.ts の ERR_CODE_SEP と一致。
pub const SEP: char = '\u{1f}';

/// コード＋技術詳細を `"CODE\u{1f}detail"` 形式で整形する。
/// detail はライブラリ由来の技術文字列（多くは英語）で、翻訳対象外。
pub fn ec(code: &str, detail: impl std::fmt::Display) -> String {
    format!("{code}{SEP}{detail}")
}

// ---- lib.rs ----
pub const E_LOCK_STT: &str = "E_LOCK_STT";
pub const E_NO_DOCUMENT_DIR: &str = "E_NO_DOCUMENT_DIR";
pub const E_FILE_TOO_LARGE: &str = "E_FILE_TOO_LARGE";
pub const E_UNSUPPORTED_AUDIO_EXT: &str = "E_UNSUPPORTED_AUDIO_EXT";
pub const E_FILE_OPEN: &str = "E_FILE_OPEN";
pub const E_NOT_RECORDING: &str = "E_NOT_RECORDING";
pub const E_EMPTY_RECORDING: &str = "E_EMPTY_RECORDING";
pub const E_JOURNAL_DIR: &str = "E_JOURNAL_DIR";
pub const E_FILE_MANAGER: &str = "E_FILE_MANAGER";
pub const E_LOCK_SETTINGS: &str = "E_LOCK_SETTINGS";
pub const E_LOCK_RECORD_STATE: &str = "E_LOCK_RECORD_STATE";
pub const E_LOCK_JOB_STATE: &str = "E_LOCK_JOB_STATE";
pub const E_ALREADY_RECORDING: &str = "E_ALREADY_RECORDING";
pub const E_TEXT_READ: &str = "E_TEXT_READ";
pub const E_SHORTCUT_PARSE: &str = "E_SHORTCUT_PARSE";
pub const E_SHORTCUT_REGISTER: &str = "E_SHORTCUT_REGISTER";
pub const E_KEYRING_INIT: &str = "E_KEYRING_INIT";
pub const E_SECRET_SAVE: &str = "E_SECRET_SAVE";
pub const E_SECRET_DELETE: &str = "E_SECRET_DELETE";

// ---- record.rs ----
pub const E_REC_BUFFER: &str = "E_REC_BUFFER";
pub const E_LOOPBACK_UNSUPPORTED: &str = "E_LOOPBACK_UNSUPPORTED";
pub const E_MIXED_UNSUPPORTED: &str = "E_MIXED_UNSUPPORTED";
pub const E_NO_INPUT_DEVICE: &str = "E_NO_INPUT_DEVICE";
pub const E_INPUT_CONFIG: &str = "E_INPUT_CONFIG";
pub const E_UNSUPPORTED_FORMAT: &str = "E_UNSUPPORTED_FORMAT";
pub const E_STREAM_BUILD: &str = "E_STREAM_BUILD";
pub const E_REC_START: &str = "E_REC_START";
pub const E_REC_THREAD_INIT: &str = "E_REC_THREAD_INIT";
pub const E_COM_INIT: &str = "E_COM_INIT";
pub const E_AUDIO_ENUM: &str = "E_AUDIO_ENUM";
pub const E_OUTPUT_DEVICE: &str = "E_OUTPUT_DEVICE";
pub const E_AUDIO_CLIENT: &str = "E_AUDIO_CLIENT";
pub const E_MIX_FORMAT: &str = "E_MIX_FORMAT";
pub const E_DEVICE_PERIOD: &str = "E_DEVICE_PERIOD";
pub const E_LOOPBACK_INIT: &str = "E_LOOPBACK_INIT";
pub const E_EVENT_HANDLE: &str = "E_EVENT_HANDLE";
pub const E_CAPTURE_CLIENT: &str = "E_CAPTURE_CLIENT";
pub const E_LOOPBACK_START: &str = "E_LOOPBACK_START";
pub const E_LOOPBACK_THREAD_INIT: &str = "E_LOOPBACK_THREAD_INIT";

// ---- stt.rs ----
pub const E_AUDIO_PROBE: &str = "E_AUDIO_PROBE";
pub const E_NO_CODEC_PARAMS: &str = "E_NO_CODEC_PARAMS";
pub const E_DECODER_BUILD: &str = "E_DECODER_BUILD";
pub const E_PACKET_READ: &str = "E_PACKET_READ";
pub const E_DECODE: &str = "E_DECODE";
pub const E_STT_MODEL_LOAD: &str = "E_STT_MODEL_LOAD";
pub const E_WAV_BUILD: &str = "E_WAV_BUILD";
pub const E_WAV_WRITE: &str = "E_WAV_WRITE";
pub const E_WAV_FINALIZE: &str = "E_WAV_FINALIZE";
pub const E_CLOUD_STT_NO_KEY: &str = "E_CLOUD_STT_NO_KEY";
pub const E_CLOUD_STT_HTTP: &str = "E_CLOUD_STT_HTTP";
pub const E_CLOUD_STT_STATUS: &str = "E_CLOUD_STT_STATUS";
pub const E_CLOUD_STT_PARSE: &str = "E_CLOUD_STT_PARSE";
pub const E_AZURE_NO_RESOURCE: &str = "E_AZURE_NO_RESOURCE";
pub const E_NO_AUDIO_TRACK: &str = "E_NO_AUDIO_TRACK";
pub const E_STT_MODEL_PATH: &str = "E_STT_MODEL_PATH";

// ---- model.rs ----
pub const E_MODEL_DOWNLOAD: &str = "E_MODEL_DOWNLOAD";
pub const E_MODEL_SIZE_MISMATCH: &str = "E_MODEL_SIZE_MISMATCH";
pub const E_MODEL_SHA256_MISMATCH: &str = "E_MODEL_SHA256_MISMATCH";

// ---- aws_sign.rs ----
pub const E_SIGV4_PARAMS: &str = "E_SIGV4_PARAMS";
pub const E_SIGV4_SIGNABLE: &str = "E_SIGV4_SIGNABLE";
pub const E_SIGV4_SIGN: &str = "E_SIGV4_SIGN";
pub const E_HTTP_BUILD: &str = "E_HTTP_BUILD";

// ---- audio_save.rs ----
pub const E_WAV_CREATE: &str = "E_WAV_CREATE";
pub const E_WAV_EXPORT: &str = "E_WAV_EXPORT";
pub const E_EMPTY_AUDIO: &str = "E_EMPTY_AUDIO";
pub const E_OPUS_ENCODER: &str = "E_OPUS_ENCODER";
pub const E_OPUS_HEAD: &str = "E_OPUS_HEAD";
pub const E_OPUS_TAGS: &str = "E_OPUS_TAGS";
pub const E_OPUS_ENCODE: &str = "E_OPUS_ENCODE";
pub const E_OPUS_PACKET: &str = "E_OPUS_PACKET";
pub const E_OPUS_EXPORT: &str = "E_OPUS_EXPORT";

// ---- refine.rs（整形エンジンのユーザー向けエラー。プロンプト文字列は対象外）----
pub const E_REFINE_EMPTY_INPUT: &str = "E_REFINE_EMPTY_INPUT";
pub const E_REFINE_HTTP: &str = "E_REFINE_HTTP";
pub const E_REFINE_EMPTY_RESULT: &str = "E_REFINE_EMPTY_RESULT";
pub const E_REFINE_NO_KEY: &str = "E_REFINE_NO_KEY";
pub const E_REFINE_GEMINI_NO_KEY: &str = "E_REFINE_GEMINI_NO_KEY";
pub const E_REFINE_ANTHROPIC_NO_KEY: &str = "E_REFINE_ANTHROPIC_NO_KEY";
pub const E_REFINE_OPENAI_NO_KEY: &str = "E_REFINE_OPENAI_NO_KEY";
pub const E_REFINE_AWS_NO_KEYS: &str = "E_REFINE_AWS_NO_KEYS";
pub const E_REFINE_AWS_NO_REGION: &str = "E_REFINE_AWS_NO_REGION";
pub const E_REFINE_MODELS_HTTP: &str = "E_REFINE_MODELS_HTTP";
pub const E_REFINE_NO_SONNET: &str = "E_REFINE_NO_SONNET";
pub const E_REFINE_OLLAMA_CONN: &str = "E_REFINE_OLLAMA_CONN";
pub const E_REFINE_AWS_CONFIG: &str = "E_REFINE_AWS_CONFIG";
pub const E_REFINE_NO_OLLAMA_MODEL: &str = "E_REFINE_NO_OLLAMA_MODEL";
pub const E_REFINE_MODELS_PARSE: &str = "E_REFINE_MODELS_PARSE";
pub const E_REFINE_NO_OPENAI_MID: &str = "E_REFINE_NO_OPENAI_MID";
pub const E_REFINE_NO_FLASH: &str = "E_REFINE_NO_FLASH";

// ---- status イベント（エラーでない進行状況のUI文言。S_ プレフィックス）----
// lib.rs が emit し、フロント（src/lib/status.ts）が status.rust.<CODE> で解決する。
pub const S_MODEL_DOWNLOAD_PCT: &str = "S_MODEL_DOWNLOAD_PCT";
pub const S_MODEL_DOWNLOAD_MB: &str = "S_MODEL_DOWNLOAD_MB";
pub const S_TRANSCRIBING_CLOUD: &str = "S_TRANSCRIBING_CLOUD";
pub const S_TRANSCRIBING: &str = "S_TRANSCRIBING";
pub const S_LOADING_AUDIO: &str = "S_LOADING_AUDIO";
pub const S_AUDIO_SAVE_FAILED: &str = "S_AUDIO_SAVE_FAILED";

/// status コードの SSOT（一意性テストとフロント側パリティ検証の基準）。
pub const ALL_STATUS: &[&str] = &[
    S_MODEL_DOWNLOAD_PCT,
    S_MODEL_DOWNLOAD_MB,
    S_TRANSCRIBING_CLOUD,
    S_TRANSCRIBING,
    S_LOADING_AUDIO,
    S_AUDIO_SAVE_FAILED,
];

/// 全コードの SSOT（一意性テストとフロント側パリティ検証の基準）。
pub const ALL: &[&str] = &[
    E_LOCK_STT,
    E_NO_DOCUMENT_DIR,
    E_FILE_TOO_LARGE,
    E_UNSUPPORTED_AUDIO_EXT,
    E_FILE_OPEN,
    E_NOT_RECORDING,
    E_EMPTY_RECORDING,
    E_JOURNAL_DIR,
    E_FILE_MANAGER,
    E_LOCK_SETTINGS,
    E_LOCK_RECORD_STATE,
    E_ALREADY_RECORDING,
    E_TEXT_READ,
    E_SHORTCUT_PARSE,
    E_SHORTCUT_REGISTER,
    E_KEYRING_INIT,
    E_SECRET_SAVE,
    E_SECRET_DELETE,
    E_REC_BUFFER,
    E_LOOPBACK_UNSUPPORTED,
    E_MIXED_UNSUPPORTED,
    E_NO_INPUT_DEVICE,
    E_INPUT_CONFIG,
    E_UNSUPPORTED_FORMAT,
    E_STREAM_BUILD,
    E_REC_START,
    E_REC_THREAD_INIT,
    E_COM_INIT,
    E_AUDIO_ENUM,
    E_OUTPUT_DEVICE,
    E_AUDIO_CLIENT,
    E_MIX_FORMAT,
    E_DEVICE_PERIOD,
    E_LOOPBACK_INIT,
    E_EVENT_HANDLE,
    E_CAPTURE_CLIENT,
    E_LOOPBACK_START,
    E_LOOPBACK_THREAD_INIT,
    E_AUDIO_PROBE,
    E_NO_CODEC_PARAMS,
    E_DECODER_BUILD,
    E_PACKET_READ,
    E_DECODE,
    E_STT_MODEL_LOAD,
    E_WAV_BUILD,
    E_WAV_WRITE,
    E_WAV_FINALIZE,
    E_CLOUD_STT_NO_KEY,
    E_CLOUD_STT_HTTP,
    E_CLOUD_STT_STATUS,
    E_CLOUD_STT_PARSE,
    E_AZURE_NO_RESOURCE,
    E_NO_AUDIO_TRACK,
    E_STT_MODEL_PATH,
    E_MODEL_DOWNLOAD,
    E_MODEL_SIZE_MISMATCH,
    E_MODEL_SHA256_MISMATCH,
    E_SIGV4_PARAMS,
    E_SIGV4_SIGNABLE,
    E_SIGV4_SIGN,
    E_HTTP_BUILD,
    E_WAV_CREATE,
    E_WAV_EXPORT,
    E_EMPTY_AUDIO,
    E_OPUS_ENCODER,
    E_OPUS_HEAD,
    E_OPUS_TAGS,
    E_OPUS_ENCODE,
    E_OPUS_PACKET,
    E_OPUS_EXPORT,
    E_REFINE_EMPTY_INPUT,
    E_REFINE_HTTP,
    E_REFINE_EMPTY_RESULT,
    E_REFINE_NO_KEY,
    E_REFINE_GEMINI_NO_KEY,
    E_REFINE_ANTHROPIC_NO_KEY,
    E_REFINE_OPENAI_NO_KEY,
    E_REFINE_AWS_NO_KEYS,
    E_REFINE_AWS_NO_REGION,
    E_REFINE_MODELS_HTTP,
    E_REFINE_NO_SONNET,
    E_REFINE_OLLAMA_CONN,
    E_REFINE_AWS_CONFIG,
    E_REFINE_NO_OLLAMA_MODEL,
    E_REFINE_MODELS_PARSE,
    E_REFINE_NO_OPENAI_MID,
    E_REFINE_NO_FLASH,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn codes_are_unique() {
        let set: HashSet<&&str> = ALL.iter().collect();
        assert_eq!(set.len(), ALL.len(), "エラーコードに重複がある");
        let sset: HashSet<&&str> = ALL_STATUS.iter().collect();
        assert_eq!(sset.len(), ALL_STATUS.len(), "statusコードに重複がある");
    }

    #[test]
    fn codes_follow_naming_convention() {
        for c in ALL {
            assert!(c.starts_with("E_"), "{c} は E_ で始まっていない");
            assert!(
                c.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_'),
                "{c} は大文字/数字/_ 以外を含む"
            );
            assert!(!c.contains(SEP), "{c} が区切り文字を含む");
        }
        for c in ALL_STATUS {
            assert!(c.starts_with("S_"), "{c} は S_ で始まっていない");
            assert!(
                c.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_'),
                "{c} は大文字/数字/_ 以外を含む"
            );
            assert!(!c.contains(SEP), "{c} が区切り文字を含む");
        }
    }

    #[test]
    fn ec_formats_code_and_detail() {
        let s = ec(E_STT_MODEL_LOAD, "no such file");
        assert_eq!(s, format!("E_STT_MODEL_LOAD{SEP}no such file"));
    }
}
