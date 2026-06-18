# Win11 タスクバー子ウィンドウ埋め込みが描画されない — 根本原因の一次情報

取得日: 2026-06-19
調査対象: `SetParent(myHwnd, FindWindow("Shell_TrayWnd"))` で WS_CHILD としてタスクバーに自前ウィンドウを埋め込む手法が Windows 11 実機で全く描画されない問題の根本原因。

---

## 1. Windows 11 タスクバーは XAML Islands で全面再実装された（最重要・一次に近い専門家情報）

- **Ramen Software（Windhawk / 7+ Taskbar Tweaker 作者、Win11 タスクバー内部の第一人者）**
  URL: https://ramensoftware.com/7-taskbar-tweaker-and-a-first-look-at-windows-11
  要点（原文引用）:
  - "the implementation of the visual part of the taskbar is completely new, written from scratch using a new technology called **XAML Islands**."
  - 大半のコンポーネントが UWP コントロールで再実装され、「非システム部分の通知領域とサムネイルプレビューのみ」が旧技術のまま。
  - 旧ツールの多くの機能は「実装の詳細が完全に置き換えられた」ため作り直しが必要。単純な実装でも "side effects and instability" を起こす。
  確度: 高（当該分野の最有力開発者の一次解説）。

- **タスクバー XAML リソース実物（22H2, OS Build 22621.1702）**
  URL: https://gist.github.com/m417z/ad0ab39351aca905f1d186b1f1c3d8c7
  要点: `MicrosoftWindows.Client.Core` の resources.pri から抽出された XAML 群。タスクバー UI が XAML で構成されていることの物的証拠。
  確度: 高（実バイナリ由来）。

## 2. SetParent はモダン XAML フレームの子で Win11 特異的に壊れる（実 issue）

- **microsoft/WindowsAppSDK #3788（= microsoft-ui-xaml #8707 の転記）**
  URL: https://github.com/microsoft/WindowsAppSDK/issues/3788
  原文（バグ報告本文）: "Using the SetParent method from user32.dll, to put a new window inside the main window, **on Windows 11** ... the child window goes inside the parent window, but **looses the resizable property and title bar, this behavior is not happening on Win 10**."
  追加コメント要点（microsoft-ui-xaml #8707 のコメント経由で確認）:
  - WinUI3 / .NET MAUI では再現するが **WinForms / WPF では起きない** → 旧来の純 HWND ではなく「モダン XAML フレーム」固有の問題。
  - スタイル（WS_CAPTION/WS_THICKFRAME）は SetParent 前後で変化なし。
  - Microsoft からの公式回答・修正なし（2023/08〜2024/07 で未解決のまま放置）。
  確度: 高（実 issue 本文）。コメントは WebFetch 要約経由（中）。

- **microsoft/microsoft-ui-xaml #8707**
  URL: https://github.com/microsoft/microsoft-ui-xaml/issues/8707
  確度: 高（実 issue）。

## 3. Tauri 等フレームワークは原因ではない（Tauri チーム員が明言）

- **tauri-apps/tauri #8299 — parent-window example が Win11 で動かない**
  URL: https://github.com/tauri-apps/tauri/issues/8299
  コメント要点（Tauri チーム員 amrbashir、API コメント取得）:
  - 原文: "on Windows, the **webview itself is a child window** and so when you create a child window on Windows it is actually **rendered behind the webview of the main window**."
  - これは **Windows プラットフォームのウィンドウ階層モデルに起因** する一般的挙動であり、Tauri 固有ではない（macOS では期待通り動く）。
  - 回避策: `parent_window` ではなく `owner_window()`（所有関係）を使う。ただし所有ウィンドウは親に追従しない。
  確度: 高（API 経由で取得した実コメント本文）。
  → 含意: フレームワークを Electron 等に替えても、Win32 のウィンドウ階層/Z オーダ問題そのものは解決しない。

## 4. TrafficMonitor は Win11 でも「Shell_TrayWnd 直下の子」で動いている（実コード）

