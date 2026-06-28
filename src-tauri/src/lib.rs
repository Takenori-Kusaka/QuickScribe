// QuickScribe — Walking Skeleton (Phase 1)
//
// このフェーズの責務は「常駐(トレイ) + ウィンドウ + 録音トグル + グローバルホットキー +
// 指定フォルダへの保存導線」を貫通させること。文字起こし(whisper)・整形(LLM)・
// システム音声ループバック・デバイス切替・Stream Deck連携は後続の縦切りで追加する
// (ADR-0006 によりスコープからは外さない)。

pub mod audio_save;
// AWS SigV4署名(Bedrock / Claude Platform on AWS の整形プロバイダ用 / ADR-0011)。
pub mod aws_sign;
pub mod model;
pub mod record;
pub mod refine;
pub mod stt;
// 保管庫エントリの一覧・解析（S4.3 Phase1: アプリ内の横断導線）。
pub mod vault;
// Windows タスクバーのサムネイルツールバー/オーバーレイ。Windowsのみ。
#[cfg(windows)]
mod taskbar;
// Windows タスクバーに埋め込む録音ウィジェット（常時表示の操作ボタン）。Windowsのみ。
#[cfg(windows)]
mod taskbar_widget;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// メモ内容を保存フォルダ(既定: ドキュメント/QuickScribe)へ書き出す。
///
/// 既存名と衝突しない一意なファイル名を返す（S4.1 R5・非破壊保存）。
/// 衝突時は `stem-2.ext`, `stem-3.ext`… を試す。`exists` は候補名の存在判定（テスト容易性のため注入）。
fn next_unique_name(stem: &str, ext: &str, exists: impl Fn(&str) -> bool) -> String {
    let first = format!("{stem}.{ext}");
    if !exists(&first) {
        return first;
    }
    let mut n = 2u32;
    loop {
        let cand = format!("{stem}-{n}.{ext}");
        if !exists(&cand) {
            return cand;
        }
        n += 1;
    }
}

/// 保存に関する設定（保存先・音声保存可否/形式・文字起こしテキスト保持・出力形式）。
/// フロントの設定から set_save_settings で更新し、保存系コマンドが参照する。
#[derive(Clone)]
struct SaveSettings {
    /// 保存先フォルダ。None は既定(ドキュメント/QuickScribe)。
    save_dir: Option<String>,
    /// 録音音声を保存するか。
    save_audio: bool,
    /// 保存形式("wav"。今後 "opus")。
    audio_format: String,
    /// 文字起こしテキスト(.txt)を保存するか。
    keep_text: bool,
    /// エントリの出力形式("txt"=本文のみ / "md"=YAMLフロントマター付きMarkdown)。S4.2。
    output_format: String,
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            save_dir: None,
            save_audio: false,
            audio_format: "wav".to_string(),
            keep_text: true,
            output_format: "txt".to_string(),
        }
    }
}

/// エントリのメタデータ(S4.2/S4.3 / Markdownフロントマター用)。
struct DocMeta<'a> {
    /// 種別: "transcript"(文字起こし) / "refined"(整形) / "note"(任意保存)。
    kind: &'a str,
    /// 整形スタイル(refined のときのみ Some)。
    style: Option<&'a str>,
    /// 内省タグ(S4.3)。空なら付与しない。
    tags: &'a [String],
}

/// 出力形式から拡張子を返す(純粋)。"md" 以外は "txt"。
fn doc_extension(format: &str) -> &'static str {
    if format.trim().eq_ignore_ascii_case("md") {
        "md"
    } else {
        "txt"
    }
}

/// YAMLフロントマターの値を1行・安全に整える(改行を空白化、前後空白除去)。
/// コロン等を含んでもダブルクォートで囲むため曖昧にならない。
fn yaml_scalar(v: &str) -> String {
    let one_line = v.replace(['\n', '\r'], " ");
    let escaped = one_line.replace('"', "'");
    format!("\"{}\"", escaped.trim())
}

/// エントリのスキーマ版（ADR-0017 / S4.4）。md フロントマターに刻む。
const CURRENT_ENTRY_SCHEMA: u32 = 1;

/// 出力形式に応じてエントリ本文を組み立てる(純粋・テスト対象)。
/// md は schema/created/type(/style/tags) のYAMLフロントマターを本文の前に付す。
/// txt はタグがあれば末尾に `Tags: a, b` 行を付す(形式に依らずタグを残す / S4.3)。
fn build_document(content: &str, format: &str, created_iso: &str, meta: &DocMeta) -> String {
    if doc_extension(format) != "md" {
        // プレーンテキスト: 本文＋(タグがあれば)末尾にタグ行。
        if meta.tags.is_empty() {
            return content.to_string();
        }
        return format!("{}\n\nTags: {}", content, meta.tags.join(", "));
    }
    let mut fm = String::from("---\n");
    // エントリスキーマ版（S4.4 / ADR-0017）。将来の非破壊移行のための版マーカー。
    fm.push_str(&format!("schema: {CURRENT_ENTRY_SCHEMA}\n"));
    fm.push_str(&format!("created: {}\n", yaml_scalar(created_iso)));
    fm.push_str(&format!("type: {}\n", yaml_scalar(meta.kind)));
    if let Some(style) = meta.style {
        fm.push_str(&format!("style: {}\n", yaml_scalar(style)));
    }
    if !meta.tags.is_empty() {
        let items: Vec<String> = meta.tags.iter().map(|t| yaml_scalar(t)).collect();
        fm.push_str(&format!("tags: [{}]\n", items.join(", ")));
    }
    fm.push_str("---\n\n");
    fm.push_str(content);
    fm
}

