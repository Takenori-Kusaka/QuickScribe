// Windows: タスクバー(Shell_TrayWnd)に自前ウィンドウを子として埋め込み、
// 「録音/停止」トグルボタンと「ウィンドウを開く」ボタンを常時表示する。
// 手本: TrafficMonitor（FindWindow("Shell_TrayWnd") + 親=タスクバーで子ウィンドウ作成 + 座標追従）。
// 一次情報: docs/research/sources/windows-taskbar-widget-impl.md
//
// 注: タスクバーへの埋め込みは Windows 非公式の手法。セキュリティソフトや Windows 更新で
//     壊れうるため、実機での反復調整が前提。コンパイルは CI(windows) で検証する。

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
    LoadCursorW, MoveWindow, RegisterClassW, SetTimer, IDC_ARROW, WINDOW_EX_STYLE, WM_LBUTTONUP,
    WM_PAINT, WM_TIMER, WNDCLASSW, WS_CHILD, WS_VISIBLE,
};

/// 録音中フラグ（描画に使用）。
static RECORDING: AtomicBool = AtomicBool::new(false);
/// ウィジェットのHWND（isize）。状態変化時に再描画するため保持。
static WIDGET_HWND: AtomicIsize = AtomicIsize::new(0);
/// emit / ウィンドウ表示に使う AppHandle。
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();

const WIDTH: i32 = 60; // 2ボタン分の幅
const TIMER_ID: usize = 1;

/// タスクバーにウィジェットを埋め込む（setup から呼ぶ。UIスレッド上）。
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
    let hinstance = match GetModuleHandleW(None) {
        Ok(h) => HINSTANCE(h.0),
        Err(_) => return,
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
        Err(_) => return,
    };
    let mut tb = RECT::default();
    let _ = GetWindowRect(taskbar, &mut tb);
    let height = (tb.bottom - tb.top).max(24);

    // 親=タスクバー + WS_CHILD でタスクバーの子ウィンドウとして埋め込む。
    let hwnd = match CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class,
        w!("QuickScribe"),
        WS_CHILD | WS_VISIBLE,
        0,
        0,
        WIDTH,
        height,
        Some(taskbar),
        None,
        Some(hinstance),
        None,
    ) {
        Ok(h) => h,
        Err(_) => return,
    };
    WIDGET_HWND.store(hwnd.0 as isize, Ordering::Relaxed);
    reposition(hwnd, taskbar);
    // 1秒ごとに位置を追従（タスクバーサイズ変更/解像度変更対策）。
    SetTimer(Some(hwnd), TIMER_ID, 1000, None);
}

/// 通知領域(TrayNotifyWnd)の左に配置する。
unsafe fn reposition(hwnd: HWND, taskbar: HWND) {
    let mut tb = RECT::default();
    if GetWindowRect(taskbar, &mut tb).is_err() {
        return;
    }
    let height = (tb.bottom - tb.top).max(24);
    let tray = FindWindowExW(Some(taskbar), None, w!("TrayNotifyWnd"), PCWSTR::null());
    let x = match tray {
        Ok(t) => {
            let mut tr = RECT::default();
            if GetWindowRect(t, &mut tr).is_ok() {
                (tr.left - tb.left) - WIDTH - 8
            } else {
                (tb.right - tb.left) - WIDTH - 200
            }
        }
        Err(_) => (tb.right - tb.left) - WIDTH - 200,
    };
    let _ = MoveWindow(hwnd, x.max(0), 0, WIDTH, height, true.into());
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            paint(hwnd);
            LRESULT(0)
        }
        WM_TIMER => {
            if let Ok(tb) = FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) {
                reposition(hwnd, tb);
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            if let Some(app) = APP.get() {
                if x < WIDTH / 2 {
                    // 左ボタン: 録音 開始/停止
                    let _ = app.emit("toggle-record", ());
                } else if let Some(win) = app.get_webview_window("main") {
                    // 右ボタン: ウィンドウを開く
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