- **実ソース TaskBarDlg.cpp**
  URL: https://raw.githubusercontent.com/zhongyang219/TrafficMonitor/master/TrafficMonitor/TaskBarDlg.cpp
  実コード抜粋:
  - L693: `hTaskbar = ::FindWindow(_T("Shell_TrayWnd"), NULL);`（親探索の最終フォールバックは依然 `Shell_TrayWnd`。Win11 専用に別 XAML 子へ切替える分岐は**無い**）
  - L999: `m_connot_insert_to_task_bar = !(::SetParent(this->m_hWnd, GetParentHwnd())); //把程序窗口设置成任务栏的子窗口`
  - L590-601: `WS_EX_LAYERED` を設定し D2D で alpha 合成して自前描画。
  - L988: `ModifyStyleEx(0, WS_EX_TOOLWINDOW);`
  - L560: `SetWindowPos(&wndTopMost, ...)` で最前面化。
  要点: **トップレベルの legacy HWND である `Shell_TrayWnd` に直接 SetParent し、自前で D2D 描画**している。XAML のコンテンツツリー（島の中）には差し込んでいない。これが Win11 でも描画できている理由。
  確度: 高（実ソース）。

- **TrafficMonitor 公式 Wiki / Help（埋め込み機構の説明）**
  - https://github.com/zhongyang219/TrafficMonitor/wiki/Taskbar-Window
  - https://github.com/zhongyang219/TrafficMonitor/blob/master/Help_en-us.md
  要点: 「タスクバーの子ウィンドウとして埋め込む」。セキュリティソフトに阻害されると失敗。Win11 ではウィジェット（左寄せ）との重なり回避設定あり。
  確度: 中〜高（公式ドキュメント）。

- **Win11 22H2 での既知の表示問題（位置ズレ系であって「全く出ない」ではない）**
  - https://github.com/zhongyang219/TrafficMonitor/issues/1318 （中央に出て右に動かせない）
  - https://github.com/zhongyang219/TrafficMonitor/issues/1197 （22H2 整列）
  要点: 22H2 で「位置が崩れる」報告は多いが「全く描画されない」ではない。つまり Shell_TrayWnd 直下方式自体は Win11 で生存している。
  確度: 中（issue タイトル・本文）。

## 5. 推奨代替: タスクバー上に重ねる最前面レイヤードツールウィンドウ

- **Shell_TrayWnd は WS_EX_TOPMOST を持つ最前面ウィンドウ（explorer.exe）**
  - 参照: https://github.com/dechamps/RudeWindowFixer
  要点: タスクバー(Shell_TrayWnd)自体が WS_EX_TOPMOST。別の透明全画面 TOPMOST ウィンドウがあると Rude Window Manager がそちらを最前面と誤認し、タスクバーが最前面を失う（＝TOPMOST オーバレイは技術的に成立する。Z オーダ競合に注意が必要なだけ）。
  確度: 中（OSS リポジトリ解説）。
- **SetParent の MSDN 仕様**
  URL: https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-setparent
  要点: 子の可視領域は親クライアント領域にクリップされる。SetParent は WS_CHILD/WS_POPUP スタイルを変更しない（呼び出し前に WS_POPUP を外し WS_CHILD を付ける必要）。可視ウィンドウなら system が再描画する。
  確度: 高（MS 公式）。

---

## 結論（要約）
Win11 22H2+ のタスクバーは XAML Islands で再実装された。しかし **トップレベルの `Shell_TrayWnd` 自体は今も legacy HWND**（XAML 島の「ホスト」）であり、ここへ直接 SetParent する TrafficMonitor 方式は Win11 でも生存している。「全く描画されない」のは Win11 が子埋め込みを一律拒否したからではなく、(a) FindWindow が誤ハンドル/別 XAML 子を掴む、(b) WS_CHILD 化・座標(DPI)・WS_EX_LAYERED+自前描画・再描画呼び出しの不備、(c) webview/フレーム自身が子ウィンドウで Z オーダ的に背面化、のいずれか。Tauri は原因ではない（同じ Win32 階層問題は Electron でも起きる）。**Win11 で最も堅いのは、子埋め込みではなく WS_EX_TOPMOST|WS_EX_TOOLWINDOW のレイヤードツールウィンドウをタスクバー矩形上に座標計算で重ねる方式**（プロセス境界/XAML 非依存）。
