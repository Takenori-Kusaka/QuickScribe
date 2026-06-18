// Windows タスクバーのサムネイルツールバー（ITaskbarList3）に「録音 開始/停止」ボタンを出す。
// Tauri v2 は生 wndproc を公開しないため、ウィンドウを SetWindowSubclass でサブクラス化し、
// "TaskbarButtonCreated" 受信後に ThumbBarAddButtons、WM_COMMAND/THBN_CLICKED で toggle-record を emit する。
// 一次情報: docs/research/sources/windows-thumbnail-toolbar.md
//
// 注: このモジュールは #[cfg(windows)] でのみコンパイルされる。実行時挙動はWindows実機で要確認。

use std::sync::atomic::{AtomicU32, Ordering};

use tauri::Emitter;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{
    DefSubclassProc, SetWindowSubclass, ITaskbarList3, TaskbarList, THBF_ENABLED, THBN_CLICKED,
    THB_FLAGS, THB_ICON, THB_TOOLTIP, THUMBBUTTON,
};
use windows::Win32::UI::WindowsAndMessaging::{
    LoadIconW, PostMessageW, RegisterWindowMessageW, HICON, IDI_APPLICATION, WM_APP, WM_COMMAND,
};

/// 録音トグルボタンのID（WM_COMMAND の LOWORD で判定）。
const TOGGLE_BTN_ID: u32 = 1;

/// 競合保険でボタン追加を再試行させる自分宛メッセージ。
const WM_ADD_THUMBBUTTONS: u32 = WM_APP + 1;

/// "TaskbarButtonCreated" の登録メッセージID（Explorer再起動時にも再送される）。
static TASKBAR_CREATED_MSG: AtomicU32 = AtomicU32::new(0);

/// メインウィンドウにサムネイルツールバーを取り付ける（setup から呼ぶ）。
/// AppHandle は subclass の dwRefData に渡すため Box でリーク（アプリ寿命と同じ）。
pub fn install(window: &tauri::WebviewWindow, app: tauri::AppHandle) {
    let hwnd = match window.hwnd() {
        // 生ポインタ経由で自前 windows クレートの HWND へ再ラップ（型バージョン差異を吸収）。
        Ok(h) => HWND(h.0),
        Err(e) => {
            eprintln!("hwnd取得に失敗（サムネイルボタン無効）: {e}");
            return;
        }
    };
    unsafe {
        let msg = RegisterWindowMessageW(w!("TaskbarButtonCreated"));
        TASKBAR_CREATED_MSG.store(msg, Ordering::Relaxed);
        let refdata: *mut tauri::AppHandle = Box::into_raw(Box::new(app));
        let _ = SetWindowSubclass(hwnd, Some(subclass_proc), 0, refdata as usize);

        // 競合保険: TaskbarButtonCreated を取りこぼしても、表示が落ち着いた頃に
        // 自分宛メッセージを送ってボタン追加を再試行する（HWNDはisizeでスレッドへ渡す）。
        let hwnd_raw = hwnd.0 as isize;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            unsafe {
                let _ = PostMessageW(
                    Some(HWND(hwnd_raw as *mut _)),
                    WM_ADD_THUMBBUTTONS,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
        });
    }
}

/// サブクラスプロシージャ。タスクバーボタン生成→ボタン追加、クリック→toggle-record。
unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uid: usize,
    refdata: usize,
) -> LRESULT {
    let app = &*(refdata as *const tauri::AppHandle);

    let created = TASKBAR_CREATED_MSG.load(Ordering::Relaxed);
    if (created != 0 && msg == created) || msg == WM_ADD_THUMBBUTTONS {
        if let Err(e) = add_buttons(hwnd) {
            eprintln!("サムネイルボタン追加に失敗: {e}");
        }
    }

    if msg == WM_COMMAND {
        let hi = ((wparam.0 >> 16) & 0xFFFF) as u32; // 通知コード
        let lo = (wparam.0 & 0xFFFF) as u32; // ボタンID
        if hi == THBN_CLICKED && lo == TOGGLE_BTN_ID {
            let _ = app.emit("toggle-record", ());
            return LRESULT(0);
        }
    }

    DefSubclassProc(hwnd, msg, wparam, lparam)
}

/// ITaskbarList3 を生成してサムネイルボタンを追加する。
unsafe fn add_buttons(hwnd: HWND) -> windows::core::Result<()> {
    // COM はUIスレッドで既に初期化済みのことが多いが、念のため（既存モードと衝突しても無視）。
    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

    let taskbar: ITaskbarList3 = CoCreateInstance(&TaskbarList, None, CLSCTX_INPROC_SERVER)?;
    taskbar.HrInit()?;

    let mut tip = [0u16; 260];
    for (i, c) in "録音 開始/停止".encode_utf16().enumerate() {
        if i >= tip.len() - 1 {
            break;
        }
        tip[i] = c;
    }

    let buttons = [THUMBBUTTON {
        dwMask: THB_ICON | THB_TOOLTIP | THB_FLAGS,
        iId: TOGGLE_BTN_ID,
        iBitmap: 0,
        hIcon: app_icon(),
        szTip: tip,
        dwFlags: THBF_ENABLED,
    }];
    taskbar.ThumbBarAddButtons(hwnd, &buttons)
}

/// ボタン用アイコン（取得失敗時はnull=アイコン無し表示）。
unsafe fn app_icon() -> HICON {
    LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
}
