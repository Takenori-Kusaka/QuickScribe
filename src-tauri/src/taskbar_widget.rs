// Windows: タスクバーの上に重ねる「独立した最前面オーバーレイウィンドウ」で
// 録音/停止・ウィンドウ表示ボタンを常時表示する。
//
// 重要(win11-taskbar-embed-rootcause 調査): Win11 22H2+ のタスクバーは XAML Islands で
// 再実装されており、SetParent による子ウィンドウ埋め込みは描画されないことがある。
// そこで子埋め込みをやめ、Shell_TrayWnd の矩形上に WS_EX_TOPMOST|TOOLWINDOW|NOACTIVATE の
// 独立トップレベルウィンドウを座標計算で重ねる（プロセス境界/XAMLに非依存＝Win11で堅い）。
//
// 透過方式: クロマキー(LWA_COLORKEY)はアイコンのアンチエイリアス縁がキー色(マゼンタ)と
// 混ざり「色付きの縁(赤い背景)」が残るため廃止。UpdateLayeredWindow による per-pixel alpha
// (32bpp プリマルチプライド ARGB)で合成し、アイコン本来の透過をそのまま反映する＝縁の滲み無し。
// どこで失敗するか切り分けるため、各ステップを診断ログ(%LOCALAPPDATA%\QuickScribe\logs\taskbar-diag.log)へ出力する。

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, Ordering};
use std::sync::{Mutex, OnceLock};