#[derive(Default)]
struct AppSettings {
    inner: std::sync::Mutex<SaveSettings>,
}

/// 文字起こし(STT)の設定（S2.4）。provider 既定は "local"。クラウド時のみ model/api_key を使う。
/// 鍵は keyring 由来をフロントが起動時に注入する（メモリ内のみ・永続化しない）。
#[derive(Clone, Default)]
struct SttSettings {
    provider: String,
    model: String,
    api_key: String,
    azure_resource: String,
}

#[derive(Default)]
struct SttState {
    inner: std::sync::Mutex<SttSettings>,
}

/// 現在のSTT設定のスナップショットを返す（未設定なら provider="local" 相当）。
fn current_stt_settings(app: &tauri::AppHandle) -> SttSettings {
    app.state::<SttState>()
        .inner
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
}

/// フロントのSTT設定を反映する（S2.4）。クラウド選択時の provider/model/api_key を保持。
#[tauri::command]
fn set_stt_settings(
    state: tauri::State<'_, SttState>,
    provider: String,
    model: String,
    api_key: String,
    azure_resource: Option<String>,
) -> Result<(), String> {
    let mut s = state
        .inner
        .lock()
        .map_err(|_| "STT設定のロックに失敗".to_string())?;
    s.provider = provider;
    s.model = model;
    s.api_key = api_key;
    s.azure_resource = azure_resource.unwrap_or_default();
    Ok(())
}

/// 現在の保存設定のスナップショットを返す。
fn current_settings(app: &tauri::AppHandle) -> SaveSettings {
    app.state::<AppSettings>()
        .inner
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
}

/// 保存先フォルダを解決する（未設定なら ドキュメント/QuickScribe）。
fn resolve_save_dir(settings: &SaveSettings) -> Result<std::path::PathBuf, String> {
    if let Some(d) = settings.save_dir.as_ref().filter(|d| !d.trim().is_empty()) {
        return Ok(std::path::PathBuf::from(d));
    }
    Ok(dirs::document_dir()
        .ok_or_else(|| "ドキュメントフォルダが見つかりません".to_string())?
        .join("QuickScribe"))
}

/// タイムスタンプ付きファイル名で dir 配下にエントリを書き出し、パスを返す（S4.1/S4.2）。
/// 出力形式(txt/md)とメタデータに従って本文を組み立て、同一秒の衝突は一意名にする（非破壊）。
/// 種別ごとのファイル名プレフィックス（純粋）。生の文字起こしと整形済みを名前で見分けられるようにする。
/// transcript=生の文字起こし / refined=整形済み / note=その他。
fn filename_prefix(kind: &str) -> &'static str {
    match kind {
        "transcript" => "transcript",
        "refined" => "refined",
        _ => "note",
    }
}

fn save_document(
    dir: &std::path::Path,
    content: &str,
    format: &str,
    meta: &DocMeta,
) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let now = chrono::Local::now();
    let ts = now.format("%Y%m%d-%H%M%S").to_string();
    let created_iso = now.format("%Y-%m-%dT%H:%M:%S").to_string();
    let body = build_document(content, format, &created_iso, meta);
    let ext = doc_extension(format);
    let stem = format!("{}-{ts}", filename_prefix(meta.kind));
    let name = next_unique_name(&stem, ext, |n| dir.join(n).exists());
    let path = dir.join(name);
    std::fs::write(&path, body).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// 保管庫エントリ(.txt/.md)を一覧する（S4.3 Phase1）。created 降順。
#[tauri::command]
fn list_entries(app: tauri::AppHandle) -> Result<Vec<vault::EntrySummary>, String> {
    let dir = resolve_save_dir(&current_settings(&app))?;
    vault::list_entries(&dir)
}

/// 保管庫フォルダを OS のファイルマネージャで開く（S4.1 R6）。無ければ作成してから開く。
#[tauri::command]
fn open_vault(app: tauri::AppHandle) -> Result<(), String> {
    let dir = resolve_save_dir(&current_settings(&app))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("保管庫の作成に失敗: {e}"))?;
    open_in_file_manager(&dir)
}

/// OS別にディレクトリをファイルマネージャで開く（待たずに起動）。
fn open_in_file_manager(dir: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    let mut cmd = {
        let mut c = std::process::Command::new("explorer");
        c.arg(dir);
        c
    };
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(dir);
        c
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(dir);
        c
    };
    // explorer は対象が開いても非0終了することがあるため spawn のみで成否判定しない。
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("ファイルマネージャの起動に失敗: {e}"))
}

