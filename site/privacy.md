# プライバシーポリシー / Privacy Policy

最終更新: 2026-06-27

QuickScribe（以下「本アプリ」）は、**プライバシーを中核に設計されたローカル完結のボイスジャーナル**です。本ポリシーは、本アプリがデータをどう扱うかを説明します。

## 基本方針：ローカル完結

- **録音・文字起こしは既定で端末内に閉じます。** 音声の録音、ローカル文字起こし（whisper.cpp / kotoba-whisper）、整形前の処理はすべてあなたの端末上で行われ、**音声データが既定で外部へ送信されることはありません。**
- 文字起こし結果・整形結果（ジャーナル）は、あなたの端末の**出力先フォルダ**（既定: ドキュメント/QuickScribe）に**プレーンな Markdown / テキストファイル**として保存されます。本アプリの開発者を含む第三者がこれらにアクセスすることはありません。
- 本アプリは**解析・トラッキング・テレメトリを行いません**。利用状況を収集・送信しません。

## クラウド連携（明示的なオプトインのみ）

ユーザーが設定で**明示的に選択し、APIキー等を入力した場合に限り**、以下の外部サービスへ対象データが送信されます。

- **クラウド文字起こし（任意）**: Groq / OpenAI / Deepgram / Microsoft Azure。選択時のみ、**録音音声**が当該プロバイダへ送信され文字起こしされます。
- **AIによる整形・用語チェック・横断発見（任意）**: Google Gemini / Anthropic / OpenAI / ローカル Ollama / AWS Bedrock / Claude Platform on AWS。選択時のみ、**文字起こしテキスト等**が当該プロバイダへ送信されます。

これらを使うかはユーザーの選択です。**プライバシーを最優先する場合は、文字起こし・整形ともに「ローカル」をご利用ください**（外部送信は発生しません）。各プロバイダのデータ取り扱いは各社のポリシーに従います（多くは既定でAPIデータを学習利用しない旨を表明しています）。本アプリは設定画面で外部送信を行う旨を明示します。

## 認証情報（APIキー）の保管

クラウド連携のためのAPIキー・資格情報は、**OSのセキュアストレージ**（Windows Credential Manager / macOS Keychain / Linux Secret Service）に保存します。平文の設定ファイルには保存しません。これらの鍵がプロバイダ以外へ送信されることはありません。

## 自動アップデート

本アプリは、更新確認のため GitHub のリリース情報（公開エンドポイント）へアクセスします。この通信で個人を識別する情報は送信しません。配布物の完全性は署名で検証します。

## データの削除

出力先フォルダ内のファイルを削除すれば、対応するジャーナルは端末から削除されます。APIキーは設定から削除できます（OSセキュアストレージから消去されます）。

## お問い合わせ・脆弱性報告

- セキュリティ上の問題は、GitHub の「[Report a vulnerability](https://github.com/Takenori-Kusaka/QuickScribe/security/advisories/new)」（非公開）からご報告ください（詳細は [SECURITY.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/SECURITY.md)）。
- その他のお問い合わせは GitHub リポジトリの Issue をご利用ください。

---

> **Summary (English):** QuickScribe is a local-first voice journal. Recording and local transcription happen on your device; audio is not sent anywhere by default. Cloud transcription (Groq/OpenAI/Deepgram/Azure) and AI refinement (Gemini/Anthropic/OpenAI/Ollama/AWS) are **opt-in only** and send data to the chosen provider solely when you enable them. API keys are stored in your OS secure storage. No analytics or telemetry are collected.