use tauri::{Emitter, Manager};
use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, EndPaint,
    GetMonitorInfoW, MonitorFromWindow, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    BLENDFUNCTION, DIB_RGB_COLORS, HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_BAR_CLASSES, INITCOMMONCONTROLSEX, NMHDR, NMTTDISPINFOW,
    TOOLTIPS_CLASSW, TTF_SUBCLASS, TTM_ADDTOOLW, TTN_GETDISPINFOW, TTS_ALWAYSTIP, TTS_NOPREFIX,
    TTTOOLINFOW,
};
use windows::Win32::UI::Shell::{
    ExtractIconW, SHAppBarMessage, ABM_GETSTATE, ABS_AUTOHIDE, APPBARDATA,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DrawIconEx, FindWindowExW, FindWindowW, GetClassNameW,
    GetForegroundWindow, GetWindowPlacement, GetWindowRect, IsWindowVisible, LoadCursorW,
    RegisterClassW, SendMessageW, SetTimer, SetWindowPos, ShowWindow, UpdateLayeredWindow,
    CW_USEDEFAULT, DI_NORMAL, HICON, HWND_TOPMOST, IDC_ARROW, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SW_HIDE, SW_SHOWMAXIMIZED, SW_SHOWNOACTIVATE, ULW_ALPHA, WINDOWPLACEMENT,
    WINDOW_STYLE, WM_LBUTTONUP, WM_NOTIFY, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
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
/// フルスクリーンアプリ検出でウィジェットを隠している間 true（重複した Show/Hide 呼び出しを避ける）。
static HIDDEN: AtomicBool = AtomicBool::new(false);
/// ユーザー設定でウィジェット表示が有効か（既定 true。設定でOFFにすると常に隠す）。
static ENABLED: AtomicBool = AtomicBool::new(true);

const TOOL_RECORD: usize = 1;
const TOOL_OPEN: usize = 2;

const WIDTH: i32 = 60; // 2ボタン分
const TIMER_ID: usize = 1;

/// 診断ログを OS の Local データ領域へ追記する（Windows=%LOCALAPPDATA%\QuickScribe\logs、
/// Linux=~/.local/share/QuickScribe/logs、macOS=~/Library/Application Support/QuickScribe/logs）。
/// 内部診断ログはユーザーの出力先(ドキュメント)ではなく、ローミングしない Local 領域へ置くのが
/// 各OSの慣行。旧実装は出力先(ドキュメント/QuickScribe)へ書いていた。
fn diag(msg: &str) {
    if let Some(base) = dirs::data_local_dir() {
        let dir = base.join("QuickScribe").join("logs");
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
        unsafe { render(HWND(h as *mut _)) };
    }
}

/// ウィジェット表示の有効/無効を設定する（設定のトグルから。即時に表示/非表示を反映）。
/// 無効中は WM_TIMER が常に隠す。有効化時は次のタイマーで条件に応じ復帰する。
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
    let h = WIDGET_HWND.load(Ordering::Relaxed);
    if h == 0 {
        return;
    }
    let hwnd = HWND(h as *mut _);
    unsafe {
        if enabled {
            // 即時表示（その後はタイマーがフルスクリーン等で適宜隠す）。
            HIDDEN.store(false, Ordering::Relaxed);
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        } else {
            HIDDEN.store(true, Ordering::Relaxed);
            let _ = ShowWindow(hwnd, SW_HIDE);
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
    let sc = SHORTCUT.lock().map(|s| s.clone()).unwrap_or_default();
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
    let _ = SendMessageW(tt, TTM_ADDTOOLW, None, Some(LPARAM(&mut ti as *mut _ as isize)));
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

    // 独立トップレベルの最前面ツールウィンドウ（子ではない）。WS_EX_LAYERED は
    // UpdateLayeredWindow による per-pixel alpha 合成に必須。
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
    render(hwnd); // 初回の中身を UpdateLayeredWindow で反映
    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    // 300ms間隔: タスクバー操作で z-order が落ちても素早く最前面へ復帰させる。
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
        let size_changed = height != LAST_H.load(Ordering::Relaxed);
        // 位置・サイズが変わったときだけ実際に移動する（毎回移動するとちらつく）。
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
        // 高さが変わると DIB のサイズも変える必要があるため再描画する。
        if size_changed {
            render(hwnd);
        }
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

/// 前面ウィンドウがそのモニタを完全に覆う「フルスクリーン」かを判定する。
/// 動画(YouTube全画面)/ゲーム/プレゼン等のフルスクリーンではタスクバー自体が隠れるため、
/// 最前面オーバーレイを残すとコンテンツの上に被さる。これを検出してウィジェットを隠す。
/// 通常の「最大化」ウィンドウはタスクバーを覆わない(矩形がモニタ全体に達しない)ため false。
unsafe fn foreground_is_fullscreen() -> bool {
    let fg = GetForegroundWindow();
    if fg.0.is_null() {
        return false;
    }
    // デスクトップ/シェル(タスクバー・Progman・WorkerW)やウィジェット自身は全画面扱いしない。
    let mut cls = [0u16; 64];
    let n = GetClassNameW(fg, &mut cls);
    if n > 0 {
        let name = String::from_utf16_lossy(&cls[..n as usize]);
        if matches!(
            name.as_str(),
            "Shell_TrayWnd"
                | "Shell_SecondaryTrayWnd"
                | "Progman"
                | "WorkerW"
                | "QuickScribeTaskbarWidget"
        ) {
            return false;
        }
    }
    let mut wr = RECT::default();
    if GetWindowRect(fg, &mut wr).is_err() {
        return false;
    }
    let mon = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(mon, &mut mi).as_bool() {
        return false;
    }
    let m = mi.rcMonitor;
    // 前面ウィンドウがモニタ矩形を完全に覆う = フルスクリーン。
    wr.left <= m.left && wr.top <= m.top && wr.right >= m.right && wr.bottom >= m.bottom
}

/// タスクバー(Shell_TrayWnd)が画面上に実際に見えているかを判定し、隠れていれば true。
/// ウィジェットはタスクバー上に置く存在なので、タスクバーが見えない時は一緒に隠すべき。
/// これは「自動的に隠す(auto-hide)設定の引っ込み時」や「最大化ウィンドウでタスクバーが
/// 退避した時」もカバーする(フルスクリーン限定だった foreground_is_fullscreen の取りこぼし是正)。
unsafe fn taskbar_offscreen() -> bool {
    let taskbar = match FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) {
        Ok(h) => h,
        Err(_) => return false,
    };
    if !IsWindowVisible(taskbar).as_bool() {
        return true;
    }
    let mut tb = RECT::default();
    if GetWindowRect(taskbar, &mut tb).is_err() {
        return false;
    }
    let mon = MonitorFromWindow(taskbar, MONITOR_DEFAULTTONEAREST);
    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(mon, &mut mi).as_bool() {
        return false;
    }
    let m = mi.rcMonitor;
    // タスクバー矩形とモニタの交差(=画面に見えている部分)の短辺＝厚みを測る。
    // 通常表示なら厚み≈40-48px。auto-hide の引っ込みやフルスクリーンでは画面外へ押し出され
    // 厚みが数px以下になる。4px 未満を「隠れている」とみなす。
    let ix = (tb.right.min(m.right) - tb.left.max(m.left)).max(0);
    let iy = (tb.bottom.min(m.bottom) - tb.top.max(m.top)).max(0);
    ix.min(iy) < 4
}

/// タスクバーが「自動的に隠す(auto-hide)」設定かを AppBar API で直接問い合わせる。
/// 矩形の厚み閾値(taskbar_offscreen)に頼らず状態を読むため、最大化判定と組み合わせると
/// フラッピングしない堅い検出になる(diag ログのhide/show頻発＝閾値ブレの是正)。
unsafe fn taskbar_autohide() -> bool {
    let mut abd = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };
    let state = SHAppBarMessage(ABM_GETSTATE, &mut abd) as u32;
    state & ABS_AUTOHIDE != 0
}

/// 前面ウィンドウが「最大化」状態かを GetWindowPlacement で直接判定する。
/// auto-hide タスクバー＋最大化のときタスクバーは退避するため、ウィジェットを隠すべき。
/// (フルスクリーン判定 foreground_is_fullscreen は auto-hide が1px予約する関係で最大化窓を
///  取りこぼすことがあるため、showCmd で確実に拾う。)
unsafe fn foreground_is_maximized() -> bool {
    let fg = GetForegroundWindow();
    if fg.0.is_null() {
        return false;
    }
    // デスクトップ/シェル/ウィジェット自身は対象外(フルスクリーン判定と同じガード)。
    let mut cls = [0u16; 64];
    let n = GetClassNameW(fg, &mut cls);
    if n > 0 {
        let name = String::from_utf16_lossy(&cls[..n as usize]);
        if matches!(
            name.as_str(),
            "Shell_TrayWnd"
                | "Shell_SecondaryTrayWnd"
                | "Progman"
                | "WorkerW"
                | "QuickScribeTaskbarWidget"
        ) {
            return false;
        }
    }
    let mut wp = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    if GetWindowPlacement(fg, &mut wp).is_err() {
        return false;
    }
    // WINDOWPLACEMENT.showCmd は u32。SW_SHOWMAXIMIZED は SHOW_WINDOW_CMD のため .0 を u32 で比較。
    wp.showCmd == SW_SHOWMAXIMIZED.0 as u32
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            // 内容は UpdateLayeredWindow(render) で合成するため、ここでは検証(validate)のみ。
            let mut ps = PAINTSTRUCT::default();
            let _ = BeginPaint(hwnd, &mut ps);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_TIMER => {
            // ユーザー設定でOFFなら検出より優先して常に隠す（設定で表示/非表示を切替）。
            if !ENABLED.load(Ordering::Relaxed) {
                if !HIDDEN.swap(true, Ordering::Relaxed) {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    diag("hide: widget disabled by setting");
                }
                return LRESULT(0);
            }
            // タスクバーが画面に見えていない時(フルスクリーン/auto-hide引っ込み/最大化での退避)は
            // 最前面オーバーレイを隠してコンテンツを邪魔しない。タスクバーが戻れば復帰させる。
            // 隠す条件(状態を直接読み、閾値ブレ＝フラッピングを避ける):
            //   ・フルスクリーンアプリがモニタを覆う
            //   ・タスクバーが画面外(auto-hide引っ込み/退避を矩形で検出)
            //   ・auto-hide かつ 前面が最大化(矩形閾値を取りこぼす最大化窓を確実に拾う)
            let fs = foreground_is_fullscreen();
            let off = taskbar_offscreen();
            let autohide = taskbar_autohide();
            let maxd = autohide && foreground_is_maximized();
            if fs || off || maxd {
                if !HIDDEN.swap(true, Ordering::Relaxed) {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    diag(&format!(
                        "hide: fullscreen={fs} taskbar_offscreen={off} autohide={autohide} maximized={maxd}"
                    ));
                }
            } else {
                if HIDDEN.swap(false, Ordering::Relaxed) {
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                    diag(&format!(
                        "show: fullscreen={fs} taskbar_offscreen={off} autohide={autohide}"
                    ));
                }
                reposition(hwnd);
            }
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

/// プリマルチプライド BGRA を1画素書き込む（背景は完全透過なのでブレンドせず上書き）。
/// buf はトップダウン 32bpp。色は素の (r,g,b)＋アルファ a を与えると内部でプリマルチプライドする。
#[inline]
unsafe fn put_px(buf: *mut u32, w: i32, h: i32, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
    if x < 0 || y < 0 || x >= w || y >= h {
        return;
    }
    let pr = (r as u32 * a as u32) / 255;
    let pg = (g as u32 * a as u32) / 255;
    let pb = (b as u32 * a as u32) / 255;
    // メモリ上は B,G,R,A の順 → little-endian u32 では 0xAARRGGBB。
    let val = ((a as u32) << 24) | (pr << 16) | (pg << 8) | pb;
    *buf.add((y * w + x) as usize) = val;
}

/// 左ボタンを buf に直接描く（録音中=白い四角/停止中=赤い丸）。丸は3x3スーパサンプルでAA。
unsafe fn draw_left_button(buf: *mut u32, w: i32, h: i32) {
    let cy = h / 2;
    let bx = (WIDTH / 2) / 2;
    if RECORDING.load(Ordering::Relaxed) {
        // 白い四角（停止）。
        for y in (cy - 6)..(cy + 6) {
            for x in (bx - 6)..(bx + 6) {
                put_px(buf, w, h, x, y, 0xFF, 0xFF, 0xFF, 0xFF);
            }
        }
    } else {
        // 赤い丸（録音開始）。R=0xE0,G=0x30,B=0x20。
        let r = 7.0f32;
        for y in (cy - 8)..=(cy + 8) {
            for x in (bx - 8)..=(bx + 8) {
                let mut cov = 0u32;
                for sy in 0..3 {
                    for sx in 0..3 {
                        let fx = x as f32 + (sx as f32 + 0.5) / 3.0 - 0.5 - bx as f32;
                        let fy = y as f32 + (sy as f32 + 0.5) / 3.0 - 0.5 - cy as f32;
                        if fx * fx + fy * fy <= r * r {
                            cov += 1;
                        }
                    }
                }
                if cov > 0 {
                    let a = (cov * 255 / 9) as u8;
                    put_px(buf, w, h, x, y, 0xE0, 0x30, 0x20, a);
                }
            }
        }
    }
}

/// 右ボタンのアプリアイコンを buf に描く。DrawIconEx でメモリDCへ描画後、アルファを整える。
/// DrawIconEx がアルファチャネルを書く実装/書かない実装の双方に対応する適応処理:
/// - 既にアルファが入っていれば(=最大alpha>0)そのまま使う（滑らかな縁）。
/// - 全てalpha=0なら、RGBが非ゼロの画素を不透明(255)にする（クロマ無しなので赤縁は出ない）。
unsafe fn draw_icon(mem_dc: windows::Win32::Graphics::Gdi::HDC, buf: *mut u32, w: i32, h: i32) -> bool {
    let icon = WIDGET_ICON.load(Ordering::Relaxed);
    if icon == 0 {
        return false;
    }
    let sz = (h - 6).clamp(16, 24);
    let ox = (WIDTH / 2) + (WIDTH / 2) / 2;
    let ix = ox - sz / 2;
    let iy = (h - sz) / 2;
    let _ = DrawIconEx(mem_dc, ix, iy, HICON(icon as *mut _), sz, sz, 0, None, DI_NORMAL);

    // 描画領域のアルファ最大値を調べる。
    let (x0, y0) = (ix.max(0), iy.max(0));
    let (x1, y1) = ((ix + sz).min(w), (iy + sz).min(h));
    let mut max_a = 0u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let v = *buf.add((y * w + x) as usize);
            max_a = max_a.max(v >> 24);
        }
    }
    if max_a == 0 {
        // DrawIconEx がアルファ未設定 → RGB非ゼロを不透明化（プリマルチプライド済み扱い）。
        for y in y0..y1 {
            for x in x0..x1 {
                let p = buf.add((y * w + x) as usize);
                let v = *p;
                if v & 0x00FF_FFFF != 0 {
                    *p = v | 0xFF00_0000;
                }
            }
        }
    }
    true
}

/// フォールバック: アイコン未取得時の「ウィンドウ風」グリフ（本体＋タイトルバー帯）を buf に描く。
unsafe fn draw_fallback_glyph(buf: *mut u32, w: i32, h: i32) {
    let cy = h / 2;
    let ox = (WIDTH / 2) + (WIDTH / 2) / 2;
    let (l, t, r, b) = (ox - 7, cy - 6, ox + 7, cy + 6);
    for y in t..b {
        for x in l..r {
            // 上3pxは濃いグレーのタイトルバー帯、以降は明るいグレー本体。
            if y < t + 3 {
                put_px(buf, w, h, x, y, 0x80, 0x80, 0x80, 0xFF);
            } else {
                put_px(buf, w, h, x, y, 0xE0, 0xE0, 0xE0, 0xFF);
            }
        }
    }
}

/// per-pixel alpha でウィジェットの中身を合成し UpdateLayeredWindow で反映する。
unsafe fn render(hwnd: HWND) {
    let mut rc = RECT::default();
    if GetWindowRect(hwnd, &mut rc).is_err() {
        return;
    }
    let w = (rc.right - rc.left).max(1);
    let h = (rc.bottom - rc.top).max(1);

    let mem = CreateCompatibleDC(None);
    if mem.0.is_null() {
        return;
    }
    let mut bmi = BITMAPINFO::default();
    bmi.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: w,
        biHeight: -h, // 負=トップダウン
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0 as u32,
        ..Default::default()
    };
    let mut bits: *mut c_void = std::ptr::null_mut();
    let dib = match CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0) {
        Ok(d) if !bits.is_null() => d,
        _ => {
            let _ = DeleteDC(mem);
            return;
        }
    };
    let old = SelectObject(mem, HGDIOBJ(dib.0));
    let buf = bits as *mut u32;
    // CreateDIBSection はゼロ初期化済み（全画素 透過）。

    draw_left_button(buf, w, h);
    if !draw_icon(mem, buf, w, h) {
        draw_fallback_glyph(buf, w, h);
    }

    let blend = BLENDFUNCTION {
        BlendOp: 0,        // AC_SRC_OVER
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: 1,    // AC_SRC_ALPHA（ソースは per-pixel alpha）
    };
    let size = SIZE { cx: w, cy: h };
    let src = POINT { x: 0, y: 0 };
    let _ = UpdateLayeredWindow(
        hwnd,
        None,            // hdcDst: 画面DCは既定
        None,            // pptDst: 現在位置を維持（移動は SetWindowPos が担当）
        Some(&size),
        Some(mem),
        Some(&src),
        COLORREF(0),
        Some(&blend),
        ULW_ALPHA,
    );

    SelectObject(mem, old);
    let _ = DeleteObject(HGDIOBJ(dib.0));
    let _ = DeleteDC(mem);
}