/// フロントの保存設定を反映する。
#[tauri::command]
fn set_save_settings(
    state: tauri::State<'_, AppSettings>,
    save_dir: Option<String>,
    save_audio: bool,
    audio_format: String,
    keep_text: bool,
    output_format: Option<String>,
) -> Result<(), String> {
    let mut s = state
        .inner
        .lock()
        .map_err(|_| "設定のロックに失敗".to_string())?;
    s.save_dir = save_dir.filter(|d| !d.trim().is_empty());
    s.save_audio = save_audio;
    s.audio_format = audio_format;
    s.keep_text = keep_text;
    if let Some(f) = output_format {
        s.output_format = f;
    }
    Ok(())
}

/// 整形結果など任意テキストを保存先へ書き出す（整形は常に保存）。tags は内省タグ(S4.3)。
#[tauri::command]
fn save_note(
    app: tauri::AppHandle,
    content: String,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    let settings = current_settings(&app);
    let dir = resolve_save_dir(&settings)?;
    let tags = tags.unwrap_or_default();
    save_document(
        &dir,
        &content,
        &settings.output_format,
        &DocMeta {
            kind: "note",
            style: None,
            tags: &tags,
        },
    )
}

/// 16kHz mono 音声を文字起こしし、保存して返す共通処理（録音/ファイル入力で共用）。
/// 別スレッド(spawn_blocking 内)から呼ぶ前提。モデルが無ければ初回に自動DLする（S2.2）。
/// 進捗(0-100%)と確定セグメントを逐次通知してUIに進捗UXを提供する。
fn transcribe_blocking(
    app: &tauri::AppHandle,
    audio: &[f32],
    timestamps: bool,
) -> Result<String, String> {
    // STT設定を解決（S2.4）。既定はローカル whisper（プライバシー）。
    let stt = current_stt_settings(app);
    let provider = if stt.provider.trim().is_empty() {
        "local".to_string()
    } else {
        stt.provider.clone()
    };
    let is_cloud = stt::is_cloud_provider(&provider);

    // ローカルのみモデルを用意（クラウドは端末外処理＝モデルDL不要）。
    let model_path = if is_cloud {
        std::path::PathBuf::new()
    } else {
        let app_dl = app.clone();
        // 選択された whisper モデル（S2.2）。stt.model が空なら既定 base。
        model::ensure_model_id(&stt.model, move |done, total| {
            let msg = match total {
                Some(t) if t > 0 => format!("whisperモデルをダウンロード中… {}%", done * 100 / t),
                _ => format!("whisperモデルをダウンロード中… {} MB", done / 1_048_576),
            };
            let _ = app_dl.emit("status", msg);
        })?
    };

    let _ = app.emit(
        "status",
        if is_cloud {
            "クラウドで文字起こし中…"
        } else {
            "文字起こし中…"
        },
    );
    let app_p = app.clone();
    let app_s = app.clone();
    // STTエンジンを解決して文字起こし（S2.3抽象 / S2.4でクラウド）。
    let engine = stt::engine_for(stt::SttConfig {
        provider,
        model: stt.model,
        api_key: stt.api_key,
        azure_resource: stt.azure_resource,
        model_path,
    });
    let text = engine.transcribe(
        audio,
        Some("ja"),
        timestamps,
        Box::new(move |pct| {
            let _ = app_p.emit("progress", pct);
        }),
        Box::new(move |seg| {
            let _ = app_s.emit("segment", seg);
        }),
    )?;

    // 文字起こしテキストの保存は設定(keep_text)に従う。空(文字起こし対象なし)は保存しない。
    let settings = current_settings(app);
    if settings.keep_text && !text.trim().is_empty() {
        if let Ok(dir) = resolve_save_dir(&settings) {
            let _ = save_document(
                &dir,
                &text,
                &settings.output_format,
                &DocMeta {
                    kind: "transcript",
                    style: None,
                    tags: &[],
                },
            );
        }
    }
    let _ = app.emit("status", "");
    let _ = app.emit("progress", 100);
    Ok(text)
}

/// 入力ファイルのサイズ上限（メモリ膨張・長時間ブロックの防止 / #397）。
pub const MAX_INPUT_BYTES: u64 = 500 * 1024 * 1024; // 500 MB

/// 入力サイズが上限内かを検証する。超過時はユーザー向けの理由（次の一手つき）を返す。
pub fn check_input_size(len: u64) -> Result<(), String> {
    if len > MAX_INPUT_BYTES {
        let mb = |b: u64| (b as f64) / (1024.0 * 1024.0);
        return Err(format!(
            "ファイルが大きすぎます（約{:.0}MB）。上限は{:.0}MBです。録音を分割してお試しください。",
            mb(len),
            mb(MAX_INPUT_BYTES),
        ));
    }
    Ok(())
}

