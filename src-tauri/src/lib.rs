// QuickScribe — Walking Skeleton (Phase 1)
//
// このフェーズの責務は「常駐(トレイ) + ウィンドウ + 録音トグル + グローバルホットキー +
// 指定フォルダへの保存導線」を貫通させること。文字起こし(whisper)・整形(LLM)・
// システム音声ループバック・デバイス切替・Stream Deck連携は後続の縦切りで追加する
// (ADR-0006 によりスコープからは外さない)。

pub mod audio_save;
// AWS SigV4署名(Bedrock / Claude Platform on AWS の整形プロバイダ用 / ADR-0011)。
pub mod aws_sign;
// 安定エラーコード（#462 i18n Phase2）。ユーザー向けエラーの SSOT。
pub mod errcode;
pub mod model;
pub mod record;
pub mod refine;
pub mod stt;
// 保管庫エントリの一覧・解析（S4.3 Phase1: アプリ内の横断導線）。
pub mod vault;
// 保管庫ドキュメントの本文組み立て・命名(#392 / DDD: lib.rs から抽出)。
pub mod entry;
pub mod job;
// Windows タスクバーのサムネイルツールバー/オーバーレイ。Windowsのみ。
#[cfg(windows)]
mod taskbar;
// Windows タスクバーに埋め込む録音ウィジェット（常時表示の操作ボタン）。Windowsのみ。
#[cfg(windows)]
mod taskbar_widget;
// テスト専用の極小HTTPサーバ＋環境変数ガード（監査項目12: HTTP経路のユニットテスト）。
#[cfg(test)]
pub(crate) mod testhttp;

/// APIベースURLを返す。テストビルドのみ環境変数でローカルテストサーバへ差し替え可能
/// （cfg(test) 限定＝リリースの公開挙動は不変 / 監査項目12: ureq経路のユニットテスト用）。
pub(crate) fn api_base(prod: &str, env_key: &str) -> String {
    #[cfg(test)]
    {
        if let Ok(v) = std::env::var(env_key) {
            let v = v.trim().trim_end_matches('/').to_string();
            if !v.is_empty() {
                return v;
            }
        }
    }
    #[cfg(not(test))]
    let _ = env_key;
    prod.to_string()
}

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use entry::{build_document, doc_extension, filename_prefix, DocMeta};

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
    /// GPU実行を無効化する(ユーザー設定 / ADR-0027)。既定 false=環境が許せばGPUを使う(速度最適)。
    /// 反転フラグなのは Default(false) を「GPU有効」に一致させるため。
    disable_gpu: bool,
}

#[derive(Default)]
struct SttState {
    inner: std::sync::Mutex<SttSettings>,
}

/// 現在のSTT設定のスナップショットを返す（未設定なら provider="local" 相当）。
fn current_stt_settings<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> SttSettings {
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
    use_gpu: Option<bool>,
) -> Result<(), String> {
    let mut s = state
        .inner
        .lock()
        .map_err(|_| errcode::E_LOCK_STT.to_string())?;
    s.provider = provider;
    s.model = model;
    s.api_key = api_key;
    s.azure_resource = azure_resource.unwrap_or_default();
    // 未指定(旧フロント)は既定=GPU有効のまま。明示 false のときだけ無効化(ADR-0027)。
    s.disable_gpu = !use_gpu.unwrap_or(true);
    Ok(())
}

/// 現在の保存設定のスナップショットを返す。
fn current_settings<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> SaveSettings {
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
        .ok_or_else(|| errcode::E_NO_DOCUMENT_DIR.to_string())?
        .join("QuickScribe"))
}

/// タイムスタンプ付きファイル名で dir 配下にエントリを書き出し、パスを返す（S4.1/S4.2）。
/// 出力形式(txt/md)とメタデータに従って本文を組み立て、同一秒の衝突は一意名にする（非破壊）。
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
fn list_entries<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<Vec<vault::EntrySummary>, String> {
    let dir = resolve_save_dir(&current_settings(&app))?;
    vault::list_entries(&dir)
}

/// 保管庫フォルダを OS のファイルマネージャで開く（S4.1 R6）。無ければ作成してから開く。
#[tauri::command]
fn open_vault<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    let dir = resolve_save_dir(&current_settings(&app))?;
    std::fs::create_dir_all(&dir).map_err(|e| errcode::ec(errcode::E_JOURNAL_DIR, e))?;
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
        .map_err(|e| errcode::ec(errcode::E_FILE_MANAGER, e))
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
        .map_err(|_| errcode::E_LOCK_SETTINGS.to_string())?;
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
/// kind は保存物の種別（既定 "note"）。用語補正済みの文字起こしを別ファイルで残す場合は
/// "transcript" を渡す(#599)。未知の kind は "note" 相当にフォールバックする(filename_prefix)。
/// いずれも非破壊(一意名)＝原本は書き換えない(ADR-0017)。
#[tauri::command]
fn save_note<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    content: String,
    tags: Option<Vec<String>>,
    kind: Option<String>,
) -> Result<String, String> {
    let settings = current_settings(&app);
    let dir = resolve_save_dir(&settings)?;
    let tags = tags.unwrap_or_default();
    let kind = kind.unwrap_or_else(|| "note".to_string());
    save_document(
        &dir,
        &content,
        &settings.output_format,
        &DocMeta {
            kind: &kind,
            style: None,
            tags: &tags,
        },
    )
}

