# ADR-0008: ライセンス・収益・配布方針

- Status: Accepted
- Date: 2026-06-16
- Deciders: Takenori Kusaka
- Relates to: [ADR-0004](0004-product-positioning-voice-journal.md), [ADR-0005](0005-tech-stack.md), [3.6 release-ops](../planning/3.6-release-ops.md)

## Context

QuickScribe は無料の個人向けOSS。コード署名の有料サービス（SSL.com eSigner 約$20/月）の原資が無い。
配布形態（インストーラ vs ポータブル実行）と収益方針を確定する必要がある。

## Decision

### ライセンス・収益
- **MIT License** を採用。
- **収益モデルは持たない（完全無料）**。
- **GitHub Sponsors** を任意で用意（¥100〜の開発支援）。優先度は低い。`.github/FUNDING.yml` を配置（有効化はアカウント側設定が必要）。

### 配布形態
- **Windows**: インストーラ(NSIS/MSI)をやめ、**ポータブル実行ファイル(.exe)** を配布する。UAC/インストールの摩擦を避ける。WebView2 Evergreen ランタイム（Win10/11に概ね同梱）に依存。
- **Linux**: **AppImage**（単一ファイルのポータブル実行）を主とする。`.deb` は任意。
- **PC起動時の自動実行は設定でON/OFF**（Win=レジストリRunキー、Linux=autostart/.desktop）。インストーラが担っていた自動起動を設定機能で代替する。

### コード署名
- **当面は未署名**で配布する（原資ゼロのため）。未署名のWindowsポータブル実行は、初回ダウンロード実行時にSmartScreen警告が出る（Mark-of-the-Web）。READMEで「詳細情報→実行」を案内する。
- **無償のOSS向けコード署名（SignPath Foundation 等、適格OSSへ証明書を無償提供する枠組み）を申請**し、承認されれば原資ゼロで署名付き配布へ移行する。これを正規の解とし、[ADR-0005](0005-tech-stack.md)/[3.6](../planning/3.6-release-ops.md)の「SSL.com eSigner（有料）」を**置き換える**。
- ビルドprovenanceは `actions/attest-build-provenance`（Sigstore・無料）で付与する。

## Consequences

- リリース成果物が「インストーラ」から「ポータブル実行＋AppImage」に変わる。[3.6](../planning/3.6-release-ops.md)の配布章とStory **S8.3/S8.4** を更新する。
- 自動起動はアプリ内設定の責務になる（Story E5/E6に反映）。
- 未署名期間のSmartScreen警告は許容。無償署名の取得を S8.3 のゴールに変更。
- updaterのminisign署名（[3.6](../planning/3.6-release-ops.md)）はコード署名とは別物として継続。