/// 音声ファイルから文字起こしし、結果を保存して返す（S1.6 ファイル入力）。
/// 非同期＋別スレッド実行でUIをブロックしない。
#[tauri::command]
async fn transcribe_file(
    app: tauri::AppHandle,
    path: String,
    timestamps: bool,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = std::path::Path::new(&path);
        // 巨大ファイルは復号前にサイズで弾く（無警告の長時間ブロック/メモリ膨張を防ぐ）。
        if let Ok(meta) = std::fs::metadata(p) {
            check_input_size(meta.len())?;
        }
        let _ = app.emit("status", "音声を読み込み中…");
        let audio = stt::decode_to_16k_mono(p)?;
        transcribe_blocking(&app, &audio, timestamps)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 利用可能な録音ソースを列挙する（S1.2/S1.3）。マイク入力＋出力デバイスのループバック。
#[tauri::command]
fn list_audio_sources() -> Result<Vec<record::AudioSource>, String> {
    record::list_audio_sources()
}

/// 選択可能な whisper モデルを列挙する（S2.2 ローカルSTTのモデル選択）。
#[tauri::command]
fn list_whisper_models() -> Vec<model::ModelInfo> {
    model::list_models()
}

/// マイク録音を開始する（S1.1/S1.2/S1.3）。
/// kind="loopback" なら出力デバイスのシステム音、それ以外はマイク入力。
/// device は入力=デバイス名 / ループバック=レンダーデバイスID（無ければ既定にフォールバック）。
/// E2E(QUICKSCRIBE_E2E=1)時は実マイク無しでもUIトグルを成立させるため何もしない。
#[tauri::command]
fn start_recording(
    state: tauri::State<'_, record::RecorderState>,
    device: Option<String>,
    kind: Option<String>,
) -> Result<(), String> {
    if std::env::var("QUICKSCRIBE_E2E").is_ok() {
        return Ok(());
    }
    let mut cur = state
        .current
        .lock()
        .map_err(|_| "録音状態のロックに失敗".to_string())?;
    if cur.is_some() {
        return Err("すでに録音中です".into());
    }
    *cur = Some(record::start(device, kind)?);
    Ok(())
}

/// マイク録音を停止し、録音音声を文字起こし・保存して返す（S1.1）。
/// 非同期＋別スレッドで文字起こしを実行しUIをブロックしない。
#[tauri::command]
async fn stop_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, record::RecorderState>,
    timestamps: bool,
) -> Result<(), String> {
    if std::env::var("QUICKSCRIBE_E2E").is_ok() {
        return Ok(());
    }
    // 録音ハンドルを取り出して停止し、音声(16k mono)を得る。ロックは await をまたがない。
    let recording = {
        let mut cur = state
            .current
            .lock()
            .map_err(|_| "録音状態のロックに失敗".to_string())?;
        cur.take().ok_or_else(|| "録音していません".to_string())?
    };
    let recorded = recording.finish()?;
    if recorded.mono16k.is_empty() {
        return Err(
            "録音データが空でした（録音が短すぎたか、選択した録音ソースに音声がありませんでした）".into(),
        );
    }
    let raw = recorded.raw;
    let sample_rate = recorded.sample_rate;
    let channels = recorded.channels;
    let audio = recorded.mono16k;

    // 文字起こしはバックグラウンドで実行（録音の非同期化＝stopは即返り録音状態を解放する）。
    // これにより文字起こし/整形中でも次の録音を開始できる。結果はイベントで通知する。
    let app_evt = app.clone();
    tauri::async_runtime::spawn(async move {
        let app_blk = app_evt.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            let text = transcribe_blocking(&app_blk, &audio, timestamps)?;
            // 文字起こし対象（発話）が無ければ、音声は保存せず空を返す。
            if text.trim().is_empty() {
                let _ = app_blk.emit("status", "");
                return Ok::<String, String>(String::new());
            }
            // 音声保存は「文字起こし対象があった場合かつ設定ON」のみ。原音を保存。
            let settings = current_settings(&app_blk);
            if settings.save_audio {
                if let Ok(dir) = resolve_save_dir(&settings) {
                    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
                    let stem = format!("rec-{ts}");
                    let r = if settings.audio_format == "opus" {
                        audio_save::save_opus(&raw, sample_rate, channels, &dir, &stem)
                    } else {
                        audio_save::save_wav(&raw, sample_rate, channels, &dir, &stem)
                    };
                    if let Err(e) = r {
                        let _ = app_blk.emit("status", format!("音声保存に失敗: {e}"));
                    }
                }
            }
            Ok(text)
        })
        .await;
        match result {
            Ok(Ok(text)) => {
                let _ = app_evt.emit("transcribe-done", text);
            }
            Ok(Err(e)) => {
                let _ = app_evt.emit("transcribe-error", e);
            }
            Err(e) => {
                let _ = app_evt.emit("transcribe-error", e.to_string());
            }
        }
    });

    Ok(())
}

