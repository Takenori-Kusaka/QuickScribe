# セキュリティポリシー / Security Policy

QuickScribe は思考整理・自己理解のための**ローカル完結**ボイスジャーナルです。プライバシーを中核に据えており、脆弱性報告を歓迎します。

## サポート対象バージョン / Supported Versions

**最新リリースのみ**をベストエフォートでサポートします。古いバージョンへの個別修正は行いません（自動アップデートで最新版へ更新してください）。

| バージョン | サポート |
|---|---|
| 最新リリース | ✅ |
| それ以前 | ❌ |

## 脆弱性の報告 / Reporting a Vulnerability

**公開 Issue では報告しないでください。** 以下の非公開チャネルを使ってください:

- GitHub の **Private Vulnerability Reporting**（推奨）: リポジトリの **Security** タブ →「**Report a vulnerability**」から非公開で報告できます。
  - 直リンク: `https://github.com/Takenori-Kusaka/QuickScribe/security/advisories/new`

報告には可能な範囲で以下を含めてください: 影響範囲・再現手順・対象バージョン/OS・想定される深刻度。

### 対応の流れ（ベストエフォート）
- 受領の確認: できる限り速やかに（個人運営のため遅延しうる点ご了承ください）。
- 調査・修正: 妥当な脆弱性は最新版で修正し、必要に応じて GitHub Security Advisory を公開します。
- クレジット: ご希望に応じて報告者を謝辞に記載します。

## 想定スコープ / Scope

- アプリ本体（録音・文字起こし・整形・ジャーナル）、自動アップデート機構、ローカルデータの取り扱い。
- **対象外の例**: ユーザーが明示的に選択した外部サービス（クラウドSTT・クラウドLLM）側の不具合、ユーザー環境/OSの問題、ソーシャルエンジニアリング。

## プライバシー設計（前提）/ Privacy by design

- **既定はローカル完結**: 録音・文字起こし（whisper.cpp）はオフラインで端末内処理。音声は既定で外部送信されません。
- **クラウド連携は明示的オプトイン**: クラウドSTT（Groq/OpenAI/Deepgram/Azure）や整形LLMは、ユーザーが設定で選択し鍵を入力した場合のみ有効になり、その場合は対象データが各プロバイダへ送信されます。各社のデータ方針は設定画面に明記しています。
- **秘密情報の保管**: API鍵・資格情報は OS のセキュアストレージ（Windows Credential Manager / macOS Keychain / Linux Secret Service）に保存し、平文設定には残しません。
- **配布物の完全性**: リリースは Tauri updater 署名付きで配布します（コード署名は導入予定）。

---

> This project supports only the latest release on a best-effort basis. Please report vulnerabilities privately via GitHub's "Report a vulnerability" (Security tab), not public issues.