/// 16kHz mono 音声を文字起こしし、保存して返す共通処理（録音/ファイル入力で共用）。
/// 別スレッド(spawn_blocking 内)から呼ぶ前提。モデルが無ければ初回に自動DLする（S2.2）。
/// 進捗(0-100%)と確定セグメントを逐次通知してUIに進捗UXを提供する。
fn transcribe_blocking<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    job_id: Option<job::JobId>,
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
            // 進行状況は安定コード＋数値詳細で通知し、フロントがローカライズする（#462）。
            let msg = match total {
                Some(t) if t > 0 => {
                    errcode::ec(errcode::S_MODEL_DOWNLOAD_PCT, done * 100 / t)
                }
                _ => errcode::ec(errcode::S_MODEL_DOWNLOAD_MB, done / 1_048_576),
            };
            let _ = app_dl.emit("status", msg);
        })?
    };

    let _ = app.emit(
        "status",
        if is_cloud {
            errcode::S_TRANSCRIBING_CLOUD
        } else {
            errcode::S_TRANSCRIBING
        },
    );
    let app_p = app.clone();
    let app_s = app.clone();
    // STTエンジンを解決して文字起こし（S2.3抽象 / S2.4でクラウド）。
    // GPU実行(ADR-0027): CUDA変種かつ実行環境にNVIDIAドライバがあり、ユーザーが無効化していないとき。
    let use_gpu = !stt.disable_gpu && nvidia_driver_present();
    let engine = stt::engine_for(stt::SttConfig {
        provider,
        model: stt.model,
        api_key: stt.api_key,
        azure_resource: stt.azure_resource,
        model_path,
        use_gpu,
    });
    let text = engine.transcribe(
        audio,
        Some("ja"),
        timestamps,
        Box::new(move |pct| {
            let _ = app_p.emit("progress", pct);
            // job_id 付き進捗（マルチジョブUI / ADR-0026）。0-100 にクランプ。
            if let Some(jid) = job_id {
                let _ = app_p.emit(
                    "job-progress",
                    JobProgress {
                        job_id: jid,
                        progress: pct.clamp(0, 100) as u8,
                    },
                );
            }
        }),
        Box::new(move |seg| {
            let _ = app_s.emit("segment", seg.clone());
            if let Some(jid) = job_id {
                let _ = app_s.emit("job-segment", JobSegment { job_id: jid, text: seg });
            }
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

// ─── マルチジョブ・キュー（ADR-0026 / #621 Phase1）──────────────────────────
// 録音停止ごとに 1 ジョブを発番し、並列度1（FIFO）の逐次キューで処理する。
// これにより「文字起こし中に次の録音を停止しても前ジョブが消えない」（取りこぼさない）。

/// 終了済みジョブの履歴保持上限（キュー無限成長の防止 / UI は「最近N件＋履歴」）。
const JOB_HISTORY_KEEP: usize = 50;

/// キュー投入待ちの音声ペイロード（UI に見せる Job メタデータとは分離）。
struct PendingWork {
    audio: Vec<f32>,
    raw: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    timestamps: bool,
}

/// ジョブ実行状態（Tauri 管理状態）。純粋な状態機械 job::JobQueue に加え、
/// 実行待ち音声と「ワーカー稼働中フラグ」を持つ。ロックは await をまたがない。
#[derive(Default)]
struct JobState {
    inner: std::sync::Mutex<JobExec>,
}

#[derive(Default)]
struct JobExec {
    queue: job::JobQueue,
    pending: std::collections::HashMap<job::JobId, PendingWork>,
    worker_active: bool,
}

impl JobExec {
    /// 不変条件を保つ: `pending`（録音全体の Vec<f32> を保持）は **Queued のジョブ分のみ**残す。
    /// Running は pick 時点で除去済み、終了(Done/Error/Canceled)・キュー削除済みの分はここで解放する。
    /// これにより「Queued のままキャンセル/消滅したジョブの音声バッファが RAM に残り続ける」リークを防ぐ
    /// （Phase3 の cancel コマンド配線後も安全 / レビュー指摘）。
    fn drop_orphan_pending(&mut self) {
        let queue = &self.queue;
        self.pending.retain(|id, _| {
            matches!(queue.get(*id).map(|j| j.status), Some(job::JobStatus::Queued))
        });
    }
}

// job_id 付きイベントのペイロード（フロントは camelCase で受ける / マルチジョブUI）。
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct JobProgress {
    job_id: job::JobId,
    progress: u8,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct JobSegment {
    job_id: job::JobId,
    text: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct JobStatusEvent {
    job_id: job::JobId,
    status: &'static str,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct JobDone {
    job_id: job::JobId,
    text: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct JobError {
    job_id: job::JobId,
    code: String,
}

/// 1 ジョブ分の文字起こし＋（設定に従い）録音音声保存。旧 stop_recording の per-job 処理を抽出。
/// 発話が無ければ空文字を返す（保存しない）。別スレッド(spawn_blocking)から呼ぶ前提。
fn run_job<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    job_id: job::JobId,
    work: &PendingWork,
) -> Result<String, String> {
    let text = transcribe_blocking(app, Some(job_id), &work.audio, work.timestamps)?;
    if text.trim().is_empty() {
        let _ = app.emit("status", "");
        return Ok(String::new());
    }
    // 音声保存は「文字起こし対象があった場合かつ設定ON」のみ。原音を保存。
    let settings = current_settings(app);
    if settings.save_audio {
        if let Ok(dir) = resolve_save_dir(&settings) {
            let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
            let stem = format!("rec-{ts}");
            let r = if settings.audio_format == "opus" {
                audio_save::save_opus(&work.raw, work.sample_rate, work.channels, &dir, &stem)
            } else {
                audio_save::save_wav(&work.raw, work.sample_rate, work.channels, &dir, &stem)
            };
            if let Err(e) = r {
                let _ = app.emit("status", errcode::ec(errcode::S_AUDIO_SAVE_FAILED, e));
            }
        }
    }
    Ok(text)
}

/// 逐次キューのワーカー。稼働中ジョブが 0 になるまで FIFO で1件ずつ処理する（並列度1）。
/// 単一ワーカーであることが逐次性（同時 Running=高々1件）を保証する。
/// 結果は job_id 付きイベント（マルチジョブUI）と旧イベント（現行UI互換）の双方で通知する。
/// JobState をロックして JobExec を操作する共通ヘルパ。ロック（借用）は本関数内で完結し
/// await をまたがない。
/// poison（ロック保持中のパニック）は **回復する**（`into_inner`）。JobState は長命な常駐
/// サブシステムであり、1度のパニックでキュー全体を恒久ブリックさせる方が害が大きいため。
/// 保持する状態（JobQueue/pending）は単純な操作のみで、torn 状態のリスクは実質無い。
fn with_job_state<R, F, T>(app: &tauri::AppHandle<R>, f: F) -> T
where
    R: tauri::Runtime,
    F: FnOnce(&mut JobExec) -> T,
{
    let st = app.state::<JobState>();
    let mut ex = st.inner.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut ex)
}

fn spawn_job_worker<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        loop {
            // 次ジョブを選ぶ（ロックはヘルパ内で完結し await をまたがない）。
            // キュー枯渇なら worker_active=false にしてワーカー終了。
            let picked = with_job_state(&app, |ex| match ex.queue.next_queued() {
                Some(id) => {
                    ex.queue.mark_running(id);
                    (Some(id), ex.pending.remove(&id))
                }
                None => {
                    ex.worker_active = false;
                    (None, None)
                }
            });
            let (id, work) = match picked {
                (Some(id), work) => (id, work),
                (None, _) => break,
            };
            let _ = app.emit(
                "job-status",
                JobStatusEvent {
                    job_id: id,
                    status: "running",
                },
            );
            // 音声ペイロードが無い（想定外）→ error にして次へ。
            let Some(work) = work else {
                finalize_error(&app, id, errcode::E_EMPTY_RECORDING.to_string());
                continue;
            };
            let app_run = app.clone();
            let result =
                tauri::async_runtime::spawn_blocking(move || run_job(&app_run, id, &work)).await;
            match result {
                Ok(Ok(text)) => {
                    with_job_state(&app, |ex| {
                        ex.queue.mark_done(id);
                        ex.queue.prune_finished(JOB_HISTORY_KEEP);
                        ex.drop_orphan_pending();
                    });
                    let _ = app.emit(
                        "job-done",
                        JobDone {
                            job_id: id,
                            text: text.clone(),
                        },
                    );
                    // 現行UI互換（Phase2 で job-* へ移行し撤去予定）。
                    let _ = app.emit("transcribe-done", text);
                }
                Ok(Err(e)) => finalize_error(&app, id, e),
                Err(e) => finalize_error(&app, id, e.to_string()),
            }
        }
    });
}

/// ジョブを Error 状態にし、job_id 付き＋旧イベントの双方で失敗を通知する。
fn finalize_error<R: tauri::Runtime>(app: &tauri::AppHandle<R>, id: job::JobId, code: String) {
    with_job_state(app, |ex| {
        ex.queue.mark_error(id, code.clone());
        ex.queue.prune_finished(JOB_HISTORY_KEEP);
        ex.drop_orphan_pending();
    });
    let _ = app.emit(
        "job-error",
        JobError {
            job_id: id,
            code: code.clone(),
        },
    );
    let _ = app.emit("transcribe-error", code);
}

/// 現在のジョブ一覧（登録順＝FIFO）を返す。マルチジョブUI（Phase2）が購読・描画する。
/// poison は回復する（一覧取得は読み取りのみで、恒久失敗させる必要がない）。
#[tauri::command]
fn list_jobs<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Vec<job::Job> {
    with_job_state(&app, |ex| ex.queue.jobs().to_vec())
}

/// 入力ファイルのサイズ上限（メモリ膨張・長時間ブロックの防止 / #397）。
pub const MAX_INPUT_BYTES: u64 = 500 * 1024 * 1024; // 500 MB

/// 入力サイズが上限内かを検証する。超過時は安定コード＋詳細（実サイズ/上限）を返す（#462）。
pub fn check_input_size(len: u64) -> Result<(), String> {
    if len > MAX_INPUT_BYTES {
        let mb = |b: u64| (b as f64) / (1024.0 * 1024.0);
        return Err(errcode::ec(
            errcode::E_FILE_TOO_LARGE,
            format!("{:.0}MB > {:.0}MB", mb(len), mb(MAX_INPUT_BYTES)),
        ));
    }
    Ok(())
}

/// 対応する音声入力形式（フロント src/lib/constants.ts の SUPPORTED_AUDIO_EXTS と一致させる
/// こと / #18。契約テスト supported_audio_exts_match_frontend が両側の一致を機械検証する）。
pub const SUPPORTED_AUDIO_EXTS: &[&str] = &["mp3", "wav", "m4a", "flac", "ogg", "opus", "aac"];

/// 拡張子が対応形式かを検証する（純関数・テスト対象）。非対応は対応形式を添えて弾く。
pub fn check_audio_extension(path: &std::path::Path) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    match ext {
        Some(e) if SUPPORTED_AUDIO_EXTS.contains(&e.as_str()) => Ok(()),
        _ => Err(errcode::ec(
            errcode::E_UNSUPPORTED_AUDIO_EXT,
            SUPPORTED_AUDIO_EXTS.join(" / "),
        )),
    }
}