/// 実行時のモデル解決に失敗したときのフォールバック既定（ミドルレンジ相当）。
/// 可能な範囲で「常に最新」を指すローリングエイリアスを採用する（deep research / ADR-0007）:
/// - gemini: `gemini-flash-latest`（公式のローリングlatestエイリアス）
/// - openai: `gpt-4o`（最新4oスナップショットを指すローリングエイリアス）
/// - anthropic: ローリングlatestが無いため取得時点の最新stable sonnetを既定にする。
fn default_model_for(provider: &str) -> &'static str {
    match provider.trim().to_ascii_lowercase().as_str() {
        "anthropic" | "claude" => "claude-sonnet-4-6",
        "openai" | "gpt" => "gpt-4o",
        "ollama" | "local" => "llama3.1",
        // AWS Bedrock のモデルIDは anthropic. プレフィックス(リージョン/アカウント依存。UIで上書き可)。
        "bedrock" | "aws-bedrock" => "anthropic.claude-sonnet-4-6",
        // Claude Platform on AWS は第一者と同じ bare ID。
        "claude-aws" | "claude-platform-aws" | "anthropic-aws" => "claude-sonnet-4-6",
        _ => "gemini-flash-latest",
    }
}

/// AWS系プロバイダ(Bedrock / Claude Platform on AWS)か。AwsConfig 組み立ての要否判定 / ADR-0011。
fn is_aws_provider(provider: &str) -> bool {
    matches!(
        provider.trim().to_ascii_lowercase().as_str(),
        "bedrock" | "aws-bedrock" | "claude-aws" | "claude-platform-aws" | "anthropic-aws"
    )
}

/// 実行時に各プロバイダのモデル一覧APIから「最新ミドルレンジ」モデルIDを解決する。
/// ビルド時固定でなく常に最新を選ぶため（ユーザ要望 / ADR-0007 deep research）。
/// 取得・解析に失敗したらフォールバック既定を返す（UIを止めない）。
#[tauri::command]
async fn resolve_model(provider: String, api_key: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let resolved = refine::resolve_latest_model(&provider, &api_key)
            .unwrap_or_else(|_| default_model_for(&provider).to_string());
        Ok::<String, String>(resolved)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 文字起こしテキストを整形(思考整理・要約)して保存し返す（E3 コアドメイン）。
/// 非同期＋別スレッドでUIをブロックしない。プロバイダ(Gemini/Anthropic/OpenAI)と
/// APIキー・モデルはフロントの設定から渡す（コードに鍵を埋め込まない / ADR-0005）。
#[tauri::command]
#[allow(clippy::too_many_arguments)]
async fn refine_text(
    app: tauri::AppHandle,
    text: String,
    provider: String,
    api_key: String,
    model: String,
    style: String,
    // ユーザー定義のカスタム整形指示(S3.3)。指定時は style の既定指示の代わりに使う。
    custom_instruction: Option<String>,
    // 内省タグ(S4.3)。保存時にメタデータとして付与する。
    tags: Option<Vec<String>>,
    // 保存するか(S4.3 Phase2 横断発見など一時的な結果は false で保管庫を汚さない)。既定 true。
    save: Option<bool>,
    // AWSプロバイダ(Bedrock / Claude Platform on AWS)用 / ADR-0011。非AWS時は未指定(None)。
    region: Option<String>,
    workspace_id: Option<String>,
    auth_mode: Option<String>, // "sigv4" | "apikey"(既定)
    aws_access_key: Option<String>,
    aws_secret_key: Option<String>,
    aws_session_token: Option<String>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let m = if model.trim().is_empty() {
            default_model_for(&provider).to_string()
        } else {
            model
        };
        // AWSプロバイダのときだけ AwsConfig を組み立てる。
        let aws_cfg = if is_aws_provider(&provider) {
            let auth = if auth_mode.as_deref() == Some("sigv4") {
                refine::AwsAuth::SigV4 {
                    access_key: aws_access_key.unwrap_or_default(),
                    secret_key: aws_secret_key.unwrap_or_default(),
                    session_token: aws_session_token,
                }
            } else {
                refine::AwsAuth::ApiKey
            };
            Some(refine::AwsConfig {
                region: region.unwrap_or_default(),
                workspace_id: workspace_id.unwrap_or_default(),
                auth,
            })
        } else {
            None
        };
        let refined = refine::refine(
            &provider,
            &api_key,
            &m,
            &style,
            &text,
            aws_cfg,
            custom_instruction,
        )?;
        // 整形結果（ジャーナルの成果物）は保存先へ書き出す（save=false の一時結果は保存しない）。
        let settings = current_settings(&app);
        if save.unwrap_or(true) {
            if let Ok(dir) = resolve_save_dir(&settings) {
                let tags = tags.unwrap_or_default();
                // 整形結果は構造化Markdownのため常に .md で保存（出力形式設定に依らない）。
                // 生の文字起こし(transcript)は出力形式設定(txt/md)に従う。
                let _ = save_document(
                    &dir,
                    &refined,
                    "md",
                    &DocMeta {
                        kind: "refined",
                        style: Some(&style),
                        tags: &tags,
                    },
                );
            }
        }
        Ok::<String, String>(refined)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// メモ/テキストファイル(.txt/.md等)を読み込んで内容を返す（整形のみ用途 / 文字起こし不要）。
#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("テキストの読み込みに失敗: {e}"))
}

/// タスクバーボタンに録音中バッジ(オーバーレイ)を表示/解除する（Windowsのみ。状態の可視化）。
#[tauri::command]
fn set_recording_overlay(app: tauri::AppHandle, recording: bool) {
    // トレイのツールチップ＋アイコンで録音状態を表示（全プラットフォーム）。
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(if recording {
            "QuickScribe — 録音中"
        } else {
            "QuickScribe — 待機中"
        }));
        if recording {
            let _ = tray.set_icon(Some(recording_tray_image()));
        } else if let Some(def) = app.default_window_icon().cloned() {
            let _ = tray.set_icon(Some(def));
        }
    }
    #[cfg(windows)]
    {
        if let Some(w) = app.get_webview_window("main") {
            if let Ok(h) = w.hwnd() {
                taskbar::set_overlay(h.0 as isize, recording);
            }
        }
        // タスクバー埋め込みウィジェットのボタン表示（録音⇄停止）も更新。
        taskbar_widget::set_recording(recording);
    }
}

