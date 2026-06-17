// QuickScribe — Walking Skeleton (Phase 1)
//
// このフェーズの責務は「常駐(トレイ) + ウィンドウ + 録音トグル + グローバルホットキー +
// 指定フォルダへの保存導線」を貫通させること。文字起こし(whisper)・整形(LLM)・
// システム音声ループバック・デバイス切替・Stream Deck連携は後続の縦切りで追加する
// (ADR-0006 によりスコープからは外さない)。

pub mod model;
pub mod record;
pub mod refine;
pub mod stt;

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

#[tauri::command]
fn save_note(content: String) -> Result<String, String> {
    let base = dirs::document_dir()
        .ok_or_else(|| "ドキュメントフォルダが見つかりません".to_string())?
        .join("QuickScribe");
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let path = base.join(note_filename(&ts));
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// 16kHz mono 音声を文字起こしし、保存して返す共通処理（録音/ファイル入力で共用）。
/// 別スレッド(spawn_blocking 内)から呼ぶ前提。モデルが無ければ初回に自動DLする（S2.2）。
/// 進捗(0-100%)と確定セグメントを逐次通知してUIに進捗UXを提供する。
fn transcribe_blocking(app: &tauri::AppHandle, audio: &[f32]) -> Result<String, String> {
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
        move |pct| {
            let _ = app_p.emit("progress", pct);
        },
        move |seg| {
            let _ = app_s.emit("segment", seg);
        },
    )?;

    let _ = save_note(text.clone())?;
    let _ = app.emit("status", "");
    let _ = app.emit("progress", 100);
    Ok(text)
}

/// 音声ファイルから文字起こしし、結果を保存して返す（S1.6 ファイル入力）。
/// 非同期＋別スレッド実行でUIをブロックしない。
#[tauri::command]
async fn transcribe_file(app: tauri::AppHandle, path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _ = app.emit("status", "音声を読み込み中…");
        let audio = stt::decode_to_16k_mono(std::path::Path::new(&path))?;
        transcribe_blocking(&app, &audio)
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
) -> Result<String, String> {
    if std::env::var("QUICKSCRIBE_E2E").is_ok() {
        return Ok(String::new());
    }
    // 録音ハンドルを取り出して停止し、音声(16k mono)を得る。ロックは await をまたがない。
    let recording = {
        let mut cur = state
            .current
            .lock()
            .map_err(|_| "録音状態のロックに失敗".to_string())?;
        cur.take().ok_or_else(|| "録音していません".to_string())?
    };
    let audio = recording.finish()?;
    if audio.is_empty() {
        return Err("録音データが空でした（マイク入力を確認してください）".into());
    }

    tauri::async_runtime::spawn_blocking(move || transcribe_blocking(&app, &audio))
        .await
        .map_err(|e| e.to_string())?
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
    text: String,
    provider: String,
    api_key: String,
    model: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let m = if model.trim().is_empty() {
            default_model_for(&provider).to_string()
        } else {
            model
        };
        let refined = refine::refine(&provider, &api_key, &m, &text)?;
        let _ = save_note(refined.clone())?;
        Ok::<String, String>(refined)
    })
    .await
    .map_err(|e| e.to_string())?
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
    // 既定の開始/停止ホットキー: Ctrl/Cmd + Shift + R
    let toggle_shortcut = Shortcut::new(Some(Modifiers::SHIFT | Modifiers::CONTROL), Code::KeyR);
    let handler_shortcut = toggle_shortcut.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed && shortcut == &handler_shortcut {
                        let _ = app.emit("toggle-record", ());
                    }
                })
                .build(),
        )
        .manage(record::RecorderState::default())
        .invoke_handler(tauri::generate_handler![
            save_note,
            transcribe_file,
            start_recording,
            stop_recording,
            resolve_model,
            refine_text
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
                // トレイアイコン左クリックでウィンドウを表示
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running QuickScribe");
}
