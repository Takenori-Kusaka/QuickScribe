// Windows: タスクバーの上に重ねる「独立した最前面オーバーレイウィンドウ」で
// 録音/停止・ウィンドウ表示ボタンを常時表示する。
//
// 重要(win11-taskbar-embed-rootcause 調査): Win11 22H2+ のタスクバーは XAML Islands で
// 再実装されており、SetParent による子ウィンドウ埋め込みは描画されないことがある。
// そこで子埋め込みをやめ、Shell_TrayWnd の矩形上に WS_EX_TOPMOST|TOOLWINDOW|NOACTIVATE の
// 独立トップレベルウィンドウを座標計算で重ねる（プロセス境界/XAMLに非依存＝Win11で堅い）。
// どこで失敗するか切り分けるため、各ステップを診断ログ(ドキュメント/QuickScribe/taskbar-diag.log)へ出力する。

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, Ordering};
use std::sync::{Mutex, OnceLock};

use tauri::{Emitter, Manager};
use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush, DeleteDC,
    DeleteObject, Ellipse, EndPaint, FillRect, InvalidateRect, Rectangle, SelectObject, HGDIOBJ,
    PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_BAR_CLASSES, INITCOMMONCONTROLSEX, NMHDR, NMTTDISPINFOW,
    TOOLTIPS_CLASSW, TTF_SUBCLASS, TTM_ADDTOOLW, TTN_GETDISPINFOW, TTS_ALWAYSTIP, TTS_NOPREFIX,
    TTTOOLINFOW,
};
use windows::Win32::UI::Shell::ExtractIconW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DrawIconEx, FindWindowExW, FindWindowW, GetClientRect,
    GetWindowRect, IsWindowVisible, LoadCursorW, RegisterClassW, SendMessageW, SetTimer,
    SetWindowPos, ShowWindow, CW_USEDEFAULT, DI_NORMAL, HICON, HWND_TOPMOST, IDC_ARROW,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SW_SHOWNOACTIVATE, WINDOW_STYLE, WM_ERASEBKGND,
    WM_LBUTTONUP, WM_NOTIFY, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

static RECORDING: AtomicBool = AtomicBool::new(false);
static WIDGET_HWND: AtomicIsize = AtomicIsize::new(0);
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
/// ツールチップに表示する現在のショートカット（OS表記。フロントから set_shortcut で更新）。
static SHORTCUT: Mutex<String> = Mutex::new(String::new());
/// 右ボタンに描画する QuickScribe アプリアイコン（HICON を isize で保持。0=未取得）。
static WIDGET_ICON: AtomicIsize = AtomicIsize::new(0);
/// 直近に配置した座標・高さ（不要な SetWindowPos の移動＝ちらつきを避けるためのキャッシュ）。
static LAST_X: AtomicI32 = AtomicI32::new(i32::MIN);
static LAST_Y: AtomicI32 = AtomicI32::new(i32::MIN);
static LAST_H: AtomicI32 = AtomicI32::new(i32::MIN);

const TOOL_RECORD: usize = 1;
const TOOL_OPEN: usize = 2;

const WIDTH: i32 = 60; // 2ボタン分
const TIMER_ID: usize = 1;
/// 背景の透過色（クロマキー）。この色で塗った部分はタスクバーが透けて見える＝テーマ非依存。
/// マゼンタはボタン色(赤/白/灰)と被らないため選択。COLORREF は 0x00BBGGRR。
const CHROMA: u32 = 0x00FF_00FF;

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
            // erase=false: 背景消去によるちらつきを避ける（全面をダブルバッファで描き直す）。
            let _ = InvalidateRect(Some(HWND(h as *mut _)), None, false.into());
        }
    }
}

/// 実行ファイル(.exe)に埋め込まれた QuickScribe アイコンを取得してキャッシュする。
/// 汎用の IDI_APPLICATION ではなくアプリ自身のアイコンを使い、右ボタンでアプリ帰属を一目で示す。
unsafe fn load_app_icon(hinstance: HINSTANCE) {
    use std::os::windows::ffi::OsStrExt;
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let wide: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    // ExtractIconW: index 0 = exe の最初のアイコングループ（アプリアイコン）。
    // 戻り値が null(0) / 1(無効) の場合は未取得扱いでフォールバックグリフを使う。
    let icon = ExtractIconW(Some(hinstance), PCWSTR(wide.as_ptr()), 0);
    let v = icon.0 as isize;
    if v != 0 && v != 1 {
        WIDGET_ICON.store(v, Ordering::Relaxed);
        diag("load_app_icon: ok");
    } else {
        diag(&format!("load_app_icon: none (v={v})"));
    }
}