/// トレイ用の「録音中」アイコン（赤い丸）を生成する。
fn recording_tray_image() -> tauri::image::Image<'static> {
    const N: u32 = 32;
    let mut rgba = vec![0u8; (N * N * 4) as usize];
    let center = (N as f32 - 1.0) / 2.0;
    let radius = center - 2.0;
    for y in 0..N {
        for x in 0..N {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= radius * radius {
                let i = ((y * N + x) * 4) as usize;
                rgba[i] = 0xE0; // R
                rgba[i + 1] = 0x20; // G
                rgba[i + 2] = 0x20; // B
                rgba[i + 3] = 0xFF; // A
            }
        }
    }
    tauri::image::Image::new_owned(rgba, N, N)
}

/// 録音トグルのグローバルホットキーを再設定する（設定でキー変更可能にする）。
/// 受理形式は Tauri アクセラレータ表記（例: "CommandOrControl+Shift+R"）。
#[tauri::command]
fn set_record_shortcut(app: tauri::AppHandle, accelerator: String) -> Result<(), String> {
    let shortcut: Shortcut = accelerator
        .parse()
        .map_err(|e| format!("ショートカット表記が不正です（{accelerator}）: {e}"))?;
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    gs.register(shortcut)
        .map_err(|e| format!("ショートカット登録に失敗: {e}"))?;
    Ok(())
}

/// タスクバーウィジェットのツールチップに表示する現在のショートカット表記を更新する（Windowsのみ）。
#[tauri::command]
fn set_taskbar_shortcut(display: String) {
    #[cfg(windows)]
    taskbar_widget::set_shortcut(display);
    #[cfg(not(windows))]
    let _ = display;
}

/// タスクバー上のウィジェット表示の有効/無効を切り替える（設定のトグル / Windowsのみ）。
#[tauri::command]
fn set_taskbar_widget(enabled: bool) {
    #[cfg(windows)]
    taskbar_widget::set_enabled(enabled);
    #[cfg(not(windows))]
    let _ = enabled;
}

/// 秘密情報(API鍵/AWSクレデンシャル)を OSセキュアストレージ(keyring)に保存する(S3.2)。
/// 空文字は「削除」扱い。サービス名 "QuickScribe"、user=key。
#[tauri::command]
fn set_secret(key: String, value: String) -> Result<(), String> {
    let entry =
        keyring::Entry::new("QuickScribe", &key).map_err(|e| format!("keyring初期化に失敗: {e}"))?;
    if value.is_empty() {
        let _ = entry.delete_credential();
        return Ok(());
    }
    entry
        .set_password(&value)
        .map_err(|e| format!("秘密情報の保存に失敗: {e}"))
}

/// OSセキュアストレージから秘密情報を取得する。未設定/取得不可(サービス無し等)は None。
#[tauri::command]
fn get_secret(key: String) -> Option<String> {
    keyring::Entry::new("QuickScribe", &key)
        .ok()
        .and_then(|e| e.get_password().ok())
}

