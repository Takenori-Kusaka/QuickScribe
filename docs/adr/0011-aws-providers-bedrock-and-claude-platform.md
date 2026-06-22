# ADR-0011: AWSプロバイダ追加（Amazon Bedrock / Claude Platform on AWS）とデュアル認証

- Status: Accepted
- Date: 2026-06-22
- Deciders: Takenori Kusaka
- 関連: [ADR-0005 BYO-Cloud/鍵をコードに埋めない], [ADR-0006 スコープ規律], [ADR-0007 調査規律], [ADR-0010 法的MUST]
- 一次情報: [docs/research/sources/aws-providers.md](../research/sources/aws-providers.md)

## 背景・課題

整形プロバイダに **Amazon Bedrock** と **Claude Platform on AWS（AWS Anthropic）** を追加したい（ユーザー要望）。
既存は同期HTTP(`ureq`)＋`FormattingEngine` trait で Gemini/Anthropic/OpenAI/Ollama を実装。出力は自由生成＋`<journal>`タグ境界抽出（ADR的に確定済）。

## 決定

1. **マルチプロバイダOSS crateへの全面移行は採用しない**（deep researchの結論）。候補(genai/rig/graniet-llm/swiftide等)は全て **tokio必須・async-only** で、同期ureqアーキテクチャの全面async化＋抽象の漏れ(`resolve_latest_model`)の代償が、要件「AWS2種追加」に対して過剰。プロバイダが5社以上に増える/全社ストリーミングUI統一が要件化したら再評価（反証条件）。

2. **Claude Platform on AWS は既存 `AnthropicEngine` の最小改修で対応**。実体は Anthropic Messages API そのもの（公式: "the same API surface"）。差分は base_url（`https://aws-external-anthropic.<region>.api.aws`）と必須ヘッダ `anthropic-workspace-id` のみ。モデルIDは第一者と同じ bare ID。

3. **Amazon Bedrock は新エンジンを追加**。リクエスト/レスポンスは Messages API互換（mantle `/anthropic/v1/messages`）。応答は既存 `extract_tagged_body` に流す。

4. **認証はデュアル（SigV4＋APIキー）両対応**（ユーザー選択）。
   - **APIキー**: ヘッダ認証のみ。同期ureqのまま、署名不要。
   - **SigV4**: `aws-sigv4`(1.4, features `sign-http`,`http1`) ＋ `aws-credential-types`(1.2) で**同期署名**（tokio非依存）。`aws-sdk-bedrockruntime` は async専用のため**不採用**。署名サービス名は **Bedrock=`bedrock`**（ホスト名 `bedrock-runtime` と食い違う点に注意）、**Claude Platform=`aws-external-anthropic`**。資格情報(access key/secret/optional session token/region)はUIから渡し `Credentials::new()` に直接投入（`aws-config`不要）。

5. **依存追加のライセンス**: `aws-sigv4`/`aws-credential-types` は Apache-2.0、`http` は MIT/Apache。すべて deny.toml allow-list 内（ADR-0010 法的MUST整合）。NOTICE は cargo-about が自動収集。

## 実装方針（段階・ADR-0006「全部やりきる」）

- `RefineRequest` に AWS用フィールドを追加（region / workspace_id / 認証モード / AWS access key・secret・session token）。既存プロバイダは未使用（後方互換）。
- `src-tauri/src/aws_sign.rs`: `http::Request` を SigV4署名しヘッダ列を返す純関数（単体テスト対象＝署名の決定性）。
- `engine_for` に `"bedrock"` と `"claude-aws"`(Claude Platform on AWS) を追加。
- `lib.rs refine_text` / `resolve_model` コマンドに Option パラメータを追加（鍵はコードに埋めない/ADR-0005、localStorage保存は当面。将来 keyring=S3.2 へ）。
- フロント: プロバイダ選択に2種追加。AWS選択時のみ region / workspace_id / 認証モード(APIキー or AWS鍵) / 各資格情報の入力欄を出す。

## 結果・トレードオフ

- 同期ureb方針を維持（tokio全面導入を回避）。コア価値(タグ抽出・ニュアンス保持)に非干渉。
- SigV4署名を自前で持つ（`aws-sigv4`に委譲＝署名の正しさは公式crateが担保）。
- 秘密情報(AWS secret)がlocalStorage平文に増える → S3.2 keyring化の優先度を上げる（フォローアップ）。

## 残確認（出荷前）
- bedrock-mantle のAPIキー種別（Anthropic発行 vs AWS資格）をAPI Reference照合。
- SigV4署名の実呼び出し検証（region/サービス名/署名ヘッダ）。
- deny.toml で aws-sigv4 依存ツリーの全ライセンスが通るか（CI cargo deny）。
