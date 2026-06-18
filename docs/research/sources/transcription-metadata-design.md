# 文字起こしメタデータ設計 — 一次情報ソース

取得日: 2026-06-18
対象: QuickScribe(ローカル個人ボイスジャーナル)向けメタデータ抽出の実装現実性検討。
関連: `whisper-metadata.md`(STT由来メタデータ), `refine.rs`(LLM整形)。

## 参照URL一覧

- OpenAI Structured Outputs: https://developers.openai.com/api/docs/guides/structured-outputs
  (旧 https://platform.openai.com/docs/guides/structured-outputs から301)
- Gemini Structured Output: https://ai.google.dev/gemini-api/docs/structured-output
- Anthropic Tool use overview: https://platform.claude.com/docs/en/docs/build-with-claude/tool-use/overview
- Anthropic Structured Outputs: https://platform.claude.com/docs/en/build-with-claude/structured-outputs
  (HN告知: https://news.ycombinator.com/item?id=45930598)

## 引用・要点(一次情報)

### A. OpenAI Structured Outputs
- 要求方法: `response_format` に `{"type":"json_schema", ..., "strict": true}`。
  Responses API では `text: { format: { type: "json_schema", strict: true, ... } }`。
- モデル: GPT-4o-mini, GPT-4o-2024-08-06 以降(GPT-5.5含む)。旧モデルは JSON mode を使用。
- スキーマ制約(strict時):
  - すべてのプロパティを `required` に列挙すること(必須)。
  - `"additionalProperties": false` を必須。
  - ネスト深さ・プロパティ数に上限あり(数値は抜粋に明記なし)。
  - `enum` で文字列を選択肢に拘束可。スキーマ準拠を型保証("explicit refusals"あり)。
- JSON mode(旧)は「有効なJSON」だがスキーマ準拠は保証しない。

### B. Gemini Structured Output
- 要求方法: `generationConfig` で `responseMimeType: "application/json"` + `responseSchema`。
  Go SDK は `responseJsonSchema` 直接指定も可。
- サポート型: string/number/integer/boolean/object/array/null。`title`/`description` で指示。
- フィールド: object={properties, required, additionalProperties}, string={enum, format(date-time/date/time)},
  number/integer={enum, minimum, maximum}, array={items, prefixItems, minItems, maxItems}。
- `enum` で分類拘束可(例: ["phishing","scam",...])。
- Gemini 2.0 系は `propertyOrdering` の明示が必要。
- 制約: 「JSON Schema仕様の全機能はサポートされない。未対応プロパティは無視される」。
  巨大/深いスキーマは拒否されうる→名前短縮・ネスト削減で対処。

### C. Anthropic(Claude) Structured Outputs ※2025-11 GA化
- 2つのモード:
  1. JSON outputs: `output_config.format` に `{"type":"json_schema","schema":{...}}`。
     (旧 `output_format` パラメータと旧ベータヘッダ `anthropic-beta: structured-outputs-2025-11-13`
      は移行期間中も動作するが非推奨。現行は `output_config.format`。)
  2. strict tool use: `tools[].strict: true` + `input_schema`(JSON Schema)。
     `tool_choice` で特定ツールを強制すれば tool_use.input が構造化JSONになる。
- 仕組み: JSON schema を文法(grammar)にコンパイルし、推論中にトークン生成を制限(プロンプト依存でない)。
- モデル: Opus 4.8/4.7/4.6/4.5, Sonnet 4.6/4.5, Haiku 4.5, Fable 5, Mythos 5 等(GA)。
- スキーマ制約:
  - 非対応: 再帰スキーマ, enum内の複合型, 外部`$ref`, 数値制約(minimum/maximum/multipleOf),
    文字列長制約(minLength/maxLength), `minItems`が0/1以外, `additionalProperties: false以外`。
  - 対応: `enum`(string/number/bool/null), `const`, `anyOf`/`allOf`(限定), 内部`$ref`/`$defs`,
    string format(date-time/time/date/duration/email/hostname/uri/ipv4/ipv6/uuid), array `minItems`(0/1のみ)。
  - 複雑度上限/リクエスト: strict tools最大20, optionalパラメータ最大24, union型パラメータ最大16。
- 制約注記: citations / message prefill とは JSON outputs モードを併用不可。
  refusal や max_tokens 切り詰め時はスキーマ不一致になりうる。文法は24h キャッシュ。

## 3社共通の設計含意(QuickScribe向け)
- 3社とも JSON Schema ベースの構造化出力をネイティブ提供。共通サブセット(object/properties/
  required/enum/string/number/array/description)に絞れば1つのスキーマで横断可能。
- 共通最小サブセットの注意点:
  - すべて `additionalProperties:false` + 全プロパティ `required`(OpenAI strict要件)に寄せるのが安全。
    任意項目は `["string","null"]` の union か enum で表現。
  - 数値/長さ制約(minimum/maxLength等)は Anthropic で非対応 → スキーマに入れず、プロンプトで誘導。
  - 深いネスト・巨大スキーマは Gemini で拒否されうる → フラット寄りに保つ。
  - Gemini 2.0 系のみ `propertyOrdering` 必要(2.5系は不要の方向)。
- refine.rs は現状プレーンテキスト出力。各社の構造化出力に載せ替えれば「整形テキスト+メタJSON」を
  1往復で取得できる(プロバイダ差はリクエスト組み立てとレスポンス取り出しのみ)。

## STT由来メタの算出(whisper-metadata.md より、本検討での再整理)
- 話速: セグメントごとに `文字数 / (t1-t0 秒)`。t0/t1 はセンチ秒(÷100で秒)。
- 発話長: セグメント `t1-t0`、全体は `Σ(t1-t0)` と末尾t1。無音は区間差から推定可。
- セグメント信頼度: `full_get_token_prob` のトークン確率を集計(平均/最小)。avg_logprob 相当。
  no_speech_prob は whisper-rs 0.14 では未露出(精度フィルタには使えない)。
- 言語: `full_lang_id_from_state` / `lang_detect`(確率ベクトル付き)。