/// OSセキュアストレージから秘密情報を削除する(未設定はOK扱い)。
#[tauri::command]
fn delete_secret(key: String) -> Result<(), String> {
    let entry =
        keyring::Entry::new("QuickScribe", &key).map_err(|e| format!("keyring初期化に失敗: {e}"))?;
    match entry.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("秘密情報の削除に失敗: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_input_size_accepts_within_limit_and_rejects_over() {
        assert!(check_input_size(0).is_ok());
        assert!(check_input_size(MAX_INPUT_BYTES).is_ok());
        let err = check_input_size(MAX_INPUT_BYTES + 1).unwrap_err();
        assert!(err.contains("大きすぎます"), "ユーザー向け理由を含む: {err}");
        assert!(err.contains("録音を分割"), "次の一手を含む: {err}");
    }

    #[test]
    fn filename_prefix_distinguishes_kinds() {
        // 生の文字起こしと整形済みをファイル名で見分けられる。
        assert_eq!(filename_prefix("transcript"), "transcript");
        assert_eq!(filename_prefix("refined"), "refined");
        assert_eq!(filename_prefix("note"), "note");
        assert_eq!(filename_prefix("unknown"), "note");
    }

    #[test]
    fn doc_extension_maps_md_else_txt() {
        assert_eq!(doc_extension("md"), "md");
        assert_eq!(doc_extension("MD"), "md");
        assert_eq!(doc_extension("txt"), "txt");
        assert_eq!(doc_extension(""), "txt");
        assert_eq!(doc_extension("zzz"), "txt");
    }

    #[test]
    fn build_document_txt_is_content_only() {
        let meta = DocMeta {
            kind: "refined",
            style: Some("構造化"),
            tags: &[],
        };
        // txt は本文そのまま(フロントマター無し・後方互換)。
        assert_eq!(build_document("本文", "txt", "2026-06-27T12:00:00", &meta), "本文");
    }

    #[test]
    fn build_document_md_refined_has_frontmatter_with_style() {
        let meta = DocMeta {
            kind: "refined",
            style: Some("構造化"),
            tags: &[],
        };
        let out = build_document("本文ABC", "md", "2026-06-27T12:00:00", &meta);
        assert!(out.starts_with("---\n"), "先頭はYAMLフロントマター");
        assert!(out.contains("schema: 1"), "スキーマ版マーカーを含む(ADR-0017)");
        assert!(out.contains("created: \"2026-06-27T12:00:00\""));
        assert!(out.contains("type: \"refined\""));
        assert!(out.contains("style: \"構造化\""));
        assert!(out.contains("\n---\n\n本文ABC"), "本文が後続する");
    }

    #[test]
    fn build_document_md_transcript_omits_style() {
        // style 無し(transcript)では style 行を出さない(三角測量)。
        let meta = DocMeta {
            kind: "transcript",
            style: None,
            tags: &[],
        };
        let out = build_document("x", "md", "2026-06-27T12:00:00", &meta);
        assert!(out.contains("type: \"transcript\""));
        assert!(!out.contains("style:"), "style 行は無い");
        assert!(!out.contains("tags:"), "tags 行は無い");
    }

    #[test]
    fn build_document_md_includes_tags_when_present() {
        let tags = vec!["仕事".to_string(), "不安".to_string()];
        let meta = DocMeta {
            kind: "refined",
            style: Some("構造化"),
            tags: &tags,
        };
        let out = build_document("本文", "md", "2026-06-27T12:00:00", &meta);
        assert!(out.contains("tags: [\"仕事\", \"不安\"]"), "frontmatterにtags配列: {out}");
    }

    #[test]
    fn build_document_txt_appends_tags_line() {
        // txt でもタグがあれば末尾に Tags: 行を付す(形式に依らずタグを残す)。
        let tags = vec!["アイデア".to_string()];
        let meta = DocMeta {
            kind: "note",
            style: None,
            tags: &tags,
        };
        let out = build_document("本文", "txt", "2026-06-27T12:00:00", &meta);
        assert_eq!(out, "本文\n\nTags: アイデア");
    }

    #[test]
    fn yaml_scalar_is_single_line_and_quoted() {
        // 改行はスペース化、二重引用符は単引用符化し、ダブルクォートで囲む(YAML安全)。
        let s = yaml_scalar("行1\n行2: \"値\"");
        assert!(!s.contains('\n'));
        assert!(s.starts_with('"') && s.ends_with('"'));
        assert!(s.contains("行1 行2"));
    }

    #[test]
    fn unique_name_without_conflict_is_plain() {
        assert_eq!(next_unique_name("note-x", "txt", |_| false), "note-x.txt");
    }

    #[test]
    fn unique_name_appends_suffix_on_conflict() {
        // note-x.txt と note-x-2.txt が埋まっていれば次は note-x-3.txt（三角測量）。
        let taken = ["note-x.txt", "note-x-2.txt"];
        let name = next_unique_name("note-x", "txt", |n| taken.contains(&n));
        assert_eq!(name, "note-x-3.txt");
    }

    #[test]
    fn resolve_save_dir_uses_override_when_set() {
        let s = SaveSettings {
            save_dir: Some("/tmp/myvault".into()),
            ..Default::default()
        };
        assert_eq!(
            resolve_save_dir(&s).unwrap(),
            std::path::PathBuf::from("/tmp/myvault")
        );
    }

    #[test]
    fn resolve_save_dir_blank_override_falls_back_to_default() {
        // 空白のみの上書きは未設定扱い（既定の保管庫へフォールバック）。
        let s = SaveSettings {
            save_dir: Some("   ".into()),
            ..Default::default()
        };
        if let Some(doc) = dirs::document_dir() {
            assert_eq!(resolve_save_dir(&s).unwrap(), doc.join("QuickScribe"));
        }
    }

    #[test]
    fn save_document_does_not_overwrite_existing() {
        // 一時ディレクトリで衝突時の非破壊保存(S4.1 R5)を結合検証。
        let meta = DocMeta {
            kind: "note",
            style: None,
            tags: &[],
        };
        let mut dir = std::env::temp_dir();
        dir.push(format!("qs-vault-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let p1 = save_document(&dir, "first", "txt", &meta).unwrap();
        let p2 = save_document(&dir, "second", "txt", &meta).unwrap();
        // 同一秒なら別名、別秒でも両方残ることを保証（どちらでもファイルは2つ）。
        assert_ne!(p1, p2);
        let count = std::fs::read_dir(&dir).unwrap().count();
        assert_eq!(count, 2, "既存エントリが上書きされず2件残る");
        assert_eq!(std::fs::read_to_string(&p1).unwrap(), "first");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_document_md_writes_md_extension_with_frontmatter() {
        let meta = DocMeta {
            kind: "refined",
            style: Some("要約"),
            tags: &[],
        };
        let mut dir = std::env::temp_dir();
        dir.push(format!("qs-vault-md-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let p = save_document(&dir, "本文", "md", &meta).unwrap();
        assert!(p.ends_with(".md"), "md 拡張子で保存される");
        let body = std::fs::read_to_string(&p).unwrap();
        assert!(body.starts_with("---\n") && body.contains("type: \"refined\""));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// メインウィンドウを表示して前面に出す（トレイ操作・常駐からの復帰で使う）。
fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 既定の開始/停止ホットキー: Ctrl/Cmd + Shift + R（設定で変更可能。set_record_shortcut）。
    let toggle_shortcut = Shortcut::new(Some(Modifiers::SHIFT | Modifiers::CONTROL), Code::KeyR);

    tauri::Builder::default()
        // 単一インスタンス: 2回目起動のargvを常駐インスタンスへ転送する（最初に登録する必要あり）。
        // 物理トリガー/自動化向けCLI（S1.5 / ADR-0014）:
        //   --toggle-record  録音トグル / --start-record 開始 / --stop-record 停止。引数無しはウィンドウ表示。
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|a| a == "--toggle-record") {
                let _ = app.emit("toggle-record", ());
            } else if argv.iter().any(|a| a == "--start-record") {
                let _ = app.emit("start-record", ());
            } else if argv.iter().any(|a| a == "--stop-record") {
                let _ = app.emit("stop-record", ());
            } else {
                show_main_window(app);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // OSログイン時の自動起動（S6.3）。--minimized で起動し常駐（ウィンドウは出さない）。
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--minimized"])
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                // 押下/解放を別イベントで通知し、フロントの録音モード（トグル/モーメンタリ）で振り分ける。
                // モーメンタリ（押している間だけ録音 / S1.5・ADR-0014）はキーの key-up が要るため
                // Pressed だけでなく Released も発火する。（set_record_shortcut 参照）
                .with_handler(move |app, _shortcut, event| match event.state() {
                    ShortcutState::Pressed => {
                        let _ = app.emit("record-press", ());
                    }
                    ShortcutState::Released => {
                        let _ = app.emit("record-release", ());
                    }
                })
                .build(),
        )
        .manage(record::RecorderState::default())
        .manage(AppSettings::default())
        .manage(SttState::default())
        .invoke_handler(tauri::generate_handler![
            save_note,
            open_vault,
            list_entries,
            transcribe_file,
            list_audio_sources,
            list_whisper_models,
            start_recording,
            stop_recording,
            resolve_model,
            refine_text,
            read_text_file,
            set_record_shortcut,
            set_save_settings,
            set_stt_settings,
            set_recording_overlay,
            set_taskbar_shortcut,
            set_taskbar_widget,
            set_secret,
            get_secret,
            delete_secret
        ])
        // ウィンドウを閉じてもアプリは終了せず、トレイに常駐する（タスクバー常駐の挙動）。
        // ただし E2E(QUICKSCRIBE_E2E=1)時はドライバが正常終了できるよう既定の閉じる挙動にする。
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if std::env::var("QUICKSCRIBE_E2E").is_err() {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(move |app| {
            // グローバルホットキー登録
            app.global_shortcut().register(toggle_shortcut.clone())?;

            // システムトレイ(右クリックメニュー)。タスクバー常駐の操作起点。
            let record_i =
                MenuItem::with_id(app, "record", "録音開始/停止", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "ウィンドウを表示", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&record_i, &show_i, &quit_i])?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("QuickScribe")
                .menu(&menu)
                // 右クリックメニューのみで開閉しないよう、左クリックの既定メニュー表示は無効化
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    // タスクバー(トレイ)から録音の開始/停止を操作する
                    "record" => {
                        let _ = app.emit("toggle-record", ());
                    }
                    "quit" => app.exit(0),
                    "show" => show_main_window(app),
                    _ => {}
                })
                // トレイアイコン左クリックでウィンドウを表示（録音操作はタスクバーから行う）。
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Windows: タスクバーのサムネイルツールバー（補助）に録音ボタンを取り付ける。
            #[cfg(windows)]
            {
                if let Some(w) = app.get_webview_window("main") {
                    taskbar::install(&w, app.handle().clone());
                }
                // タスクバーに録音/停止＋ウィンドウ表示ボタンを埋め込む（本命の操作導線）。
                taskbar_widget::install(app.handle().clone());
            }

            // 自動起動（--minimized）時はウィンドウを出さずトレイ常駐から始める（S6.3）。
            if std::env::args().any(|a| a == "--minimized") {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running QuickScribe");
}
