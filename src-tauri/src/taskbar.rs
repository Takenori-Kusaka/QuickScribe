// Windows タスクバーのサムネイルツールバー（ITaskbarList3）に「録音 開始/停止」ボタンを出す。
// Tauri v2 は生 wndproc を公開しないため、ウィンドウを SetWindowSubclass でサブクラス化し、
// "TaskbarButtonCreated" 受信後に ThumbBarAddButtons、WM_COMMAND/THBN_CLICKED で toggle-record を emit する。
// 一次情報: docs/research/sources/windows-thumbnail-toolbar.md
//
// 注: このモジュールは #[cfg(windows)] でのみコンパイルされる。実行時挙動はWindows実機で要確認。

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use tauri::Emitter;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateBitmap, CreateDIBSection, DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{
    DefSubclassProc, SetWindowSubclass, ITaskbarList3, TaskbarList, THBF_ENABLED, THBN_CLICKED,
    THB_FLAGS, THB_ICON, THB_TOOLTIP, THUMBBUTTON,
};
use windows::Win32::UI::WindowsAndMessaging::{
    ChangeWindowMessageFilterEx, CreateIconIndirect, LoadIconW, PostMessageW, RegisterWindowMessageW,
    HICON, ICONINFO, IDI_APPLICATION, MSGFLT_ALLOW, WM_APP, WM_COMMAND,
};

/// 録音トグルボタンのID（WM_COMMAND の LOWORD で判定）。
const TOGGLE_BTN_ID: u32 = 1;

/// 競合保険でボタン追加を再試行させる自分宛メッセージ。
const WM_ADD_THUMBBUTTONS: u32 = WM_APP + 1;

/// "TaskbarButtonCreated" の登録メッセージID（Explorer再起動時にも再送される）。
static TASKBAR_CREATED_MSG: AtomicU32 = AtomicU32::new(0);

/// 既にボタンを追加済みか（二重 Add を避け、2回目以降は Update にする）。
static BUTTONS_ADDED: AtomicBool = AtomicBool::new(false);

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

        // UIPI対策（MS公式サンプル準拠）。昇格起動時に TaskbarButtonCreated と
        // クリックの WM_COMMAND がメッセージフィルタで遮断されると「何も出ない」ため、
        // 明示的に許可する（非昇格でも無害）。これが出ない最有力原因への対処。
        let _ = ChangeWindowMessageFilterEx(hwnd, msg, MSGFLT_ALLOW, None);
        let _ = ChangeWindowMessageFilterEx(hwnd, WM_COMMAND, MSGFLT_ALLOW, None);

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
        // 初回は Add、2回目以降は Update（ThumbBarAddButtonsは一度きりのため）。
        let already = BUTTONS_ADDED.swap(true, Ordering::Relaxed);
        if let Err(e) = add_or_update_buttons(hwnd, already) {
            BUTTONS_ADDED.store(false, Ordering::Relaxed); // 次回再試行できるよう戻す。
            eprintln!("サムネイルボタン設定に失敗: {e}");
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

/// ITaskbarList3 でサムネイルボタンを追加（初回）または更新（2回目以降）する。
unsafe fn add_or_update_buttons(hwnd: HWND, update: bool) -> windows::core::Result<()> {
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
    if update {
        taskbar.ThumbBarUpdateButtons(hwnd, &buttons)
    } else {
        taskbar.ThumbBarAddButtons(hwnd, &buttons)
    }
}

/// ボタン用アイコン（取得失敗時はnull=アイコン無し表示）。
unsafe fn app_icon() -> HICON {
    LoadIconW(None, IDI_APPLICATION).unwrap_or_default()
}

/// 「録音中」を表す赤い丸（●）のアイコンを生成する。
/// RECインジケータの一般的慣習（カメラ/レコーダ/配信アプリ等が赤丸）に倣う。
/// 32bppのアルファ付きDIBSection（CreateBitmapのDDBはアルファが不安定なため）で作る。
unsafe fn recording_icon() -> HICON {
    const N: i32 = 16;
    let mut bmi = BITMAPINFO::default();
    bmi.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: N,
        biHeight: -N, // 負=トップダウン
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0 as u32,
        ..Default::default()
    };
    let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
    let hbm_color = match CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0) {
        Ok(h) if !bits.is_null() => h,
        _ => return HICON::default(),
    };

    // 円の内側＝不透明な赤、外側＝完全透明（プリマルチプライドではなく素のARGBでOK）。
    let px = bits as *mut u32;
    let center = (N as f32 - 1.0) / 2.0;
    let radius = center - 0.5;
    for y in 0..N {
        for x in 0..N {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let inside = dx * dx + dy * dy <= radius * radius;
            // BGRA(LE)で 0xAARRGGBB。内側=不透明な赤、外側=透明。
            let v: u32 = if inside { 0xFFE0_2020 } else { 0x0000_0000 };
            *px.add((y * N + x) as usize) = v;
        }
    }

    let hbm_mask = CreateBitmap(N, N, 1, 1, None);
    let info = ICONINFO {
        fIcon: true.into(),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };
    let icon = CreateIconIndirect(&info).unwrap_or_default();
    let _ = DeleteObject(HGDIOBJ(hbm_color.0));
    let _ = DeleteObject(HGDIOBJ(hbm_mask.0));
    icon
}

/// タスクバーボタンに録音中バッジ(オーバーレイアイコン)を表示/解除する。
/// hwnd_raw は WebviewWindow::hwnd() の生ポインタ(isize)。録音状態をタスクバー上で可視化する。
pub fn set_overlay(hwnd_raw: isize, recording: bool) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let taskbar: ITaskbarList3 =
            match CoCreateInstance(&TaskbarList, None, CLSCTX_INPROC_SERVER) {
                Ok(t) => t,
                Err(_) => return,
            };
        let _ = taskbar.HrInit();
        let hwnd = HWND(hwnd_raw as *mut _);
        if recording {
            // 録音中＝赤い丸（●）。RECインジケータの世界的慣習に倣う。
            let icon = recording_icon();
            let _ = taskbar.SetOverlayIcon(hwnd, icon, w!("録音中"));
        } else {
            let _ = taskbar.SetOverlayIcon(hwnd, HICON::default(), PCWSTR::null());
        }
    }
}
