# ダウンロード

最新版は GitHub Releases から入手できます。

<div style="margin: 1.5rem 0;">
  <a href="https://github.com/Takenori-Kusaka/QuickScribe/releases/latest" style="display:inline-block;padding:0.7rem 1.4rem;background:#4f46e5;color:#fff;border-radius:8px;font-weight:700;text-decoration:none;">最新リリースを開く（GitHub Releases）</a>
</div>

## プラットフォーム別

| OS | ファイル |
|---|---|
| Windows (x64) | `QuickScribe_<version>_x64-setup.exe` |
| Windows (ARM64) | `QuickScribe_<version>_arm64-setup.exe` |
| Linux (AppImage) | `QuickScribe_<version>_amd64.AppImage` |
| Linux (deb) | `QuickScribe_<version>_amd64.deb` |

インストール後はアプリ内の自動アップデートで最新版に更新されます。

## コード署名について

QuickScribe の Windows バイナリのコード署名には、**[SignPath Foundation](https://signpath.org/)** の無償コード署名プログラム（オープンソース向け）を利用しています。

> Code signing for QuickScribe is provided by the **[SignPath Foundation](https://signpath.org/)** (free code signing for open-source projects).

未署名の配布物で SmartScreen 警告が出る場合は、「詳細情報」→「実行」で起動できます（署名の整備状況により表示が変わります）。

## 完全性の検証

自動アップデートの配布物は Tauri updater 署名で完全性を担保しています。手動ダウンロード時は GitHub Releases の正規URLから取得してください。
