# LLM出力制御：整形結果から「本文だけ」を安定取得する（一次情報・deep research）

調査日: 2026-06-22。ADR-0007（原典主義／問い設計）に従い、4本のdeep research（Anthropic / Gemini / OpenAI / 横断品質）を一次情報で実施。
目的: 整形(refine)結果の「前置き(AIの挨拶)＋本文＋後書き(補足)」の3層を排し、**本文だけ**を取り出す。
**最優先制約**: QuickScribeのコア価値＝「ニュアンスを残しつつ思考を整理する整形の知性」。**本文の品質・ニュアンスを犠牲にしてはならない。**

本アプリはRustのraw HTTP（`ureq`）で各社APIを直接叩く（Rustに公式SDK無し→raw HTTPが正攻法）。よって本書はREST/ワイヤ形式を一次情報とする。

---

## 結論（先に）

**全プロバイダ共通で「自由形式生成 ＋ XMLタグ境界抽出（`<journal>…</journal>`）」を主軸に採用。本文（長文・ニュアンス命）は絶対にJSON文字列フィールドに入れない。**

これは「構造化で品質が落ちる派」と「落ちない派」の双方が合意する唯一の一点 ——「**推論・表現の場(reasoning/expression space)を奪う設計だけが品質を落とす**」—— から直接導かれる。本文をJSON値として直接吐かせる＝表現の場を奪う＝コア価値（ニュアンス保持）と衝突する。

### なぜ「各社ネイティブ構造化出力で本文1フィールド」を捨てたか（当初案の棄却理由）

当初は Gemini=responseSchema / Anthropic=tool use強制 / OpenAI=json_schema strict で `{"body": "<markdown>"}` を強制する案だった。一次情報で次が判明し棄却:

1. **constrained decodingが保証するのは「JSONの構文妥当性」だけで、本文の品質は一切救わない。** 守りたいのは後者なので安心材料にならない（OpenAI/Anthropic両公式の仕組み説明から導出）。
2. **長文をJSON文字列に詰めると実測で品質が落ち、壊れやすい。** aiderの実証: 「all of the models did worse on the benchmark when asked to return code in a structured JSON response」「The models can reliably produce valid JSON, but code inside is more prone to syntax errors」。Claude 3.5 Sonnetで悪化大。(<https://aider.chat/2024/08/14/code-in-json.html>)
3. **truncation＝即JSON破損。** max_tokens/MAX_TOKENS途中切れ時、JSON文字列の途中で切れると全体がパース不能（プレーンテキストより壊れ方が致命的）。3社とも該当（後述）。
4. **Anthropic: tool use強制は extended/adaptive thinking と併用すると400エラー**（公式明記）。思考整理の知性を活かすにはthinkingを切りたくない＝コア価値と衝突。
5. **prefill（`{`先頭埋めでJSON強制）は Sonnet 4.6 / Opus 4.6+ で400**（裏取り済み）。古典手法は封じられた。

---

## 論点1: 構造化出力は品質を下げるか（両論・一次情報）

### 劣化派 — Tam et al. 2024 "Let Me Speak Freely?"（EMNLP 2024 Industry, arXiv 2408.02442）
抄録逐語:
> "we observe a significant decline in LLMs' reasoning abilities under format restrictions. Furthermore, we find that stricter format constraints generally lead to greater performance degradation in reasoning tasks."

数値例: GSM8K / GPT-3.5-turbo は自然言語 75.99% → JSON-mode 49.25%（約 -26.7pt）。**ただし分類タスクでは構造化が有利**と著者も認める。

**最重要の但し書き（論文本文逐語）**——推奨の核心:
> "Comparing NL-to-Format with unrestricted Natural Language responses, we observe nearly identical performance across most models, as both derive answers from the same initial natural language response."
> "a balance must be struck between the desire for easily parseable, structured outputs and the need to preserve the LLM's inherent reasoning abilities."

→ **劣化派の論文自身が、「まず自由形式で生成→後で構造化(NL-to-Format=2段階)」なら劣化はほぼ消えると実証。** 劣化するのは「いきなりJSON値として答えさせる」場合。

### 反論派 — dottxt "Say What You Mean"（<https://blog.dottxt.ai/say-what-you-mean.html>）
逐語: "structured generation is not the same thing as JSON-mode." / "the paper uses different prompts for structured generation and unstructured generation." / "structured generation outperforms unstructured generation."
→ プロンプトを揃えると構造化が僅差で上。批判の核心は「比較が不公平」「JSON-mode≠保証付きconstrained decodingの混同」。

### 中立整理（3つの別問題）
(i) スキーマ制約そのもの（構文保証）/ (ii) プロンプトでJSONを頼む（保証なし）/ (iii) 思考の場を奪う —— は別物。
**文献的合意**: 「(iii)を放置して(i)/(ii)を課すと劣化。推論・表現欄を確保すれば制約自体は品質を落とさない（分類ではむしろ上がる）」。

---

## 論点2: JSON封入 vs 後処理strip vs 2ステップ

- **方式1 JSON封入 `{"body":"…長文markdown…"}` → 非推奨**。論点1・aider実証・ローカルIssue群が品質劣化と破損を裏付け。改行・引用符・コードフェンス ``` でパースが壊れやすい。
- **方式2 自由生成 ＋ 前置き/後書きstrip（XMLタグ境界） → 採用**。ニュアンス保持に最適（制約ゼロ）。Anthropic公式が正攻法と明言:
  > "Respond directly without preamble. Do not start with phrases like 'Here is...', 'Based on...', etc." / "If the occasional preamble slips through, strip it in post-processing."
  （<https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/be-clear-and-direct>）
  決定打は **XMLタグで本文を囲ませる**（`<journal>…</journal>`）。抽出は「タグ内を抜く」決定的処理になり、言語依存の文字列マッチを避けられる。JSONと違い**エスケープ不要で改行・引用符・``` を一切壊さない**。
- **方式3 2ステップ(整形→抽出) → 限定用途のみ**。抽出をLLMにすると本文を改変/要約しニュアンスを削るリスク＋レイテンシ/コスト2倍。コードでタグ抽出するなら実質方式2。本アプリではオーバーキル。

---

## 論点3: 各社の出力挙動・失敗モード（実装で必ず扱う）

### Anthropic Messages API
- 強制ツール(`tool_choice:{type:"tool"}`)は **extended/adaptive thinkingと併用不可（400）**。(define-tools 公式)
- 強制ツール時は「natural language response or explanation を tool_use の前に出さなくなる」(同)。
- 失敗: `stop_reason=="max_tokens"`（途中切れ）/ `"refusal"`（200で返るが本文がスキーマ非準拠）。`content[0]`を読む前に`stop_reason`を確認。
- prefillは最新世代で400。
- 出典: structured-outputs.md / agents-and-tools/tool-use/define-tools.md / strict-tool-use.md（platform.claude.com）

### Google Gemini generateContent (v1beta)
- JSON強制は `generationConfig.responseMimeType="application/json"` ＋ `responseSchema`（OpenAPI 3.0 Schemaのサブセット。型は**大文字** OBJECT/STRING…）。新形式 `responseFormat{text{mimeType,schema}}` との**過渡期**で、対象モデルでどちらが通るかは実呼び出しで確定が必要。
- 巨大/深いネストのスキーマは拒否され得る（本件の本文取得では構造化を使わないので無関係）。
- `finishReason`: STOP / MAX_TOKENS / SAFETY / RECITATION。**STOPでも `candidates[0].content.parts[0].text` が空になり得る**（公式フォーラム多数報告）。
- 出典: ai.google.dev/gemini-api/docs/structured-output, ai.google.dev/api/generate-content, troubleshooting, safety-settings

### OpenAI Chat Completions
- `response_format:{type:"json_schema", json_schema:{name, strict:true, schema:{…, additionalProperties:false, required:全列挙}}}`。strict=constrained decoding。
- 非対応: `maxLength`/`minLength`/`pattern`/`format`、数値範囲、配列長 等（→ 本文長はプロンプト+max_tokensで制御）。
- 失敗: `message.refusal`（refusal時 `content` は null）/ `finish_reason=="length"`（途中切れ）。対応下限 `gpt-4o-2024-08-06`。
- 出典: developers.openai.com/api/docs/guides/structured-outputs（platform URLから301）

### ローカル Ollama / llama.cpp
- `format`にJSON Schemaを渡せるが、**保証は構文妥当性だけ**。llama.cpp GBNF公式: "Grammars currently have performance gotchas"。Ollama公式: プロンプトでJSON指示しないと大量の空白を吐く/temperature=0推奨。
- **長文を1つのJSON文字列に詰めるのは最も壊れやすい**（反復ループでJSON未終端、長コンテキストでエスケープ失敗の実Issue多数: ollama#15502, hermes-agent#13042）。
- 出典: github.com/ggml-org/llama.cpp grammars/README.md, ollama.com/blog/structured-outputs, ollama#15502

---

## 論点4: QuickScribeへの実装方針（確定）

整形は既に `FormattingEngine` で抽象化済み（PR#343, commit 499b5d3）。以下を全エンジン共通で実装する。

### 主軸: 自由生成 ＋ XMLタグ境界抽出
1. システム指示で「整形後の本文だけを `<journal>…</journal>` で囲み、タグ外に前置き/後書きを書くな」と明示（`RefineStyle::system_prompt`）。**本文をJSONに入れない。**
2. 応答からタグ内本文を**決定的に抽出**（最初の`<journal>`〜最後の`</journal>`＝最外）。
3. 各社のリクエストbodyは現状（system＋messages）のまま。構造化出力・tool use・prefillは使わない。

### フォールバック階層（堅牢性。最初から設計）
- **L1**: `<journal>…</journal>` 正常 → タグ内抽出。
- **L1'**: 終了タグ欠落（truncation等）→ 開始タグ以降を本文として救済。
- **L2**: タグ欠落 → 定型前置き行（日本語/英語: 「はい」「以下が」「整形しました」「Here is」等）を**保守的に1行だけ**除去。過剰除去で本文を壊さない。
- **L3**: それでも空 → 既存の空エラー（整形失敗）。※ユーザーの文字起こし原文は別途保存済みで失われない（コア価値: 整形は付加価値、原文喪失は事故）。

### プロバイダ別の留意
- 3社＋ローカルとも本文は自由形式が最も安全・実装最小（1呼び出し）。
- ローカル(Ollama)はモデル差大。採用モデルで「タグ閉じ忘れ率・反復ループ率・前置き混入率」を**出荷前に実測**（残課題）。
- prefillは使わない（Opus 4.8非サポート）。

### 残課題（出荷前/将来）
1. 採用ローカルモデルでの実測スパイク（タグ閉じ忘れ・反復ループ・前置き混入頻度）。
2. 日本語前置きパターン辞書のL2拡充。
3. タグ衝突対策（本文中にユーザーが `<journal>` 様の語を発話した稀ケース。必要なら区切りトークンをランダム化）。
4. （任意）「構造化メタデータ（タグ付け等）が必要になったら」本文とは別フィールドで、NL-to-Format（本文は自由生成→別途構造化）にする。

---

## 主要ソース一覧

| URL | 種別 | 用途 |
|---|---|---|
| https://arxiv.org/abs/2408.02442 (Tam et al., EMNLP 2024) | 査読 | 構造化で推論劣化／NL-to-Formatは非劣化 |
| https://blog.dottxt.ai/say-what-you-mean.html | ベンダー | 反論（構造化≠JSON-mode、比較公平性） |
| https://aider.chat/2024/08/14/code-in-json.html | 実証 | 長文/コードをJSONに入れると全モデルで品質低下 |
| https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/be-clear-and-direct | 一次(公式) | 「前置きなし＋漏れたら後処理strip」正攻法 |
| https://platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools.md | 一次(公式) | 強制ツール×thinking=400、強制時NL抑制 |
| https://platform.claude.com/docs/en/build-with-claude/structured-outputs.md | 一次(公式) | constrained decoding、max_tokens/refusal、prefill非互換 |
| https://ai.google.dev/gemini-api/docs/structured-output | 一次(公式) | responseMimeType/Schema、複雑度制限、過渡期REST形 |
| https://ai.google.dev/api/generate-content | 一次(公式) | generationConfig定義、finishReason |
| https://developers.openai.com/api/docs/guides/structured-outputs | 一次(公式) | json_schema strict正規形、refusal、非対応keyword、モデル要件 |
| https://github.com/ggml-org/llama.cpp/blob/master/grammars/README.md | 一次(公式) | GBNFは構文保証のみ、性能落とし穴 |
| https://ollama.com/blog/structured-outputs | 一次(公式) | format、temperature=0/プロンプト指示推奨 |
| https://github.com/ollama/ollama/issues/15502 | 準一次 | 長文JSONの反復ループ/未終端破損 |

（注: 「長文proseをJSONに包むなというOpenAI公式の明示文言」は確認できず、憶測で埋めず欠落として記録。方式1回避の最強根拠はaiderの実証。「構造化が長文markdown品質を下げる」Gemini公式明言も確認できず＝一次情報の沈黙として記録。）
