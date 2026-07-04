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

## 動作環境・対応形式

| 項目 | 内容 |
|---|---|
| 対応OS | Windows 10/11（x64 / ARM64）、Linux（AppImage / deb・x64） |
| 対応音声形式 | `mp3` / `wav` / `m4a` / `flac` / `ogg` / `opus` / `aac` |
| 入力ファイルサイズ上限 | 500 MB |
| ローカル文字起こしモデル | 初回文字起こし時に whisper モデルを自動ダウンロード（既定 `base` ≈ 142MB。日本語特化 `kotoba-whisper` 量子化 ≈ 538MB なども選択可）。以降はローカルに保存され再利用します。 |

> ローカル文字起こしと録音は端末内で完結します。クラウド STT / AI 整形を明示的に選んだ場合のみ、対象データが各プロバイダへ送信されます（[プライバシーポリシー](/privacy)）。

## コード署名について（現在は未署名）

QuickScribe の Windows バイナリは **現在は未署名** です。そのため初回起動時に Microsoft Defender SmartScreen の「発行元不明」警告が出る場合があります。その際は **「詳細情報」→「実行」** で起動できます。

将来的にはオープンソース向けの無償コード署名（[SignPath Foundation](https://signpath.org/) の OSS 署名プログラムなど）の利用を予定しています（申請・審査中）。署名が整備され次第、この警告は表示されなくなります。

## 完全性の検証

自動アップデートの配布物は Tauri updater 署名で完全性を担保しています。手動ダウンロード時は GitHub Releases の正規URLから取得してください。
