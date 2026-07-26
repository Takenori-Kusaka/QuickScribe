# トラブルシュートガイド

うまく動かないときの切り分け手順。ここに載っていない事象は [Issues](https://github.com/Takenori-Kusaka/QuickScribe/issues) へ。

## タスクバーのウィジェットが表示されない（Windows）

タスクバー上の録音ボタンは、タスクバー（`Shell_TrayWnd`）の矩形に重ねた独立ウィンドウとして描画している。表示されない場合は診断ログを有効にして、どの段階で失敗しているかを確認する。

診断ログは**既定で無効**。常駐アプリがディスクを消費し続けないための既定値なので、切り分けのときだけ有効にする（#667）。

### 有効にする

環境変数 `QS_TASKBAR_DIAG` に `1` を設定してから QuickScribe を起動する。

```powershell
# 現在のセッションだけで有効化して起動する
$env:QS_TASKBAR_DIAG = "1"
& "$env:LOCALAPPDATA\Programs\QuickScribe\quickscribe.exe"
```

インストール先が異なる場合はパスを読み替える。設定は**プロセス起動時に一度だけ**読まれるため、起動中のアプリに後から効かせることはできない。再起動が必要。

### ログの場所

```
%LOCALAPPDATA%\QuickScribe\logs\taskbar-diag.log
```

1 MB を超えると `taskbar-diag.log.1` へ退避し、新しいログを書き始める。**世代は2つまで**で、それ以上は増えない（合計約 2 MB が上限）。

### 中身について

記録されるのはウィンドウハンドルや Win32 API の失敗理由といった**内部の状態のみ**で、録音音声・文字起こし結果・保管庫の本文は含まれない。issue へ添付しても問題ないが、念のため中身を確認してから貼ること。

### 無効に戻す

環境変数を消して再起動すれば書き込みは止まる。既存のログファイルは自動では消えないので、不要なら手動で削除する。

```powershell
Remove-Item Env:\QS_TASKBAR_DIAG
Remove-Item "$env:LOCALAPPDATA\QuickScribe\logs\taskbar-diag.log*"
```
