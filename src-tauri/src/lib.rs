// QuickScribe — Walking Skeleton (Phase 1)
//
// このフェーズの責務は「常駐(トレイ) + ウィンドウ + 録音トグル + グローバルホットキー +
// 指定フォルダへの保存導線」を貫通させること。文字起こし(whisper)・整形(LLM)・
// システム音声ループバック・デバイス切替・Stream Deck連携は後続の縦切りで追加する
// (ADR-0006 によりスコープからは外さない)。

pub mod audio_save;
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
fn save_text_in(dir: &std::path::Path, content: &str) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let path = dir.join(note_filename(&ts));
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
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

/// マイク録音を開始する（S1.1）。既定の入力デバイスから収集を始める。
/// E2E(QUICKSCRIBE_E2E=1)時は実マイク無しでもUIトグルを成立させるため何もしない。
#[tauri::command]
fn start_recording(state: tauri::State<'_, record::RecorderState>) -> Result<(), String> {
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
    *cur = Some(record::start()?);
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
        return Err("録音データが空でした（マイク入力を確認してください）".into());
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
        _ => "gemini-flash-latest",
    }
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
async fn refine_text(
    app: tauri::AppHandle,
    text: String,
    provider: String,
    api_key: String,
    model: String,
    style: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let m = if model.trim().is_empty() {
            default_model_for(&provider).to_string()
        } else {
            model
        };
        let refined = refine::refine(&provider, &api_key, &m, &style, &text)?;
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
            transcribe_file,
            start_recording,
            stop_recording,
            resolve_model,
            refine_text,
            read_text_file,
            set_record_shortcut,
            set_save_settings,
            set_recording_overlay,
            set_taskbar_shortcut
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
