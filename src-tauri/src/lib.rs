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
/// 保存先はのちに設定で上書き可能にする(現状は既定のみ)。
/// タイムスタンプ文字列からメモのファイル名を組み立てる（純粋関数・テスト対象 S7.1）。
fn note_filename(ts: &str) -> String {
    format!("note-{ts}.txt")
}

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

/// 保存に関する設定（保存先・音声保存可否/形式・文字起こしテキスト保持）。
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
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            save_dir: None,
            save_audio: false,
            audio_format: "wav".to_string(),
            keep_text: true,
        }
    }
}

#[derive(Default)]
struct AppSettings {
    inner: std::sync::Mutex<SaveSettings>,
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

/// タイムスタンプ付きファイル名で dir 配下にテキストを書き出し、パスを返す。
/// 同一秒の衝突では既存を上書きせず一意名にする（S4.1 R4/R5）。
fn save_text_in(dir: &std::path::Path, content: &str) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let stem = format!("note-{ts}");
    let name = next_unique_name(&stem, "txt", |n| dir.join(n).exists());
    let path = dir.join(name);
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
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
) -> Result<(), String> {
    let mut s = state
        .inner
        .lock()
        .map_err(|_| "設定のロックに失敗".to_string())?;
    s.save_dir = save_dir.filter(|d| !d.trim().is_empty());
    s.save_audio = save_audio;
    s.audio_format = audio_format;
    s.keep_text = keep_text;
    Ok(())
}

/// 整形結果など任意テキストを保存先へ書き出す（整形は常に保存）。
#[tauri::command]
fn save_note(app: tauri::AppHandle, content: String) -> Result<String, String> {
    let dir = resolve_save_dir(&current_settings(&app))?;
    save_text_in(&dir, &content)
}

/// 16kHz mono 音声を文字起こしし、保存して返す共通処理（録音/ファイル入力で共用）。
/// 別スレッド(spawn_blocking 内)から呼ぶ前提。モデルが無ければ初回に自動DLする（S2.2）。
/// 進捗(0-100%)と確定セグメントを逐次通知してUIに進捗UXを提供する。
fn transcribe_blocking(
    app: &tauri::AppHandle,
    audio: &[f32],
    timestamps: bool,
) -> Result<String, String> {
    // モデル（初回はダウンロードし進捗を通知）。
    let app_dl = app.clone();
    let model = model::ensure_model(move |done, total| {
        let msg = match total {
            Some(t) if t > 0 => format!("whisperモデルをダウンロード中… {}%", done * 100 / t),
            _ => format!("whisperモデルをダウンロード中… {} MB", done / 1_048_576),
        };
        let _ = app_dl.emit("status", msg);
    })?;

    let _ = app.emit("status", "文字起こし中…");
    let app_p = app.clone();
    let app_s = app.clone();
    let text = stt::transcribe_with(
        &model,
        audio,
        Some("ja"),
        timestamps,
        move |pct| {
            let _ = app_p.emit("progress", pct);
        },
        move |seg| {
            let _ = app_s.emit("segment", seg);
        },
    )?;

    // 文字起こしテキストの保存は設定(keep_text)に従う。空(文字起こし対象なし)は保存しない。
    let settings = current_settings(app);
    if settings.keep_text && !text.trim().is_empty() {
        if let Ok(dir) = resolve_save_dir(&settings) {
            let _ = save_text_in(&dir, &text);
        }
    }
    let _ = app.emit("status", "");
    let _ = app.emit("progress", 100);
    Ok(text)
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
        let _ = app.emit("status", "音声を読み込み中…");
        let audio = stt::decode_to_16k_mono(std::path::Path::new(&path))?;
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
        let refined = refine::refine(&provider, &api_key, &m, &style, &text, aws_cfg)?;
        // 整形結果（ジャーナルの成果物）は常に保存先へ書き出す。
        if let Ok(dir) = resolve_save_dir(&current_settings(&app)) {
            let _ = save_text_in(&dir, &refined);
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
    fn note_filename_has_prefix_and_extension() {
        assert_eq!(note_filename("20260616-120000"), "note-20260616-120000.txt");
    }

    #[test]
    fn note_filename_uses_given_timestamp() {
        assert!(note_filename("abc").starts_with("note-"));
        assert!(note_filename("abc").ends_with(".txt"));
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
    fn save_text_in_does_not_overwrite_existing() {
        // 一時ディレクトリで衝突時の非破壊保存(R5)を結合検証。
        let mut dir = std::env::temp_dir();
        dir.push(format!("qs-vault-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let p1 = save_text_in(&dir, "first").unwrap();
        let p2 = save_text_in(&dir, "second").unwrap();
        // 同一秒なら別名、別秒でも両方残ることを保証（どちらでもファイルは2つ）。
        assert_ne!(p1, p2);
        let count = std::fs::read_dir(&dir).unwrap().count();
        assert_eq!(count, 2, "既存エントリが上書きされず2件残る");
        assert_eq!(std::fs::read_to_string(&p1).unwrap(), "first");
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
        // `quickscribe --toggle-record` で録音トグル、引数無しはウィンドウ表示。
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|a| a == "--toggle-record") {
                let _ = app.emit("toggle-record", ());
            } else {
                show_main_window(app);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                // 録音トグルのショートカットは1つだけ登録するため、押下イベントは全て録音トグルに割当て。
                // （ユーザがキーを変更しても再登録だけで済む。set_record_shortcut 参照）
                .with_handler(move |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = app.emit("toggle-record", ());
                    }
                })
                .build(),
        )
        .manage(record::RecorderState::default())
        .manage(AppSettings::default())
        .invoke_handler(tauri::generate_handler![
            save_note,
            open_vault,
            transcribe_file,
            list_audio_sources,
            start_recording,
            stop_recording,
            resolve_model,
            refine_text,
            read_text_file,
            set_record_shortcut,
            set_save_settings,
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running QuickScribe");
}
