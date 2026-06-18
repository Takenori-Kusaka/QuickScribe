# Windows タスクバーから録音操作する — 一次情報メモ

取得日: 2026-06-18
対象: QuickScribe (Tauri v2 + Rust, `tauri = "2"`, `windows = "0.61"`)
目的: 実機で「サムネイルツールバーのボタンが何も出ない」原因究明と、確実に動く実装パス／代替手段の一次情報収集。
前提メモ: `docs/research/sources/windows-thumbnail-toolbar.md`（基礎API・型）も併読。

---

## 0. 結論サマリ（先に）

- tao は自ウィンドウ proc を `RegisterClassExW.lpfnWndProc` で設定し **`SetWindowSubclass` は使っていない**。
  よって我々の `SetWindowSubclass` は tao クラス proc の上に正しく載り、`DefSubclassProc` で下へチェインする。
  → **サブクラスのチェイン順序は失敗原因ではない**（最有力仮説から除外）。
- Microsoft 公式サンプルは **`TaskbarButtonCreated` を WndProc 内で初回受信時に `RegisterWindowMessage` し、`ChangeWindowMessageFilter` で `TaskbarButtonCreated` と `WM_COMMAND` を許可**してから `ThumbBarSetImageList`+`ThumbBarAddButtons` を呼ぶ。
- Electron は **初回 `ThumbBarAddButtons` / 2回目以降 `ThumbBarUpdateButtons`**、HICON 直指定（`THB_ICON`）で動作。`TaskbarButtonCreated` 受信時に `RestoreThumbarButtons`。
- サムネイルツールバーは **タスクバーボタンのサムネイルプレビュー内下部にしか出ない**。マウスオーバーして初めて見える。「タスクバー上に常時ボタン」というユーザ期待とは構造的に乖離。

---

## 1. ThumbBarAddButtons 公式仕様（出典: MS Learn, ms.date 2025-08-22）
https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-itaskbarlist3-thumbbaraddbuttons

Remarks 要点（原文より）:
- ツールバーは標準 toolbar control。**最大7ボタン**。中央寄せ・透過で、**サムネイルの下の領域**に表示（サムネイルを覆わない）。
- 「**Thumbnail toolbars are displayed only when thumbnails are being displayed.**」
  = サムネイルが表示されている時だけ出る。グループ化で枚数が多くサムネイルが出ない場合は legacy メニューにフォールバックし、ツールバーは出ない。
- クリック時: 当該ウィンドウに `WM_COMMAND`。`HIWORD(wParam)==THBN_CLICKED`, `LOWORD(wParam)==ボタンID`。
- 追加後はボタンの増減・並べ替え不可。表示/非表示・状態変更は `ThumbBarUpdateButtons` のみ。
- E_INVALIDARG: hwnd がプロセスのものでない／**タスクバーボタンに紐づくウィンドウでない**、または cButtons が 1..7 外。
- **公式サンプルコードは HICON ではなく `ThumbBarSetImageList`+`THB_BITMAP`（イメージリスト）を使用**（hIcon は使っていない）。
  ただし Electron は HICON+`THB_ICON` で動作実績あり（後述）→ どちらも可。

## 2. Microsoft 公式サンプル ThumbnailToolbar.cpp（最重要・正しい初期化順序）
出典(原典): microsoft/Windows-classic-samples
.../Win7Samples/winui/shell/appshellintegration/TaskbarThumbnailToolbar/ThumbnailToolbar.cpp
https://github.com/microsoft/Windows-classic-samples/tree/main/Samples/Win7Samples/winui/shell/appshellintegration/TaskbarThumbnailToolbar

要点（WebFetch 抽出）:
```c
// WndProc 内、初回メッセージで動的メッセージIDをキャッシュ
static UINT s_uTBBC = WM_NULL;
if (s_uTBBC == WM_NULL) {
    s_uTBBC = RegisterWindowMessage(L"TaskbarButtonCreated");
    // ★ 昇格時/メッセージフィルタ対策。これが無いと TaskbarButtonCreated / WM_COMMAND が届かない
    ChangeWindowMessageFilter(s_uTBBC, MSGFLT_ADD);
    ChangeWindowMessageFilter(WM_COMMAND, MSGFLT_ADD);
}
...
if (message == s_uTBBC) {
    CreateThumbnailToolbar(hWnd);   // ← ここで初めて ThumbBar 系を呼ぶ
}
```
`CreateThumbnailToolbar` の流れ:
```c
// DPIに応じた bmp を imagelist にロード
hr = pTaskbarList->ThumbBarSetImageList(hWnd, himl);
// buttons[i].dwMask = THB_BITMAP | THB_TOOLTIP | THB_FLAGS; iBitmap=i;
hr = pTaskbarList->ThumbBarAddButtons(hWnd, ARRAYSIZE(buttons), buttons);
```
WM_COMMAND は `IDTB_BUTTON1` 等のIDで分岐（サンプルは MessageBox 表示）。