/// 音声ファイルから文字起こしし、結果を保存して返す（S1.6 ファイル入力）。
/// 非同期＋別スレッド実行でUIをブロックしない。
#[tauri::command]
async fn transcribe_file<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    path: String,
    timestamps: bool,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = std::path::Path::new(&path);
        // 対応形式かを先に検証（非対応は分かりやすく弾く / #18）。
        check_audio_extension(p)?;
        // 巨大ファイルは復号前にサイズで弾く。metadata失敗は無警告スキップせず明示エラーにする。
        let meta = std::fs::metadata(p).map_err(|e| errcode::ec(errcode::E_FILE_OPEN, e))?;
        check_input_size(meta.len())?;
        let _ = app.emit("status", errcode::S_LOADING_AUDIO);
        let audio = stt::decode_to_16k_mono(p)?;
        // ファイル入力(S1.6)は録音ジョブキューには載せない（job_id なし＝従来どおり）。
        transcribe_blocking(&app, None, &audio, timestamps)
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

/// NVIDIAドライバ(nvcuda.dll)が実行環境に存在するか（ADR-0027 の実行時GPU判定）。
/// CUDA変種でも非搭載機では GPU を使わない（use_gpu=false ならCUDA APIは一切呼ばれない＝安全）。
/// 64bitプロセスのため System32 は実体を指す。CPUビルド/非Windowsでは常に false。
fn nvidia_driver_present() -> bool {
    if !cfg!(feature = "cuda") {
        return false;
    }
    #[cfg(windows)]
    {
        let sysroot = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
        return std::path::Path::new(&sysroot).join("System32").join("nvcuda.dll").exists();
    }
    #[cfg(not(windows))]
    false
}

/// このビルドの文字起こし実行バックエンド（配布変種と実行時GPU判定の可視化 / ADR-0027）。
/// variant: "cuda"=GPU変種ビルド / "cpu"=既定のCPU版。gpu_available: 実行環境でGPUが使えるか。
/// 起動時にフロントがこれを読み、既定で速度最適な実行モード(GPU可ならGPU)を選ぶ。
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SttBackendInfo {
    variant: &'static str,
    gpu_available: bool,
}

#[tauri::command]
fn stt_backend() -> SttBackendInfo {
    SttBackendInfo {
        variant: if cfg!(feature = "cuda") { "cuda" } else { "cpu" },
        gpu_available: nvidia_driver_present(),
    }
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
        .map_err(|_| errcode::E_LOCK_RECORD_STATE.to_string())?;
    if cur.is_some() {
        return Err(errcode::E_ALREADY_RECORDING.into());
    }
    *cur = Some(record::start(device, kind)?);
    Ok(())
}

