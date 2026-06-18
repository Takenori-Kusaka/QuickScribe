# Windows タスクバーに録音ボタンを「実際に出す」実装 — 一次情報ソース

取得日: 2026-06-18
目的: QuickScribe(Tauri v2 / Rust / Windows)でタスクバーに録音トグルを常時表示する確実な方式を、実在OSSの実コードで裏付ける。

---

## 方式1: タスクバー上オーバーレイ（自前子ウィンドウ）= TrafficMonitor 型 【本命・実証あり】

### zhongyang219/TrafficMonitor （C++ / MFC, 35k+ stars）

実ファイル(WebFetchで確認):

- `TrafficMonitor/TaskBarDlg.cpp` (基底クラス CTaskBarDlg)
  - 確認した実コード:
    ```cpp
    // タスクバー本体HWNDを取得
    hTaskbar = ::FindWindow(_T("Shell_TrayWnd"), NULL);
    // 自前ダイアログをタスクバーの子ウィンドウにする（これが「埋め込み」の核心）
    m_connot_insert_to_task_bar = !(::SetParent(this->m_hWnd, GetParentHwnd()));
    // 座標追従
    ::GetWindowRect(m_hTaskbar, m_rcTaskbar);
    // AdjustWindowPos() -> ResetTaskbarPos() / CalculateWindowSize() / AdjustTaskbarWndPos()
    ```
- `TrafficMonitor/TaskBarDlg.h` (確認)
  - 抽象基底の純粋仮想:
    ```cpp
    virtual void InitTaskbarWnd() = 0;
    virtual void AdjustTaskbarWndPos(bool force_adjust) = 0;
    virtual HWND GetParentHwnd() = 0;       // 埋め込み先(親)を返す
    HWND m_hTaskbar;
    CRect m_rcTaskbar;
    bool m_connot_insert_to_task_bar{ false };  // 埋め込み失敗フラグ
    bool AdjustWindowPos(bool force_adjust = false);
    ```
- `TrafficMonitor/Win11TaskbarDlg.cpp` (Windows 11 派生クラス, 確認)
  - `GetParentHwnd()` は **`m_hTaskbar` をそのまま返す**（= Shell_TrayWnd に直接 SetParent）。
  - `InitTaskbarWnd()` は位置計算のためにタスクバーの子を探す:
    ```cpp
    m_hNotify = ::FindWindowEx(m_hTaskbar, 0, L"TrayNotifyWnd", NULL);  // 通知領域
    m_hStart  = ::FindWindowEx(m_hTaskbar, nullptr, L"Start", NULL);     // スタートボタン
    ```
  - 位置反映は `MoveWindow(m_rect);`。`m_rcStart` と `m_rcNotify` の矩形から表示位置を決める。
- `TrafficMonitor/TaskbarHelper.cpp` (マルチモニタ, 確認)
  - セカンダリタスクバーは `EnumWindows` で **`Shell_SecondaryTrayWnd`** を列挙し、
    `CompareTaskbarByMonitorOrder` でモニタ座標順にソート、矩形内包判定でモニタ対応付け。

要点(実コードから):
- 埋め込み先(親) = **プライマリ: `Shell_TrayWnd`**, **セカンダリ: `Shell_SecondaryTrayWnd`**。`SetParent(self, taskbarHwnd)` 一発。
- `TrayNotifyWnd` / `Start` は **座標計算のためだけ**に探す（埋め込みには使わない）。
- 失敗要因(Wiki記載): セキュリティソフトのブロック、スタートメニュー展開中。数回リトライしてダメならエラー表示。

参照URL:
- https://github.com/zhongyang219/TrafficMonitor
- https://github.com/zhongyang219/TrafficMonitor/wiki/Taskbar-Window
- https://raw.githubusercontent.com/zhongyang219/TrafficMonitor/master/TrafficMonitor/TaskBarDlg.cpp
- https://raw.githubusercontent.com/zhongyang219/TrafficMonitor/master/TrafficMonitor/TaskBarDlg.h
- https://raw.githubusercontent.com/zhongyang219/TrafficMonitor/master/TrafficMonitor/Win11TaskbarDlg.cpp
- https://raw.githubusercontent.com/zhongyang219/TrafficMonitor/master/TrafficMonitor/TaskbarHelper.cpp

### ChrisAnd1998/TaskbarXI （C++） — 反例(本方式ではない)
- `Taskbar11.cpp` 確認。`FindWindow(L"Shell_TrayWnd",0)`, `FindWindowEx(...,L"RebarWindow32"/L"MSTaskSwWClass")`,
  `EnumWindows(EnumCallbackTaskbars,...)` は使うが、**SetWindowRgn でタスクバーを整形するだけ**で
  子ウィンドウを埋め込まない。座標/クラス名探索の参考にはなるが配置方式は別物。
- https://github.com/ChrisAnd1998/TaskbarXI/blob/main/Taskbar11.cpp

---

## 方式2: サムネイルツールバー ITaskbarList3 — 「動いている」基準実装

### electron/electron （C++） — setThumbarButtons の実処理
実ファイル: `shell/browser/ui/win/taskbar_host.cc` (WebFetchで確認)

確認した実コード要点:
- `InitializeTaskbar()`:
  ```cpp
  CoCreateInstance(CLSID_TaskbarList, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&taskbar_));
  taskbar_->HrInit();
  ```