**ここから読み取れる差分（我々の実装との比較）:**
1. 公式は `ChangeWindowMessageFilter`（または Ex）を呼んでいる。我々の `taskbar.rs` は**呼んでいない**。
2. 公式は WndProc 内で `RegisterWindowMessage` をキャッシュ。我々は install() で事前登録（これ自体は問題ない）。

## 3. ChangeWindowMessageFilter(Ex) — UIPI とメッセージ遮断
出典: MS Learn
https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-changewindowmessagefilterex
https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-changewindowmessagefilter

- UIPI（User Interface Privilege Isolation, Vista以降）: 低い整合性レベル（explorer.exe = medium）から高い整合性レベル（管理者昇格アプリ = high）への
  ウィンドウメッセージは**既定で遮断される**。
- タスクバーは explorer.exe の一部。**アプリを管理者昇格で起動している場合、`TaskbarButtonCreated` も `THBN_CLICKED` の `WM_COMMAND` も届かない** → ボタンが出ない／クリックが効かない。
- 対策: `ChangeWindowMessageFilterEx(hwnd, msg, MSGFLT_ALLOW, NULL)` を該当メッセージに対して呼ぶ。
- 非昇格（通常起動）なら UIPI 遮断は起きないので、この対策が無くても本来は届く。
  → 「実機で出ない」が**昇格起動かどうか**で切り分け必須。

## 4. Electron の実装（確実に動く実例・HICON 直指定で動作）
出典(原典): electron/electron `shell/browser/ui/win/taskbar_host.cc`
https://github.com/electron/electron/blob/main/shell/browser/ui/win/taskbar_host.cc
および `native_window_views_win.cc`（メッセージ振り分け）

ITaskbarList3 生成:
```cpp
if (FAILED(::CoCreateInstance(CLSID_TaskbarList, nullptr, CLSCTX_INPROC_SERVER,
                              IID_PPV_ARGS(&taskbar_))) ||
    FAILED(taskbar_->HrInit())) { taskbar_.Reset(); return false; }
```
初回 vs 更新の使い分け（重要）:
- 初回: `taskbar_->ThumbBarAddButtons(window, size, data)`、`thumbar_buttons_added_ = true`
- 2回目以降: `thumbar_buttons_added_` が true なら `ThumbBarUpdateButtons()`
アイコン/ツールチップ:
- `thumb_button.dwMask |= THB_ICON; thumb_button.hIcon = icons[i].get();`（**HICON 直指定。THB_ICON 使用**）
- `THB_TOOLTIP` + `szTip` にUTF-8→wide 変換した文字列を `CopyStringToBuf`
- `GetThumbarButtonFlags()` が "disabled"/"dismissonclick"/"nobackground"/"hidden"/"noninteractive" を THBF へ
クリック振り分け（native_window_views_win.cc, PreHandleMSG）:
```cpp
if (message == taskbar_created_message_) {
    taskbar_host_.RestoreThumbarButtons(GetAcceleratedWidget());
    return true;
}
...
if (HIWORD(w_param) == THBN_CLICKED)
    return taskbar_host_.HandleThumbarButtonEvent(LOWORD(w_param));
```
→ **Electron は HICON で問題なく動く**。よって我々の `hIcon = LoadIconW(IDI_APPLICATION)` 自体は致命ではない
  （ただし `unwrap_or_default()` で NULL になった場合、ボタン面が空のアイコンになる可能性。`THB_ICON` を立てつつ hIcon が NULL だと描画不定 → アイコンは確実に有効な HICON を渡すべき）。
→ Electron は `TaskbarButtonCreated` 受信で **Restore（再追加）** している。Explorer 再起動・初回生成の両方をこれでカバー。

## 5. tao のウィンドウ proc 設定（サブクラス・チェイン問題の検証）
出典(原典): tauri-apps/tao `src/platform_impl/windows/event_loop.rs`
https://github.com/tauri-apps/tao/blob/dev/src/platform_impl/windows/event_loop.rs

- tao は `RegisterClassExW { lpfnWndProc: Some(thread_event_target_callback::<T>), .. }` でクラス proc を登録。
  通常ウィンドウは `public_window_callback`。**`SetWindowSubclass` は使用していない。**