/// マイク録音を停止し、録音音声を文字起こし・保存して返す（S1.1）。
/// 非同期＋別スレッドで文字起こしを実行しUIをブロックしない。
#[tauri::command]
async fn stop_recording<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
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
            .map_err(|_| errcode::E_LOCK_RECORD_STATE.to_string())?;
        cur.take().ok_or_else(|| errcode::E_NOT_RECORDING.to_string())?
    };
    let recorded = recording.finish()?;
    if recorded.mono16k.is_empty() {
        return Err(errcode::E_EMPTY_RECORDING.into());
    }
    // 音声長（秒）＝行ラベル/ETA の目安。16kHz mono 前提。
    let duration_secs = recorded.mono16k.len() as f64 / stt::WHISPER_SR as f64;
    let work = PendingWork {
        audio: recorded.mono16k,
        raw: recorded.raw,
        sample_rate: recorded.sample_rate,
        channels: recorded.channels,
        timestamps,
    };

    // ジョブを発番して逐次キューへ即投入し、stop は即返る（取りこぼさない / ADR-0026）。
    // ワーカー未稼働なら起動する。稼働中なら既存ワーカーが FIFO で拾う（並列度1）。
    let created_at_ms = chrono::Local::now().timestamp_millis();
    let (start_worker, snapshot) = with_job_state(&app, |ex| {
        let id = ex.queue.enqueue(created_at_ms, duration_secs);
        ex.pending.insert(id, work);
        let start = !ex.worker_active;
        if start {
            ex.worker_active = true;
        }
        (start, ex.queue.get(id).cloned())
    });
    // 一覧に行を追加できるよう、発番したジョブのメタデータを通知する。
    if let Some(job) = snapshot {
        let _ = app.emit("job-created", job);
    }
    if start_worker {
        spawn_job_worker(app.clone());
    }

    Ok(())
}

/// 実行時のモデル解決に失敗したときのフォールバック既定（ミドルレンジ相当）。
/// 可能な範囲で「常に最新」を指すローリングエイリアスを採用する（deep research / ADR-0007）:
/// - gemini: `gemini-flash-latest`（公式のローリングlatestエイリアス）
/// - openai: `gpt-4o`（最新4oスナップショットを指すローリングエイリアス）
/// - anthropic: ローリングlatestが無いため取得時点の最新stable sonnetを既定にする。
fn default_model_for(provider: &str) -> &'static str {
    refine::RefineProvider::parse(provider).default_model()
}

/// AWS系プロバイダ(Bedrock / Claude Platform on AWS)か。AwsConfig 組み立ての要否判定 / ADR-0011。
fn is_aws_provider(provider: &str) -> bool {
    refine::RefineProvider::parse(provider).is_aws()
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

/// `refine_text` コマンドの引数（#392: 16引数を params 構造体へ集約 / OCP）。
/// フロントは camelCase で送る（`buildRefineArgs` と対応）。鍵はコードに埋め込まない(ADR-0005)。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefineTextParams {
    pub text: String,
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub style: String,
    /// ユーザー定義のカスタム整形指示(S3.3)。指定時は style の既定指示の代わりに使う。
    pub custom_instruction: Option<String>,
    /// 内省タグ(S4.3)。保存時にメタデータとして付与する。
    pub tags: Option<Vec<String>>,
    /// 保存するか(S4.3 Phase2 横断発見など一時的な結果は false で保管庫を汚さない)。既定 true。
    pub save: Option<bool>,
    /// AWSプロバイダ(Bedrock / Claude Platform on AWS)用 / ADR-0011。非AWS時は None。
    pub region: Option<String>,
    pub workspace_id: Option<String>,
    /// "sigv4" | "apikey"(既定)。
    pub auth_mode: Option<String>,
    pub aws_access_key: Option<String>,
    pub aws_secret_key: Option<String>,
    pub aws_session_token: Option<String>,
    /// 整形出力言語(翻訳 / #453)。Some(英語名)時、指定言語で整形出力する。非指定は原語のまま。
    pub output_lang: Option<String>,
    /// OpenAI互換エンドポイントの接続先(base_url / #593)。OpenAIプロバイダで Some かつ非空なら
    /// 公式の代わりにこの URL へ送る(LiteLLM 等ゲートウェイ・self-host ローカルLLM対応)。上級者向け。
    pub base_url: Option<String>,
}

/// 文字起こしテキストを整形(思考整理・要約)して保存し返す（E3 コアドメイン）。
/// 非同期＋別スレッドでUIをブロックしない。プロバイダ(Gemini/Anthropic/OpenAI)と
/// APIキー・モデルはフロントの設定から渡す（コードに鍵を埋め込まない / ADR-0005）。
#[tauri::command]
async fn refine_text<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    params: RefineTextParams,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let RefineTextParams {
            text,
            provider,
            api_key,
            model,
            style,
            custom_instruction,
            tags,
            save,
            region,
            workspace_id,
            auth_mode,
            aws_access_key,
            aws_secret_key,
            aws_session_token,
            output_lang,
            base_url,
        } = params;
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
            output_lang,
            base_url,
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
    std::fs::read_to_string(&path).map_err(|e| errcode::ec(errcode::E_TEXT_READ, e))
}

/// トレイのメニュー/ツールチップ文言（UI言語で解決した文字列をフロントが送る / #462）。
/// トレイメニューはRust側で文字列が必要なため、フロントが起動時とUI言語切替時に
/// `set_tray_texts` で現在言語の文言を注入する。未注入時は日本語既定で表示する。
struct TrayTexts {
    inner: std::sync::Mutex<TrayTextValues>,
}

#[derive(Clone)]
struct TrayTextValues {
    tooltip_recording: String,
    tooltip_idle: String,
}

impl Default for TrayTexts {
    fn default() -> Self {
        Self {
            inner: std::sync::Mutex::new(TrayTextValues {
                tooltip_recording: "QuickScribe — 録音中".into(),
                tooltip_idle: "QuickScribe — 待機中".into(),
            }),
        }
    }
}

/// トレイのメニュー項目・ツールチップ文言を現在のUI言語へ更新する（フロントから呼ぶ）。
/// メニューは項目テキストを変えるため作り直して差し替える（メニュー操作はメインスレッドで行う）。
#[tauri::command]
fn set_tray_texts<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    record: String,
    show: String,
    quit: String,
    tooltip_recording: String,
    tooltip_idle: String,
) {
    if let Ok(mut g) = app.state::<TrayTexts>().inner.lock() {
        g.tooltip_recording = tooltip_recording;
        g.tooltip_idle = tooltip_idle.clone();
    }
    let app2 = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(tray) = app2.tray_by_id("main-tray") {
            let items = (
                MenuItem::with_id(&app2, "record", &record, true, None::<&str>),
                MenuItem::with_id(&app2, "show", &show, true, None::<&str>),
                MenuItem::with_id(&app2, "quit", &quit, true, None::<&str>),
            );
            if let (Ok(r), Ok(s), Ok(q)) = items {
                if let Ok(menu) = Menu::with_items(&app2, &[&r, &s, &q]) {
                    let _ = tray.set_menu(Some(menu));
                }
            }
            // 録音状態はフロントが set_recording_overlay で管理するため、ここでは待機中表示へ更新。
            let _ = tray.set_tooltip(Some(tooltip_idle.as_str()));
        }
    });
}