- ボタン追加は **トップレベルの window HWND** に対して:
  ```cpp
  taskbar_->ThumbBarAddButtons(window, thumb_buttons.size(), thumb_buttons.data());
  ```
- アイコンは **空でなく実ビットマップから HICON を生成**:
  ```cpp
  icons[i] = IconUtil::CreateHICONFromSkBitmap(button.icon.AsBitmap());
  thumb_button.dwMask |= THB_ICON;
  thumb_button.hIcon = icons[i].get();   // THB_ICON + hIcon。THB_BITMAP/HIMAGELISTは未使用
  ```
- クリックは `callback_map_[iId]` に保存→`HandleThumbarButtonEvent(button_id)` で実行。
- 注: taskbar_host.cc 自体は TaskbarButtonCreated を listen せず、Electron は別所
  (DesktopWindowTreeHostWin)でメッセージ受信。PR #2400 で「TaskbarButtonCreated 受信が必須」と明記。

参照URL:
- https://github.com/electron/electron/blob/main/shell/browser/ui/win/taskbar_host.cc
- https://github.com/electron/electron/pull/2400
- https://www.electronjs.org/docs/latest/tutorial/windows-taskbar

### Tauri v2 でのサムネイルツールバー動作実績 → 確認できず
- tauri-apps/tauri #10141 (feat: Windows Thumbnail Toolbar) を確認。
  **動作する Rust/windows-rs 実装コードは投稿されていない**（2024-06 時点の機能要望のまま）。
  https://github.com/tauri-apps/tauri/issues/10141
- skipTaskbar の Windows バグ: tauri-apps/tauri #10422。
  WS_EX_TOOLWINDOW/WS_EX_APPWINDOW を手で弄る回避も「うまくいかない」報告。
  https://github.com/tauri-apps/tauri/issues/10422
- 結論: **Rust/Tauri 界隈にサムネイルツールバーの動作実績ある実装は確認できなかった。**

---

## 方式3: ジャンプリスト（右クリック→タスク）

### MS公式サンプル CustomJumpList（C++） — 完全な実シーケンス
実ファイル: `winui/shell/appshellintegration/CustomJumpList/CustomJumpListSample.cpp` (WebFetchで確認)

確認したシーケンス:
1. `CoCreateInstance(CLSID_DestinationList ...)` → `ICustomDestinationList`
2. `BeginList(&cMinSlots, IID_PPV_ARGS(&poaRemoved))`
3. `CoCreateInstance(CLSID_EnumerableObjectCollection ...)` → `IObjectCollection`
4. `_CreateShellLink`:
   ```cpp
   IShellLink *psl;
   CoCreateInstance(CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&psl));
   // ... SetPath/SetArguments/SetIconLocation ...
   IPropertyStore *pps;  psl->QueryInterface(IID_PPV_ARGS(&pps));
   PROPVARIANT propvar;
   InitPropVariantFromString(pszTitle, &propvar);   // ← タイトル
   pps->SetValue(PKEY_Title, propvar);
   pps->Commit();
   ```
5. `AddUserTasks(poa)` → 6. `CommitList()`
- 注: サンプル自体は SetCurrentProcessExplicitAppUserModelID を含まないが、本番では先に呼ぶ必要。
- https://github.com/pauldotknopf/WindowsSDK7-Samples/blob/master/winui/shell/appshellintegration/CustomJumpList/CustomJumpListSample.cpp

### windows-rs での PKEY_Title / PROPVARIANT 構築 — 重要な制約
- **`InitPropVariantFromString` は windows-rs に未バインド**（propvarutil.h のインライン関数のため）。
  microsoft/windows-rs #976 で確認。代替に `InitPropVariantFromStringVector` 等は存在するが用途違い。
  https://github.com/microsoft/windows-rs/issues/976
- 回避: `PROPVARIANT` を手で構築（`vt = VT_LPWSTR`, `Anonymous.Anonymous.Anonymous.pwszVal = PWSTR(...)`）。
  `PROPVARIANT` は `windows::Win32::System::Com::StructuredStorage` にある。
  https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Com/StructuredStorage/struct.PROPVARIANT.html
- `IShellLinkW`, `ICustomDestinationList`, `IObjectCollection` は windows-rs の
  `windows::Win32::UI::Shell` に存在(確認: docs-rs IShellLinkW ページ)。

---

## QuickScribe 現状コード(src-tauri/src/taskbar.rs)の所見
- 方式2(サムネイル)を採用。`window.hwnd()` のトップレベルHWNDに SetWindowSubclass、
  `TaskbarButtonCreated` 受信後 `ThumbBarAddButtons`、`THBN_CLICKED` で emit。MS手順自体は正しい。
- アイコンは `LoadIconW(IDI_APPLICATION)` = 有効なHICON（空HICONではない）。これは失敗原因ではない。
- 差分(動くElectronとの比較で残る疑い):
  - サムネイルボタンは「ウィンドウのサムネイル(ホバープレビュー)内」にしか出ない。
    **常時表示ではない**。ユーザ要件「常時表示の物理的ボタン」と本質的に不一致。
  - Tauri/WebView2 構成で hwnd() が返すのが本当にタスクバーボタンを持つトップレベルか、
    WS_EX_TOOLWINDOW 付与でサムネイル対象外になっていないか要検証(#10422 関連)。