- tao の proc は巨大な match だが **`WM_COMMAND` は処理していない**（消費されない）。
- 帰結:
  - 我々の `SetWindowSubclass` は tao の **クラス proc の手前**に挿入される。`SetWindowSubclass` 系は
    `lpfnWndProc` とは別レイヤ（comctl32 のサブクラスチェイン）で動き、未処理は `DefSubclassProc` 経由で
    最終的に元の `lpfnWndProc`（tao の proc）へ渡る。
  - したがって **`TaskbarButtonCreated` / `WM_COMMAND` は我々のサブクラス proc に届く**（tao が先に奪うことはない）。
  - → 「tao が先にサブクラス化していて自分の proc に届かない」という仮説は**否定**。

## 6. window.hwnd() はトップレベルか（子HWND誤掴み検証）
出典: `docs/research/sources/windows-thumbnail-toolbar.md` §1（Tauri ソース）＋
WebView2 子ウィンドウに関する Tauri issue 群（#10079, #12450, wry #650）。
- Tauri `window.hwnd()` は tao のトップレベルウィンドウ HWND を返す（raw-window-handle Win32 由来）。
- WebView2 のホスト HWND は**その子**であり、`window.hwnd()` では返らない。
- → 我々は**トップレベル HWND を正しく掴んでいる**。子HWND誤掴みではない（ThumbBar はトップレベルに対して正しい）。

## 7. 代替: Jump List（ICustomDestinationList）
出典: MS Learn / The Old New Thing / WindowsSDK7-Samples
https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-icustomdestinationlist
https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-icustomdestinationlist-addusertasks
https://devblogs.microsoft.com/oldnewthing/20131223-00/?p=2303
https://github.com/pauldotknopf/WindowsSDK7-Samples/blob/master/winui/shell/appshellintegration/CustomJumpList/CustomJumpListSample.cpp

- タスクバーボタン**右クリック**で出るカスタムタスク（Tasks カテゴリ）。`AddUserTasks(IObjectArray<IShellLink>)`。
- 各タスクは `IShellLink` = **コマンドライン引数付きで自プロセスを起動**するショートカット。`SetArguments` 必須。
- 「実行中アプリへトグル送信」は ICustomDestinationList 自体には無い。
  **アプリ側の単一インスタンス＋argv転送**で実現する（＝既存インスタンスを起こして処理させる）。
  → QuickScribe は既に `--toggle-record` を実装済み。Jump List タスク「録音 開始/停止」を
    `QuickScribe.exe --toggle-record` で登録すれば、単一インスタンス機構が既存ウィンドウへトグルを転送できる。
- BeginList → AddUserTasks → CommitList の順。AppUserModelID を `SetCurrentProcessExplicitAppUserModelID` で
  明示すると安定（出典: platformsdk.shell スレッド）。

## 8. 代替: Overlay Icon（状態表示のみ）
- `ITaskbarList3::SetOverlayIcon` で「録音中」バッジ表示は可能。**操作はできない**（状態表示専用）。
  録音中インジケータとして Jump List/Tray と併用すると体験が締まる。

## 9. 比較（発見性・実装確実性・Tauri容易性）
| 手段 | 操作可否 | 発見性 | 実装確実性 | Tauri容易性 |
|---|---|---|---|---|
| Thumbnail toolbar | 可（クリック） | 低（ホバー時のみ・気付かれにくい） | 中（TaskbarButtonCreated待ち+フィルタ+昇格依存） | 低（生Win32+subclass） |
| Jump List | 可（右クリック→タスク, argv経由） | 中 | **高**（静的タスク＋既存`--toggle-record`） | 中（COM呼びだが起動時1回） |
| Tray 左クリックトグル | 可 | 高（常時アイコン） | **最高**（tray-icon 標準API） | **最高**（Tauri公式 tray-icon） |
| Overlay icon | 不可（表示のみ） | - | 高 | 中 |

## 10. 未確認/留意
- 実機が**昇格起動**かは未確認（最有力切り分け）。`ChangeWindowMessageFilterEx` 追加で解決する可能性。
- `LoadIconW(None, IDI_APPLICATION)` が稀に NULL を返した場合の描画（`THB_ICON`+NULL hIcon）は不定。
  → 確実な HICON を渡すか、公式同様 `ThumbBarSetImageList`+`THB_BITMAP` に切替が堅い。
- サムネイル無効化（タスクバー設定/グループ化）時はそもそもツールバー非表示（仕様）。
