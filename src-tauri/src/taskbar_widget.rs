// Windows: タスクバーの上に重ねる「独立した最前面オーバーレイウィンドウ」で
// 録音/停止・ウィンドウ表示ボタンを常時表示する。
//
// 重要(win11-taskbar-embed-rootcause 調査): Win11 22H2+ のタスクバーは XAML Islands で
// 再実装されており、SetParent による子ウィンドウ埋め込みは描画されないことがある。
// そこで子埋め込みをやめ、Shell_TrayWnd の矩形上に WS_EX_TOPMOST|TOOLWINDOW|NOACTIVATE の
// 独立トップレベルウィンドウを座標計算で重ねる（プロセス境界/XAMLに非依存＝Win11で堅い）。
// どこで失敗するか切り分けるため、各ステップを診断ログ(ドキュメント/QuickScribe/taskbar-diag.log)へ出力する。

use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::OnceLock;

use tauri::{Emitter, Manager};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, Ellipse, EndPaint, FillRect, FrameRect,
    InvalidateRect, Rectangle, SelectObject, HGDIOBJ, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, FindWindowExW, FindWindowW, GetClientRect, GetWindowRect,
    IsWindowVisible, LoadCursorW, RegisterClassW, SetTimer, SetWindowPos, ShowWindow, HWND_TOPMOST,
    IDC_ARROW, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_SHOWNOACTIVATE, WM_LBUTTONUP, WM_PAINT, WM_TIMER,
    WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

static RECORDING: AtomicBool = AtomicBool::new(false);
static WIDGET_HWND: AtomicIsize = AtomicIsize::new(0);
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();

const WIDTH: i32 = 60; // 2ボタン分
const TIMER_ID: usize = 1;

/// 診断ログを ドキュメント/QuickScribe/taskbar-diag.log に追記する。
fn diag(msg: &str) {
    if let Some(doc) = dirs::document_dir() {
        let dir = doc.join("QuickScribe");
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("taskbar-diag.log"))
        {
            use std::io::Write;
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(f, "[{ts}] {msg}");
        }
    }
}

/// タスクバーにオーバーレイウィジェットを設置する（setup から呼ぶ。UIスレッド上）。
pub fn install(app: tauri::AppHandle) {
    let _ = APP.set(app);
    unsafe { create_widget() };
}

/// 録音状態を更新し、ウィジェットを再描画する。
pub fn set_recording(recording: bool) {
    RECORDING.store(recording, Ordering::Relaxed);
    let h = WIDGET_HWND.load(Ordering::Relaxed);
    if h != 0 {
        unsafe {
            let _ = InvalidateRect(Some(HWND(h as *mut _)), None, true.into());
        }
    }
}

unsafe fn create_widget() {
    diag("create_widget: start");
    let hinstance = match GetModuleHandleW(None) {
        Ok(h) => HINSTANCE(h.0),
        Err(e) => {
            diag(&format!("GetModuleHandleW failed: {e}"));
            return;
        }
    };
    let class = w!("QuickScribeTaskbarWidget");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: hinstance,
        lpszClassName: class,
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        ..Default::default()
    };
    RegisterClassW(&wc);

    let taskbar = match FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) {
        Ok(h) => h,
        Err(e) => {
            diag(&format!("FindWindow(Shell_TrayWnd) failed: {e}"));
            return;
        }
    };
    diag(&format!("Shell_TrayWnd hwnd = {:?}", taskbar.0));

    // 独立トップレベルの最前面ツールウィンドウ（子ではない）。
    let hwnd = match CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED,
        class,
        w!("QuickScribe"),
        WS_POPUP,
        0,
        0,
        WIDTH,
        40,
        None, // 親なし＝トップレベル
        None,
        Some(hinstance),
        None,
    ) {
        Ok(h) => h,
        Err(e) => {
            diag(&format!("CreateWindowExW failed: {e}"));
            return;
        }
    };
    WIDGET_HWND.store(hwnd.0 as isize, Ordering::Relaxed);
    diag(&format!("created widget hwnd = {:?}", hwnd.0));

    // 不透明レイヤード（全面不透明・自前WM_PAINT描画。透明度はLWA未設定で完全不透明）。
    use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LWA_ALPHA};
    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);

    reposition(hwnd);
    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    SetTimer(Some(hwnd), TIMER_ID, 1000, None);
    diag(&format!("after show: visible = {}", IsWindowVisible(hwnd).as_bool()));
}

/// Shell_TrayWnd の矩形上（通知領域の左）に、スクリーン座標で重ねる。
unsafe fn reposition(hwnd: HWND) {
    let taskbar = match FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) {
        Ok(h) => h,
        Err(_) => return,
    };
    let mut tb = RECT::default();
    if GetWindowRect(taskbar, &mut tb).is_err() {
        return;
    }
    let height = (tb.bottom - tb.top).max(24);

    // 通知領域(TrayNotifyWnd)の左に配置（スクリーン座標）。
    let x = match FindWindowExW(Some(taskbar), None, w!("TrayNotifyWnd"), PCWSTR::null()) {
        Ok(t) => {
            let mut tr = RECT::default();
            if GetWindowRect(t, &mut tr).is_ok() {
                tr.left - WIDTH - 8
            } else {
                tb.right - WIDTH - 200
            }
        }
        Err(_) => tb.right - WIDTH - 200,
    };
    let _ = SetWindowPos(
        hwnd,
        Some(HWND_TOPMOST),
        x.max(tb.left),
        tb.top,
        WIDTH,
        height,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            paint(hwnd);
            LRESULT(0)
        }
        WM_TIMER => {
            reposition(hwnd);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            if let Some(app) = APP.get() {
                if x < WIDTH / 2 {
                    let _ = app.emit("toggle-record", ());
                } else if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn paint(hwnd: HWND) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut rc = RECT::default();
    let _ = GetClientRect(hwnd, &mut rc);

    // 背景（タスクバーに馴染むダーク）。
    let bg = CreateSolidBrush(COLORREF(0x0020_2020));
    FillRect(hdc, &rc, bg);
    let _ = DeleteObject(HGDIOBJ(bg.0));

    let cy = rc.bottom / 2;
    let half = WIDTH / 2;
    let recording = RECORDING.load(Ordering::Relaxed);

    // 左ボタン: 録音中=白い四角(停止)、停止中=赤い丸(録音開始)。
    let bx = half / 2;
    if recording {
        let brush = CreateSolidBrush(COLORREF(0x00FF_FFFF));
        let old = SelectObject(hdc, HGDIOBJ(brush.0));
        let _ = Rectangle(hdc, bx - 6, cy - 6, bx + 6, cy + 6);
        SelectObject(hdc, old);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    } else {
        let brush = CreateSolidBrush(COLORREF(0x0020_30E0)); // BGR=赤
        let old = SelectObject(hdc, HGDIOBJ(brush.0));
        let _ = Ellipse(hdc, bx - 7, cy - 7, bx + 7, cy + 7);
        SelectObject(hdc, old);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }

    // 右ボタン: ウィンドウを開く（枠だけ）。
    let ox = half + half / 2;
    let frame = CreateSolidBrush(COLORREF(0x00CC_CCCC));
    let fr = RECT {
        left: ox - 7,
        top: cy - 6,
        right: ox + 7,
        bottom: cy + 6,
    };
    FrameRect(hdc, &fr, frame);
    let _ = DeleteObject(HGDIOBJ(frame.0));

    let _ = EndPaint(hwnd, &ps);
}
