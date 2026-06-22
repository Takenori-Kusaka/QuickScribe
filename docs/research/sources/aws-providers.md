# AWSプロバイダ追加（Bedrock / Claude Platform on AWS）とマルチLLM OSS評価（一次情報・deep research）

調査日: 2026-06-22。ADR-0007準拠。目的: 整形プロバイダに **AWS Bedrock** と **Claude Platform on AWS（AWS Anthropic）** を追加。あわせて「組み込み可能なマルチプロバイダOSS crateへ載せ替えるべきか」を評価。

## 結論（意思決定駆動）

1. **全面的なマルチプロバイダcrate移行は見送り。** 現状のハンドロール（同期`ureq`＋`FormattingEngine`＋`<journal>`タグ抽出）が薄く効いており、全候補crateが**tokio必須・async-only**。全面async化＋抽象の漏れ（`resolve_latest_model`等）の代償が保守削減を上回る。要件が「AWS2種追加」に留まる限り過剰投入。
2. **Claude Platform on AWS は crate不要・SigV4不要で追加できる**（最重要発見）。実体は **Anthropic Messages API そのもの**で、base_url と `anthropic-workspace-id` ヘッダの違いだけ。**APIキー認証パスが公式に存在**。
3. **Bedrock** は (a) bedrock-mantle 経由 `/anthropic/v1/messages`（APIキー・同期ureq可）か、(b) SigV4必須環境では `aws-sigv4`（Apache-2.0・**tokio非依存**）で同期署名。`aws-sdk-bedrockruntime` は async専用のため不採用。

## 反証された前提: 「aws-external-anthropic にはcrate/非同期SDKが必要」

`aws-external-anthropic` は crate名ではなく **Claude Platform on AWS のエンドポイント識別子兼SigV4サービス名**。実体は既存 Messages API。

AWS公式 [making-requests](https://docs.aws.amazon.com/claude-platform/latest/userguide/making-requests.html):
> "Claude Platform on AWS uses the Anthropic Messages API (`/v1/messages`), the same API surface as the Claude first-party platform. The differences are the base URL, the authentication method, and a required `anthropic-workspace-id` header"
> "If you prefer not to add the AWS SDK dependency, you can use the base `Anthropic` client." （`ANTHROPIC_BASE_URL='https://aws-external-anthropic.<region>.api.aws'` ＋ `anthropic-workspace-id` ヘッダ）

Anthropic公式 [claude-platform-on-aws](https://platform.claude.com/docs/en/build-with-claude/claude-platform-on-aws):
> "AWS provides the authentication layer (SigV4 **or API key**)"

→ 既存 `AnthropicEngine`（`refine.rs`、同期ureq、`x-api-key`＋`anthropic-version: 2023-06-01`）は、**base_url可変化＋`anthropic-workspace-id`ヘッダ追加**だけで対応。モデルIDは第一者と同じ bare ID（`claude-sonnet-4-6` 等、`anthropic.`プレフィックス無し）。

## Bedrock のRust実装手段（一次情報）

| crate | 版 | ライセンス | 役割 | 非同期 |
|---|---|---|---|---|
| `aws-sdk-bedrockruntime` | 1.135.0 | Apache-2.0 | 高レベルSDK・自動署名 | **async/tokio必須→不採用** |
| `aws-sigv4` | 1.4.5 | Apache-2.0 | 署名計算のみ | **tokio不要**（HTTPクライアント非依存） |
| `aws-credential-types` | 1.2.14 | Apache-2.0 | `Credentials`型（plain stringから生成） | 軽量 |

全てpermissive（Apache-2.0）＝deny.toml/法的MUST整合。

**重大トラップ（一次情報）**: Bedrock の **SigV4サービス署名名は `"bedrock"`**（ホスト名 `bedrock-runtime.{region}.amazonaws.com` と食い違う）。`"bedrock-runtime"`で署名すると拒否。awslabs/aws-sdk-rust `sdk/bedrockruntime/src/config.rs`:
> `pub fn signing_name(&self) -> &'static str { "bedrock" }`

Claude Platform on AWS の署名名は `"aws-external-anthropic"`。

`aws-sigv4` 同期署名スケッチ（ureq維持）:
```rust
let identity = Credentials::new(access_key, secret_key, session_token /*Option*/, None, "user").into();
let params = v4::SigningParams::builder()
    .identity(&identity).region(&region).name("bedrock") // Claude Platform時は "aws-external-anthropic"
    .time(SystemTime::now()).settings(SigningSettings::default()).build().unwrap().into();
let signable = SignableRequest::new("POST", &url, std::iter::empty(), SignableBody::Bytes(body))?;
let (instructions, _) = sign(signable, &params)?.into_parts();
instructions.apply_to_request_http1x(&mut req); // → ureqで送信
```

## crate比較（AWS2種ネイティブ対応の検証・脱落根拠）

| crate | Bedrock | Claude on Bedrock | Ollama | License | 備考 |
|---|---|---|---|---|---|
| genai | ○内蔵(api/sigv4) | ○ | ○ | MIT/Apache | AWS optional化済・素テキスト最良。tokio必須 |
| graniet/llm | ○(optional) | ◎ARN/cross-region | ○ | MIT | 1.x安定。tokio必須 |
| rig (+rig-bedrock) | ○別crate | ○ | ○ | MIT | star 7.7k。tokio必須 |
| allms | ○ | **×Novaのみ** | **×** | MIT | Claude-on-Bedrock不可・Ollama非対応→不適 |
| swiftide | ○ | ○ | ○ | MIT | フルRAG/agent FW・過剰 |
| langchain-rust | **×** | **×** | ○ | MIT | Bedrock/Gemini非対応・crates.io公開停止 |

全面移行が正当化される反証条件: プロバイダが5社以上に増える/全社ストリーミングUI統一/モデル一覧取得共通化が要件化したら genai 等への移行がペイ。

## 段階実装案（ADR-0006: 全部やりきる／段階実装）

- **段階1（推奨・低リスク）**: Claude Platform on AWS を `AnthropicEngine` 改修で追加（base_url可変化＋`anthropic-workspace-id`／APIキー）。同期urepのまま・crate/tokio不要。
- **段階2**: Bedrock を新プロバイダで追加。APIキー(mantle `/anthropic/v1/messages`)で済むなら同期ureqのまま。SigV4-IAMが要件なら `aws-sigv4`(tokio非依存)で同期署名。応答は既存 `extract_tagged_body` に流す（タグ方式と相性良）。

## 残確認点（憶測と区別・出荷前）
- bedrock-mantle のAPIキーの種類（Anthropic発行か/AWS資格か）はAPI Reference照合推奨。Claude Platform on AWS のAPIキーは短期キー（mint可）。
- `genai`の`bedrock_sigv4`が公式aws-sdkか独自署名かは未確認（採用検討時のみ）。
- deny.toml allow-list に Apache-2.0 が含まれるか確認（aws-sigv4採用時）。

## 主要ソース
- [Claude Platform on AWS（Anthropic公式）](https://platform.claude.com/docs/en/build-with-claude/claude-platform-on-aws)
- [Making requests（AWS公式）](https://docs.aws.amazon.com/claude-platform/latest/userguide/making-requests.html)
- [aws-sigv4 docs.rs](https://docs.rs/aws-sigv4) / [aws-sdk-bedrockruntime](https://docs.rs/aws-sdk-bedrockruntime)
- [genai](https://github.com/jeremychone/rust-genai) / [graniet/llm](https://github.com/graniet/llm) / [rig-bedrock](https://crates.io/crates/rig-bedrock)