/// ツールチップに表示する現在のショートカット表記（例 "Ctrl+Shift+R"）を更新する。
pub fn set_shortcut(display: String) {
    if let Ok(mut s) = SHORTCUT.lock() {
        *s = display;
    }
}

/// ボタンごとのツールチップ文言（動作＋現在のショートカット＋アプリ名）。
/// WCAG/Microsoft Fluent: アイコンのみボタンにはツールチップとショートカット併記が必要。
fn tooltip_text(id: usize) -> String {
    let sc = SHORTCUT
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default();
    if id == TOOL_RECORD {
        let action = if RECORDING.load(Ordering::Relaxed) {
            "録音を停止"
        } else {
            "録音を開始"
        };
        if sc.is_empty() {
            format!("{action} ・ QuickScribe")
        } else {
            format!("{action} ・ {sc} ・ QuickScribe")
        }
    } else {
        "QuickScribe のウィンドウを開く".to_string()
    }
}

/// ツールチップに1ボタン分の矩形ツールを登録する（テキストはコールバックで動的生成）。
unsafe fn add_tool(tt: HWND, parent: HWND, id: usize, left: i32, right: i32) {
    let mut ti = TTTOOLINFOW {
        cbSize: std::mem::size_of::<TTTOOLINFOW>() as u32,
        uFlags: TTF_SUBCLASS,
        hwnd: parent,
        uId: id,
        rect: RECT {
            left,
            top: 0,
            right,
            bottom: 60,
        },
        // LPSTR_TEXTCALLBACKW = ((LPWSTR)-1)。windows crate に定数が無いため値を直接構築する
        // （TTN_GETDISPINFOW でテキストを動的供給する合図）。
        lpszText: PWSTR(-1isize as *mut u16),
        ..Default::default()
    };
    let _ = SendMessageW(
        tt,
        TTM_ADDTOOLW,
        None,
        Some(LPARAM(&mut ti as *mut _ as isize)),
    );
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

    // クロマキー透過: 背景(CHROMA)で塗った部分を透明にし、タスクバーを透けさせる。
    // ボタンだけが浮いて見えるためタスクバー色/テーマに依存しない。
    use windows::Win32::UI::WindowsAndMessaging::{SetLayeredWindowAttributes, LWA_COLORKEY};
    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(CHROMA), 0, LWA_COLORKEY);

    // ツールチップ（アイコンのみボタンの説明＋現在ショートカット＋アプリ名 / WCAG・Fluent準拠）。
    let icc = INITCOMMONCONTROLSEX {
        dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_BAR_CLASSES,
    };
    let _ = InitCommonControlsEx(&icc);
    if let Ok(tt) = CreateWindowExW(
        WS_EX_TOPMOST,
        TOOLTIPS_CLASSW,
        PCWSTR::null(),
        // TTS_* は u32、WS_POPUP は WINDOW_STYLE のため、ビット OR して WINDOW_STYLE で包む。
        WINDOW_STYLE(WS_POPUP.0 | TTS_ALWAYSTIP | TTS_NOPREFIX),
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        Some(hwnd),
        None,
        Some(hinstance),
        None,
    ) {
        add_tool(tt, hwnd, TOOL_RECORD, 0, WIDTH / 2);
        add_tool(tt, hwnd, TOOL_OPEN, WIDTH / 2, WIDTH);
    }

    // 右ボタン用にアプリアイコンを取得（失敗時はフォールバックグリフ）。
    load_app_icon(hinstance);

    reposition(hwnd);
    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    // 300ms間隔: タスクバー操作で z-order が落ちても素早く最前面へ復帰させる
    // （ダブルバッファ描画なので頻繁な再描画でもちらつかない）。
    SetTimer(Some(hwnd), TIMER_ID, 300, None);
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
    let target_x = x.max(tb.left);
    let target_y = tb.top;
    let moved = target_x != LAST_X.load(Ordering::Relaxed)
        || target_y != LAST_Y.load(Ordering::Relaxed)
        || height != LAST_H.load(Ordering::Relaxed);
    if moved {
        // 位置・サイズが変わったときだけ実際に移動する（毎回移動すると再描画でちらつく）。
        LAST_X.store(target_x, Ordering::Relaxed);
        LAST_Y.store(target_y, Ordering::Relaxed);
        LAST_H.store(height, Ordering::Relaxed);
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            target_x,
            target_y,
            WIDTH,
            height,
            SWP_NOACTIVATE,
        );
    } else {
        // 位置不変でも最前面だけ維持する。タスクバー操作で z-order が落ち
        // 一時的にボタンが隠れるのを素早く復帰させる（移動しないのでちらつかない）。
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            paint(hwnd);
            LRESULT(0)
        }
        // 背景消去はしない（全面をダブルバッファで描き直すため）。既定の消去によるちらつき防止。
        WM_ERASEBKGND => LRESULT(1),
        WM_TIMER => {
            reposition(hwnd);
            LRESULT(0)
        }
        WM_NOTIFY => {
            // ツールチップの表示テキスト要求に動的応答（動作＋ショートカット＋アプリ名）。
            let nmhdr = lparam.0 as *const NMHDR;
            if !nmhdr.is_null() && (*nmhdr).code == TTN_GETDISPINFOW as u32 {
                let di = lparam.0 as *mut NMTTDISPINFOW;
                let text = tooltip_text((*nmhdr).idFrom);
                let mut buf: Vec<u16> = text.encode_utf16().collect();
                buf.truncate(79);
                buf.push(0);
                let n = buf.len();
                // di は system 提供の有効ポインタ。raw pointer 経由の暗黙 autoref
                // (dangerous_implicit_autorefs)を避けるため &raw mut で生ポインタを得て書き込む。
                let sz = (&raw mut (*di).szText) as *mut u16;
                std::ptr::copy_nonoverlapping(buf.as_ptr(), sz, n);
                (*di).lpszText = PWSTR(sz);
            }
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
    let w = rc.right.max(1);
    let h = rc.bottom.max(1);

    // ダブルバッファ: いったんメモリDCに全部描いてから一括転送する。
    // 「背景塗り→各ボタン描画」の途中状態が画面に出ないため、録音トグルや
    // タスクバー操作に伴う再描画でちらつかない。
    let mem = CreateCompatibleDC(Some(hdc));
    let bmp = CreateCompatibleBitmap(hdc, w, h);
    let old_bmp = SelectObject(mem, HGDIOBJ(bmp.0));

    // 背景はクロマキー色で塗る → 透過してタスクバーが見える（ボタンだけ浮く）。
    let bg = CreateSolidBrush(COLORREF(CHROMA));
    FillRect(mem, &rc, bg);
    let _ = DeleteObject(HGDIOBJ(bg.0));

    let cy = h / 2;
    let half = WIDTH / 2;
    let recording = RECORDING.load(Ordering::Relaxed);

    // 左ボタン: 録音中=白い四角(停止)、停止中=赤い丸(録音開始)。
    let bx = half / 2;
    if recording {
        let brush = CreateSolidBrush(COLORREF(0x00FF_FFFF));
        let old = SelectObject(mem, HGDIOBJ(brush.0));
        let _ = Rectangle(mem, bx - 6, cy - 6, bx + 6, cy + 6);
        SelectObject(mem, old);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    } else {
        let brush = CreateSolidBrush(COLORREF(0x0020_30E0)); // BGR=赤
        let old = SelectObject(mem, HGDIOBJ(brush.0));
        let _ = Ellipse(mem, bx - 7, cy - 7, bx + 7, cy + 7);
        SelectObject(mem, old);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }

    // 右ボタン: QuickScribe のウィンドウを開く。アプリアイコンを描いてアプリ帰属を
    // 一目で示し、左の停止(■)と確実に区別する。アイコン未取得時のみフォールバックで
    // 「ウィンドウ風」グリフ（本体＋タイトルバー帯）を描く。
    let ox = half + half / 2;
    let icon = WIDGET_ICON.load(Ordering::Relaxed);
    if icon != 0 {
        let sz = (h - 6).clamp(16, 24);
        let _ = DrawIconEx(
            mem,
            ox - sz / 2,
            (h - sz) / 2,
            HICON(icon as *mut _),
            sz,
            sz,
            0,
            None,
            DI_NORMAL,
        );
    } else {
        let (l, t, r, b) = (ox - 7, cy - 6, ox + 7, cy + 6);
        let body = CreateSolidBrush(COLORREF(0x00E0_E0E0)); // 本体: 明るいグレー
        let old = SelectObject(mem, HGDIOBJ(body.0));
        let _ = Rectangle(mem, l, t, r, b);
        SelectObject(mem, old);
        let _ = DeleteObject(HGDIOBJ(body.0));
        let bar_rc = RECT { left: l, top: t, right: r, bottom: t + 3 };
        let bar = CreateSolidBrush(COLORREF(0x0080_8080)); // タイトルバー帯: 濃いグレー
        FillRect(mem, &bar_rc, bar);
        let _ = DeleteObject(HGDIOBJ(bar.0));
    }

    // メモリDC → 画面へ一括転送。
    let _ = BitBlt(hdc, 0, 0, w, h, Some(mem), 0, 0, SRCCOPY);

    SelectObject(mem, old_bmp);
    let _ = DeleteObject(HGDIOBJ(bmp.0));
    let _ = DeleteDC(mem);
    let _ = EndPaint(hwnd, &ps);
}
