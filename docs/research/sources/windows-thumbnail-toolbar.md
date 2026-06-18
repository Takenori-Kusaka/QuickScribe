# Windows Thumbnail Toolbar in Tauri v2 — 一次情報メモ

取得日: 2026-06-18
対象: QuickScribe (Tauri v2 + Rust, `tauri = "2"`)
目的: タスクバーのサムネイルツールバー（thumbnail toolbar）に録音開始/停止ボタンを出し、クリックで録音トグルする実装の一次情報収集。

---

## 1. Tauri v2 `WebviewWindow::hwnd()` / `Window::hwnd()` の戻り値型

出典(原典・Tauri ソース dev ブランチ): `crates/tauri/src/window/mod.rs`
https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/src/window/mod.rs

```rust
use windows::Win32::Foundation::HWND;   // ファイル冒頭の import

#[cfg(windows)]
pub fn hwnd(&self) -> crate::Result<HWND> {
  self
    .window
    .dispatcher
    .window_handle()
    .map_err(Into::into)
    .and_then(|handle| {
      if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
        Ok(HWND(h.hwnd.get() as _))
      } else {
        Err(crate::Error::InvalidWindowHandle)
      }
    })
}
```

- 戻り値型: `tauri::Result<windows::Win32::Foundation::HWND>`（= `windows` クレートの `HWND`）。raw-window-handle 由来ではなく、内部で raw-window-handle から `windows::...::HWND` に再ラップして返している。
- `#[cfg(windows)]` ゲート付き。`WebviewWindow` は `Deref<Target=Window>` なので `webview_window.hwnd()` も同じメソッドを呼べる（`WebviewWindow` 自体の docs.rs ページには出ないが Deref 経由で利用可）。
- docs.rs (https://docs.rs/tauri/latest/tauri/window/struct.Window.html, version 2.11.3) では `hwnd()` が表示されない。これは docs.rs が既定で Linux ターゲットでビルドするため `#[cfg(windows)]` メソッドがドキュメント化されないことが理由。メソッドは実在する。

## 2. `windows` クレートのバージョン整合

出典: Tauri `crates/tauri/Cargo.toml` (dev, package version 2.11.3)
https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri/Cargo.toml

```toml
[target."cfg(windows)".dependencies]
windows = { version = "0.61", features = [
  "Win32_Foundation",
  "Win32_UI",
  "Win32_UI_WindowsAndMessaging",
] }
```

- Tauri 2.11.3 系は `windows = "0.61"` に依存。
- HWND の表現（出典: microsoft windows-docs-rs, windows 0.62.2 表示。0.61 も同一形）:
  https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Foundation/struct.HWND.html
  ```rust
  #[repr(transparent)]
  pub struct HWND(pub *mut c_void);
  ```
- 0.61↔0.62 で `windows-core` に破壊的変更があった（型システム改善）。出典: microsoft/windows-rs Releases / Aug 2025 issue #3746。
  https://github.com/microsoft/windows-rs/releases
- 重要: `HWND` は `#[repr(transparent)]` の `*mut c_void` ラッパなので、異なる `windows` バージョン間でも内部ポインタは同一表現。型不一致は「別クレートインスタンスの別型」として起こるが、`HWND_other(tauri_hwnd.0)` で再ラップすれば値は保持される。

## 3. `ITaskbarList3` の Rust シグネチャ（windows クレート）

出典: microsoft windows-docs-rs (windows 0.62.2 表示。0.61 も同一API)
https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Shell/struct.ITaskbarList3.html

```rust
pub unsafe fn HrInit(&self) -> Result<()>;
pub unsafe fn ThumbBarAddButtons(&self, hwnd: HWND, pbutton: &[THUMBBUTTON]) -> Result<()>;
pub unsafe fn ThumbBarUpdateButtons(&self, hwnd: HWND, pbutton: &[THUMBBUTTON]) -> Result<()>;
pub unsafe fn ThumbBarSetImageList(&self, hwnd: HWND, himl: HIMAGELIST) -> Result<()>;
```

THUMBBUTTON 構造体 (出典: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Shell/struct.THUMBBUTTON.html):
```rust
#[repr(C)]
pub struct THUMBBUTTON {
    pub dwMask: THUMBBUTTONMASK,
    pub iId: u32,
    pub iBitmap: u32,
    pub hIcon: HICON,
    pub szTip: [u16; 260],
    pub dwFlags: THUMBBUTTONFLAGS,
}
```
- `THUMBBUTTONMASK(pub i32)`, `THUMBBUTTONFLAGS(pub i32)` (repr(transparent))。
- 定数は `windows::Win32::UI::Shell` のトップレベル定数: THB_BITMAP, THB_ICON, THB_TOOLTIP, THB_FLAGS / THBF_ENABLED, THBF_DISABLED, THBF_HIDDEN, THBF_DISMISSONCLICK 等。
- `THBN_CLICKED: u32 = 6144` (出典: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Shell/constant.THBN_CLICKED.html)

ThumbBarAddButtons の制約 (出典: MS Learn, ms.date 2025-08-22):
https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-itaskbarlist3-thumbbaraddbuttons
- 最大7ボタン。後から追加/削除/並べ替え不可（最初に全集合を渡す）。以後は ThumbBarUpdateButtons で表示/非表示・状態変更のみ。
- クリック時: 当該ウィンドウに `WM_COMMAND` が送られ、`HIWORD(wParam)==THBN_CLICKED`, `LOWORD(wParam)==ボタンID`。
- C++ 例（ThumbBarSetImageList + ThumbBarAddButtons, CoCreateInstance(CLSID_TaskbarList, ...IID_PPV_ARGS)）が掲載されている。

## 4. ボタン追加のタイミング (TaskbarButtonCreated)

出典: MS Learn taskbar-extensions / Thumbnail Toolbar Sample
https://learn.microsoft.com/en-us/windows/win32/shell/samples-taskbarthumbnailtoolbar
- サムネイルボタンは「タスクバーボタンが生成された後」にのみ追加可能。
- `RegisterWindowMessageW("TaskbarButtonCreated")` で得たメッセージIDを wndproc で受信してから `ThumbBarAddButtons` を呼ぶのが定石。ウィンドウ表示前/タスクバーボタン生成前に呼ぶと E_INVALIDARG（hwnd がタスクバーボタンに紐づかない）。

tao の前例（ITaskbarList を CoCreateInstance して progress 設定。サムネイルではないが同一COMパターンの実証）:
出典: https://github.com/tauri-apps/tao/blob/dev/src/platform_impl/windows/window.rs (set_progress_bar 付近)
```rust
let taskbar_list: ITaskbarList = CoCreateInstance(&TaskbarList, None, CLSCTX_SERVER).unwrap();
```

## 5. クリック受信 / サブクラス化（最大リスク）

Tauri v2 は生の wndproc メッセージを公開していない（未実装の feature request）。
出典: [feat] Expose raw window procedure messages on Windows — tauri-apps/tauri issue #11650 (open)
https://github.com/tauri-apps/tauri/issues/11650
- 提案中の `RunEvent::WindowEvent::Raw(u32, usize, isize)` は未実装。`on_window_event` では WM_COMMAND は拾えない。
- よって `WM_COMMAND`/`THBN_CLICKED`/`TaskbarButtonCreated` を拾うには `SetWindowSubclass` でサブクラス化が必須。

SetWindowSubclass (出典: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Shell/fn.SetWindowSubclass.html):
```rust
pub unsafe fn SetWindowSubclass(
    hwnd: HWND,
    pfnsubclass: SUBCLASSPROC,
    uidsubclass: usize,
    dwrefdata: usize,
) -> BOOL;
```
- `DefSubclassProc`, `RemoveWindowSubclass` も同 `windows::Win32::UI::Shell` モジュール（feature `Win32_UI_Shell` 必要）。
- SUBCLASSPROC: `unsafe extern "system" fn(hwnd, msg: u32, wparam: WPARAM, lparam: LPARAM, uidsubclass: usize, dwrefdata: usize) -> LRESULT`

## 6. アイコン (HICON)

- `THB_ICON` を `dwMask` に立て `hIcon` に `HICON` を渡す。アイコン無し（`THB_TOOLTIP` のみ）でも可能だが、ボタン面が空になるためアイコン推奨。
- `LoadImageW`/`LoadIconW`/`CreateIconFromResource` 等で取得（windows::Win32::UI::WindowsAndMessaging）。

## 7. 既存実装・要望

- Tauri 公式に thumbnail toolbar サポート要望: [feat] Windows Thumbnail Toolbar (like media control buttons) — tauri-apps/tauri issue #10141 (open, 2024-06-27)。実装・回避策コードは無し。
  https://github.com/tauri-apps/tauri/issues/10141
- 生 wndproc 公開要望 #11650（上記）。
- tao の ITaskbarList 利用（progress）が COM パターンの実証例。
- `windows`/`winsafe` の ITaskbarList3 ドキュメントあり。完全な「Tauri v2 で thumbnail toolbar」公開実例は未発見（要・追検証）。

---

## 未確認/不確実
- windows 0.61 の ITaskbarList3/THUMBBUTTON が 0.62 と完全同一シグネチャか（0.62.2 ドキュメントで確認済み。0.61 個別ページは未取得 → cargo doc で要確認）。
- THB_*/THBF_* 各定数の正確な数値（Win32ヘッダ準拠と推定: THB_BITMAP=1,THB_ICON=2,THB_TOOLTIP=4,THB_FLAGS=8; THBF_ENABLED=0,THBF_DISABLED=1,THBF_DISMISSONCLICK=2,THBF_HIDDEN=8 → cargo doc で要確定）。
- tao が TaskbarButtonCreated を内部処理しているか（未確認。していなければ自前サブクラスで登録が必要）。