/// タスクバーボタンに録音中バッジ(オーバーレイ)を表示/解除する（Windowsのみ。状態の可視化）。
#[tauri::command]
fn set_recording_overlay(app: tauri::AppHandle, recording: bool) {
    // トレイのツールチップ＋アイコンで録音状態を表示（全プラットフォーム）。
    if let Some(tray) = app.tray_by_id("main-tray") {
        let texts = app
            .state::<TrayTexts>()
            .inner
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| TrayTextValues {
                tooltip_recording: "QuickScribe — 録音中".into(),
                tooltip_idle: "QuickScribe — 待機中".into(),
            });
        let _ = tray.set_tooltip(Some(if recording {
            texts.tooltip_recording.as_str()
        } else {
            texts.tooltip_idle.as_str()
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
        .map_err(|e| errcode::ec(errcode::E_SHORTCUT_PARSE, format!("{accelerator}: {e}")))?;
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    gs.register(shortcut)
        .map_err(|e| errcode::ec(errcode::E_SHORTCUT_REGISTER, e))?;
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
    let entry = keyring::Entry::new("QuickScribe", &key)
        .map_err(|e| errcode::ec(errcode::E_KEYRING_INIT, e))?;
    if value.is_empty() {
        let _ = entry.delete_credential();
        return Ok(());
    }
    entry
        .set_password(&value)
        .map_err(|e| errcode::ec(errcode::E_SECRET_SAVE, e))
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
    let entry = keyring::Entry::new("QuickScribe", &key)
        .map_err(|e| errcode::ec(errcode::E_KEYRING_INIT, e))?;
    match entry.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(errcode::ec(errcode::E_SECRET_DELETE, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stt_backend_reports_build_variant() {
        // 既定(CPU)ビルドでは variant="cpu"・gpu_available=false。
        // cuda feature 有効時は variant="cuda"・gpu_available=ドライバ実在に依存（ADR-0027契約）。
        let info = stt_backend();
        let expected = if cfg!(feature = "cuda") { "cuda" } else { "cpu" };
        assert_eq!(info.variant, expected);
        if !cfg!(feature = "cuda") {
            assert!(!info.gpu_available, "CPUビルドではGPU利用不可と報告する");
        }
    }

    #[test]
    fn check_audio_extension_accepts_supported_rejects_others() {
        use std::path::Path;
        assert!(check_audio_extension(Path::new("a.mp3")).is_ok());
        assert!(
            check_audio_extension(Path::new("A.WAV")).is_ok(),
            "大文字も許容"
        );
        assert!(
            check_audio_extension(Path::new("a.opus")).is_ok(),
            ".opus はデコード対応済(#560)のため受理する"
        );
        let err = check_audio_extension(Path::new("a.txt")).unwrap_err();
        assert!(
            err.starts_with(errcode::E_UNSUPPORTED_AUDIO_EXT),
            "安定コードを返す: {err}"
        );
        assert!(err.contains("mp3"), "対応形式一覧を detail に含む: {err}");
        assert!(
            check_audio_extension(Path::new("noext")).is_err(),
            "拡張子なしは弾く"
        );
    }

    #[test]
    fn check_input_size_accepts_within_limit_and_rejects_over() {
        assert!(check_input_size(0).is_ok());
        assert!(check_input_size(MAX_INPUT_BYTES).is_ok());
        let err = check_input_size(MAX_INPUT_BYTES + 1).unwrap_err();
        assert!(
            err.starts_with(errcode::E_FILE_TOO_LARGE),
            "安定コードを返す: {err}"
        );
        assert!(err.contains("500MB"), "上限を detail に含む: {err}");
    }

    /// TS/Rust の対応音声形式リストの契約テスト（#18 / 監査項目: SUPPORTED_AUDIO_EXTS 乖離）。
    /// フロント src/lib/constants.ts の SUPPORTED_AUDIO_EXTS と完全一致（順序含む）を検証する。
    #[test]
    fn supported_audio_exts_match_frontend() {
        let ts = include_str!("../../src/lib/constants.ts");
        let line = ts
            .lines()
            .find(|l| l.contains("SUPPORTED_AUDIO_EXTS") && l.contains('['))
            .expect("constants.ts に SUPPORTED_AUDIO_EXTS の定義が見つからない");
        let start = line.find('[').unwrap();
        let end = line.rfind(']').unwrap();
        let front: Vec<&str> = line[start + 1..end]
            .split(',')
            .map(|s| s.trim().trim_matches('"'))
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(
            front, SUPPORTED_AUDIO_EXTS,
            "フロント(constants.ts)とRust(lib.rs)の対応形式が乖離している"
        );
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

    // ─── ここから コマンド層のテスト（tauri mock runtime / 監査項目12） ───
    use crate::testhttp::{env_scope, serve, set_envs, Route};

    /// アプリと同じ管理状態を備えた mock runtime アプリを組み立てる。
    fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .manage(AppSettings::default())
            .manage(SttState::default())
            .manage(record::RecorderState::default())
            .manage(TrayTexts::default())
            .manage(JobState::default())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("mock app")
    }

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("qs_lib_{}_{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// dir 配下に predicate を満たすファイル名が現れるまで待つ（非同期パイプラインの完了待ち）。
    fn wait_for_file(dir: &std::path::Path, pred: impl Fn(&str) -> bool) -> bool {
        for _ in 0..300 {
            if let Ok(rd) = std::fs::read_dir(dir) {
                for e in rd.flatten() {
                    if pred(&e.file_name().to_string_lossy()) {
                        return true;
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        false
    }

    #[test]
    fn settings_commands_update_managed_state() {
        let app = mock_app();
        set_save_settings(
            app.state(),
            Some("/tmp/qs-x".into()),
            true,
            "opus".into(),
            false,
            Some("md".into()),
        )
        .unwrap();
        let s = current_settings(app.handle());
        assert_eq!(s.save_dir.as_deref(), Some("/tmp/qs-x"));
        assert!(s.save_audio);
        assert_eq!(s.audio_format, "opus");
        assert!(!s.keep_text);
        assert_eq!(s.output_format, "md");
        // 空白のみの保存先は未設定扱い / 出力形式 None は既存値を維持。
        set_save_settings(app.state(), Some("  ".into()), false, "wav".into(), true, None).unwrap();
        let s = current_settings(app.handle());
        assert!(s.save_dir.is_none());
        assert_eq!(s.output_format, "md", "None は上書きしない");

        set_stt_settings(app.state(), "azure".into(), "m1".into(), "k1".into(), Some("res".into()), None)
            .unwrap();
        let st = current_stt_settings(app.handle());
        assert_eq!(st.provider, "azure");
        assert_eq!(st.model, "m1");
        assert_eq!(st.api_key, "k1");
        assert_eq!(st.azure_resource, "res");
    }

    #[test]
    fn save_note_list_entries_and_open_vault_roundtrip() {
        let app = mock_app();
        let dir = tmp_dir("vault");
        set_save_settings(
            app.state(),
            Some(dir.to_string_lossy().into_owned()),
            false,
            "wav".into(),
            true,
            Some("md".into()),
        )
        .unwrap();
        let path = save_note(
            app.handle().clone(),
            "こころのメモ".into(),
            Some(vec!["内省".into()]),
            None,
        )
        .unwrap();
        assert!(path.ends_with(".md"));
        let entries = list_entries(app.handle().clone()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, "note");
        assert_eq!(entries[0].tags, vec!["内省"]);
        // open_vault はフォルダを必ず作る（ファイルマネージャ起動の成否は環境依存で不問）。
        let _ = open_vault(app.handle().clone());
        assert!(dir.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_note_with_transcript_kind_saves_as_transcript() {
        // #599: 用語補正済みの文字起こしは kind="transcript" で別ファイル保存する(非破壊・原本は残る)。
        let app = mock_app();
        let dir = tmp_dir("vault-corrected");
        set_save_settings(
            app.state(),
            Some(dir.to_string_lossy().into_owned()),
            false,
            "wav".into(),
            true,
            Some("md".into()),
        )
        .unwrap();
        let path = save_note(
            app.handle().clone(),
            "補正済み本文".into(),
            None,
            Some("transcript".into()),
        )
        .unwrap();
        assert!(
            path.contains("transcript"),
            "transcript プレフィックスで保存される: {path}"
        );
        let entries = list_entries(app.handle().clone()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, "transcript");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn start_stop_recording_e2e_mode_short_circuits() {
        let app = mock_app();
        let _g = set_envs(&[("QUICKSCRIBE_E2E", "1")]);
        assert!(start_recording(app.state(), None, None).is_ok());
        assert!(tauri::async_runtime::block_on(stop_recording(
            app.handle().clone(),
            app.state(),
            false
        ))
        .is_ok());
    }

    #[test]
    fn start_recording_rejects_double_start() {
        let app = mock_app();
        let _g = env_scope(&[], &["QUICKSCRIBE_E2E"]);
        // 録音中の状態を偽キャプチャで再現 → 二重開始は安定コードで拒否。
        *app.state::<record::RecorderState>().current.lock().unwrap() =
            Some(record::test_recording(vec![(vec![0.0; 16], 16000, 1)]));
        let err = start_recording(app.state(), None, None).unwrap_err();
        assert_eq!(err, errcode::E_ALREADY_RECORDING);
    }

    #[test]
    fn stop_recording_errors_without_active_recording_or_speech() {
        let app = mock_app();
        let _g = env_scope(&[], &["QUICKSCRIBE_E2E"]);
        // 録音していない。
        let err = tauri::async_runtime::block_on(stop_recording(
            app.handle().clone(),
            app.state(),
            false,
        ))
        .unwrap_err();
        assert_eq!(err, errcode::E_NOT_RECORDING);
        // 空の録音。
        *app.state::<record::RecorderState>().current.lock().unwrap() =
            Some(record::test_recording(vec![(vec![], 16000, 1)]));
        let err = tauri::async_runtime::block_on(stop_recording(
            app.handle().clone(),
            app.state(),
            false,
        ))
        .unwrap_err();
        assert_eq!(err, errcode::E_EMPTY_RECORDING);
    }

    #[test]
    fn stop_recording_pipeline_transcribes_and_saves_audio() {
        // 停止 → クラウドSTT(モック) → transcript保存 → 録音音声(opus)保存 の貫通検証。
        let (base, _) = serve(vec![Route::json(
            "/v1/listen",
            200,
            r#"{"results":{"channels":[{"alternatives":[{"transcript":"クラウド文字起こし"}]}]}}"#,
        )]);
        let app = mock_app();
        let dir = tmp_dir("pipeline");
        let _g = env_scope(&[("QS_TEST_DEEPGRAM_BASE", base.as_str())], &["QUICKSCRIBE_E2E"]);
        set_save_settings(
            app.state(),
            Some(dir.to_string_lossy().into_owned()),
            true,
            "opus".into(),
            true,
            Some("txt".into()),
        )
        .unwrap();
        set_stt_settings(app.state(), "deepgram".into(), "".into(), "dk".into(), None, None).unwrap();
        *app.state::<record::RecorderState>().current.lock().unwrap() =
            Some(record::test_recording(vec![(vec![0.2; 3200], 16000, 1)]));
        tauri::async_runtime::block_on(stop_recording(app.handle().clone(), app.state(), false))
            .unwrap();
        assert!(
            wait_for_file(&dir, |n| n.starts_with("transcript-") && n.ends_with(".txt")),
            "文字起こしテキストが保存される"
        );
        assert!(
            wait_for_file(&dir, |n| n.starts_with("rec-") && n.ends_with(".opus")),
            "録音音声が opus で保存される"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// dir 配下に pred を満たすファイルが少なくとも want 件現れるまで待つ。
    fn wait_for_file_count(dir: &std::path::Path, want: usize, pred: impl Fn(&str) -> bool) -> bool {
        for _ in 0..300 {
            if let Ok(rd) = std::fs::read_dir(dir) {
                let n = rd
                    .flatten()
                    .filter(|e| pred(&e.file_name().to_string_lossy()))
                    .count();
                if n >= want {
                    return true;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        false
    }

    #[test]
    fn drop_orphan_pending_keeps_only_queued_audio() {
        // pending は Queued のジョブ分のみ保持する不変条件（音声バッファのリーク防止 / レビュー指摘）。
        fn work() -> PendingWork {
            PendingWork {
                audio: vec![0.0; 8],
                raw: vec![0.0; 8],
                sample_rate: 16000,
                channels: 1,
                timestamps: false,
            }
        }
        let mut ex = JobExec::default();
        let a = ex.queue.enqueue(0, 1.0);
        let b = ex.queue.enqueue(0, 1.0);
        ex.pending.insert(a, work());
        ex.pending.insert(b, work());
        // a を完了させる（pick 相当で pending から抜く運用だが、ここでは終了状態のみ再現）。
        ex.queue.mark_running(a);
        ex.queue.mark_done(a);
        ex.drop_orphan_pending();
        // 終了した a の pending は解放され、まだ Queued の b は残る。
        assert!(!ex.pending.contains_key(&a), "終了ジョブの音声は解放される");
        assert!(ex.pending.contains_key(&b), "Queuedジョブの音声は保持される");
        // b もキャンセル相当で Queued を外れたら解放される（Phase3 の cancel 安全性）。
        ex.queue.cancel(b);
        ex.drop_orphan_pending();
        assert!(ex.pending.is_empty(), "Queued を外れた音声は全て解放される");
    }

    #[test]
    fn stop_recording_queues_multiple_jobs_without_losing_any() {
        // マルチジョブ(ADR-0026 #621): 文字起こし中に次の録音を停止しても、
        // 前ジョブは消えず両方が逐次処理され結果が保存される（取りこぼさない）。
        let (base, _) = serve(vec![Route::json(
            "/v1/listen",
            200,
            r#"{"results":{"channels":[{"alternatives":[{"transcript":"複数ジョブ"}]}]}}"#,
        )]);
        let app = mock_app();
        let dir = tmp_dir("multi-job");
        let _g = env_scope(&[("QS_TEST_DEEPGRAM_BASE", base.as_str())], &["QUICKSCRIBE_E2E"]);
        set_save_settings(
            app.state(),
            Some(dir.to_string_lossy().into_owned()),
            false,
            "wav".into(),
            true,
            Some("txt".into()),
        )
        .unwrap();
        set_stt_settings(app.state(), "deepgram".into(), "".into(), "dk".into(), None, None).unwrap();
        // 2 本の録音を連続で停止＝2 ジョブをキュー投入。
        for _ in 0..2 {
            *app.state::<record::RecorderState>().current.lock().unwrap() =
                Some(record::test_recording(vec![(vec![0.2; 3200], 16000, 1)]));
            tauri::async_runtime::block_on(stop_recording(app.handle().clone(), app.state(), false))
                .unwrap();
        }
        // 両ジョブの transcript が保存される（1件も失わない）。
        assert!(
            wait_for_file_count(&dir, 2, |n| n.starts_with("transcript-")
                && n.ends_with(".txt")),
            "2 ジョブ分の文字起こしが両方保存される"
        );
        // list_jobs は 2 件を返し、いずれも done になる。
        let ok = (0..300).any(|_| {
            let jobs = list_jobs(app.handle().clone());
            jobs.len() == 2
                && jobs
                    .iter()
                    .all(|j| j.status == crate::job::JobStatus::Done)
                || {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    false
                }
        });
        assert!(ok, "list_jobs が 2 件・全て done を返す");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn transcribe_file_validates_and_transcribes_via_cloud() {
        let app = mock_app();
        // 非対応拡張子。
        let err = tauri::async_runtime::block_on(transcribe_file(
            app.handle().clone(),
            "note.txt".into(),
            false,
        ))
        .unwrap_err();
        assert!(err.starts_with(errcode::E_UNSUPPORTED_AUDIO_EXT), "{err}");
        // 実在しないファイル。
        let err = tauri::async_runtime::block_on(transcribe_file(
            app.handle().clone(),
            "missing.wav".into(),
            false,
        ))
        .unwrap_err();
        assert!(err.starts_with(errcode::E_FILE_OPEN), "{err}");
        // 成功: WAV → デコード → クラウドSTT(モック) → keep_text で保存。
        let (base, _) = serve(vec![Route::json(
            "/v1/listen",
            200,
            r#"{"results":{"channels":[{"alternatives":[{"transcript":"ファイル入力の本文"}]}]}}"#,
        )]);
        let _g = set_envs(&[("QS_TEST_DEEPGRAM_BASE", base.as_str())]);
        let dir = tmp_dir("file-in");
        set_save_settings(
            app.state(),
            Some(dir.to_string_lossy().into_owned()),
            false,
            "wav".into(),
            true,
            Some("txt".into()),
        )
        .unwrap();
        set_stt_settings(app.state(), "deepgram".into(), "".into(), "dk".into(), None, None).unwrap();
        let wav = audio_save::save_wav(&vec![0.1; 1600], 16000, 1, &dir, "input").unwrap();
        let text = tauri::async_runtime::block_on(transcribe_file(
            app.handle().clone(),
            wav.to_string_lossy().into_owned(),
            false,
        ))
        .unwrap();
        assert_eq!(text, "ファイル入力の本文");
        assert!(
            wait_for_file(&dir, |n| n.starts_with("transcript-")),
            "keep_text で文字起こしを保存"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refine_text_command_refines_and_saves_markdown() {
        let app = mock_app();
        let (base, seen) = serve(vec![Route::json(
            ":generateContent",
            200,
            r#"{"candidates":[{"content":{"parts":[{"text":"<journal>整形結果の本文</journal>"}]}}]}"#,
        )]);
        let _g = set_envs(&[("QS_TEST_GEMINI_BASE", base.as_str())]);
        let dir = tmp_dir("refine");
        set_save_settings(
            app.state(),
            Some(dir.to_string_lossy().into_owned()),
            false,
            "wav".into(),
            true,
            Some("txt".into()),
        )
        .unwrap();
        let params = RefineTextParams {
            text: "生の文字起こし".into(),
            provider: "gemini".into(),
            api_key: "k".into(),
            model: "".into(), // 空はフォールバック既定モデルを使う。
            style: "summary".into(),
            custom_instruction: None,
            tags: Some(vec!["夜".into()]),
            save: Some(true),
            region: None,
            workspace_id: None,
            auth_mode: None,
            aws_access_key: None,
            aws_secret_key: None,
            aws_session_token: None,
            output_lang: None,
            base_url: None,
        };
        let out =
            tauri::async_runtime::block_on(refine_text(app.handle().clone(), params)).unwrap();
        assert_eq!(out, "整形結果の本文");
        assert!(
            seen.lock().unwrap()[0].contains("gemini-flash-latest"),
            "モデル未指定は既定へフォールバック"
        );
        // 整形結果は出力形式設定に依らず常に .md で保存。
        assert!(wait_for_file(&dir, |n| n.starts_with("refined-") && n.ends_with(".md")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refine_text_command_builds_aws_config_for_sigv4() {
        let app = mock_app();
        let (base, seen) = serve(vec![Route::json(
            "/invoke",
            200,
            r#"{"content":[{"type":"text","text":"<journal>AWS整形</journal>"}]}"#,
        )]);
        let _g = set_envs(&[("QS_TEST_BEDROCK_BASE", base.as_str())]);
        let params = RefineTextParams {
            text: "本文".into(),
            provider: "bedrock".into(),
            api_key: "".into(),
            model: "anthropic.claude-x".into(),
            style: "structured".into(),
            custom_instruction: None,
            tags: None,
            save: Some(false), // 一時結果は保管庫を汚さない。
            region: Some("ap-northeast-1".into()),
            workspace_id: None,
            auth_mode: Some("sigv4".into()),
            aws_access_key: Some("AKIA123".into()),
            aws_secret_key: Some("secret".into()),
            aws_session_token: None,
            output_lang: None,
            base_url: None,
        };
        let out =
            tauri::async_runtime::block_on(refine_text(app.handle().clone(), params)).unwrap();
        assert_eq!(out, "AWS整形");
        let req = seen.lock().unwrap()[0].clone();
        assert!(req.contains("/model/anthropic.claude-x/invoke"), "{req}");
        assert!(req.to_ascii_lowercase().contains("x-amz-date"), "SigV4署名が付く: {req}");
    }

    #[test]
    fn resolve_model_falls_back_to_default_on_failure() {
        // 鍵なし＝一覧取得不可 → プロバイダ既定へフォールバック（UIを止めない）。
        let m = tauri::async_runtime::block_on(resolve_model("anthropic".into(), "".into()))
            .unwrap();
        assert_eq!(m, "claude-sonnet-4-6");
        let m = tauri::async_runtime::block_on(resolve_model("".into(), "".into())).unwrap();
        assert_eq!(m, "gemini-flash-latest");
        // ヘルパの分岐も固定。
        assert_eq!(default_model_for("openai"), "gpt-4o");
        assert!(is_aws_provider("bedrock"));
        assert!(!is_aws_provider("gemini"));
    }

    #[test]
    fn read_text_file_reads_or_errors_with_stable_code() {
        let p = std::env::temp_dir().join(format!("qs_read_{}.txt", std::process::id()));
        std::fs::write(&p, "テキスト内容").unwrap();
        assert_eq!(read_text_file(p.to_string_lossy().into_owned()).unwrap(), "テキスト内容");
        let _ = std::fs::remove_file(&p);
        let err = read_text_file("qs-definitely-missing.txt".into()).unwrap_err();
        assert!(err.starts_with(errcode::E_TEXT_READ), "{err}");
    }

    #[test]
    fn recording_tray_image_draws_red_disc_with_transparent_corners() {
        let img = recording_tray_image();
        assert_eq!(img.width(), 32);
        assert_eq!(img.height(), 32);
        let rgba = img.rgba();
        let px = |x: usize, y: usize| {
            let i = (y * 32 + x) * 4;
            (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3])
        };
        assert_eq!(px(16, 16), (0xE0, 0x20, 0x20, 0xFF), "中心は赤");
        assert_eq!(px(0, 0).3, 0, "角は透明");
    }

    #[test]
    fn set_tray_texts_updates_tooltip_state() {
        let app = mock_app();
        set_tray_texts(
            app.handle().clone(),
            "Record".into(),
            "Show".into(),
            "Quit".into(),
            "QuickScribe — recording".into(),
            "QuickScribe — idle".into(),
        );
        let g = app.state::<TrayTexts>().inner.lock().unwrap().clone();
        assert_eq!(g.tooltip_recording, "QuickScribe — recording");
        assert_eq!(g.tooltip_idle, "QuickScribe — idle");
    }

    #[test]
    fn taskbar_and_startup_commands_are_safe_no_ops_here() {
        // 非Windows/非perf環境では副作用なく完走する（Windowsでは実装へ委譲）。
        set_taskbar_shortcut("Ctrl+Shift+R".into());
        set_taskbar_widget(true);
        set_taskbar_widget(false);
        let _g = env_scope(&[], &["QS_PERF_STARTUP"]);
        report_startup(); // 環境変数なし → 即 return。
    }

    #[test]
    fn secret_commands_tolerate_headless_keyring() {
        // 空値の保存は「削除」扱いで常に成功（ヘッドレスCIでも安全）。
        assert!(set_secret("qs-test-empty-secret".into(), String::new()).is_ok());
        // 未設定キーの取得は None。
        assert!(get_secret("qs-test-definitely-missing".into()).is_none());
        // 未設定キーの削除は OK か、セキュアストレージ不在の安定コード。
        match delete_secret("qs-test-definitely-missing".into()) {
            Ok(()) => {}
            Err(e) => assert!(e.starts_with(errcode::E_SECRET_DELETE), "{e}"),
        }
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
/// プロセス起動時刻（run() 入口）。起動時間ベンチ(#403)用。
static APP_START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// 起動時間ベンチ(#403 案A): フロントの ready(onMount)時に呼ばれ、run() 開始からの経過(ms)を
/// ログへ書く。`QS_PERF_STARTUP` 設定時のみ動作し、通常運用ではオーバーヘッド無し（即return）。
/// CI(perf.yml)が xvfb 起動後に `%LOCALAPPDATA%\QuickScribe\logs\perf-startup.txt` を読み取る。
#[tauri::command]
fn report_startup() {
    if std::env::var("QS_PERF_STARTUP").is_err() {
        return;
    }
    let Some(start) = APP_START.get() else {
        return;
    };
    let ms = start.elapsed().as_millis();
    if let Some(dir) = dirs::data_local_dir().map(|d| d.join("QuickScribe").join("logs")) {
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("perf-startup.txt"), format!("startup_ms={ms}\n"));
    }
    eprintln!("startup_ms={ms}");
}

pub fn run() {
    // プロセス起動時刻を記録（起動時間ベンチ #403）。以降の ready で経過を測る。
    let _ = APP_START.set(std::time::Instant::now());
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
        // 習慣ナッジ用のローカル通知（S9.4 #58 / opt-in・サーバー無し）。
        .plugin(tauri_plugin_notification::init())
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
        .manage(TrayTexts::default())
        .manage(JobState::default())
        .invoke_handler(tauri::generate_handler![
            save_note,
            open_vault,
            list_entries,
            transcribe_file,
            list_audio_sources,
            list_whisper_models,
            stt_backend,
            start_recording,
            stop_recording,
            list_jobs,
            resolve_model,
            refine_text,
            read_text_file,
            set_record_shortcut,
            set_save_settings,
            set_stt_settings,
            set_recording_overlay,
            set_tray_texts,
            set_taskbar_shortcut,
            set_taskbar_widget,
            set_secret,
            get_secret,
            delete_secret,
            report_startup
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
            // 初期文言は日本語既定。フロントが起動直後に set_tray_texts で現在のUI言語へ更新する。
            let record_i = MenuItem::with_id(app, "record", "録音開始/停止", true, None::<&str>)?;
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
