# 一次情報: OpenAI モデル一覧 API

- 取得日: 2026-06-18
- 一次ソース: <https://developers.openai.com/api/reference/resources/models/methods/list>
  （`platform.openai.com/docs/api-reference/models/list` は WebFetch で 403。developers.openai.com を一次情報として採用）

## A. モデル一覧エンドポイント

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| URL | `https://api.openai.com/v1/models` |
| 認証 | `Authorization: Bearer $OPENAI_API_KEY` |
| ページング | ドキュメント上クエリ/ページングパラメータの記載なし（全件返却） |

## B. レスポンス JSON スキーマ（公式例）

トップレベル: `object`("list") ＋ `data[]`。

```json
{
  "object": "list",
  "data": [
    {
      "id": "model-id-0",
      "object": "model",
      "created": 1686935002,
      "owned_by": "organization-owner"
    }
  ]
}
```

- フィールド: `id`(モデル識別子) / `object`("model"固定) / `created`(Unix秒) / `owned_by`(所有組織)。
- 注: capabilities やティア情報のフィールドは**無い**（id 文字列のみが手がかり）。

## C. 並び順

- 公式リファレンスにソート順の保証**記載なし**（"Sorting: Not specified"）。
- → 並び順に依存せず `created`（Unix秒）降snで自前ソートする必要がある。ただし `created` の信頼性には限界あり（下記E）。

## D. 「ミドルレンジ汎用チャット最新」選択ロジック

OpenAI は明示ティア表記が無いため、id 文字列のヒューリスティクスで定義:

1. ベース系列に限定: id が `gpt-4o` または `gpt-4.1` 等の汎用チャット系で始まるものを候補に。
2. **除外語**でフィルタ（部分一致除外）: `mini`, `nano`, `audio`, `realtime`, `search`, `transcribe`, `tts`, `embedding`, `moderation`, `image`, `vision-preview`, `instruct`, `o1`/`o3`/`o4`(推論系) など。
3. snapshot(日付サフィックス `-YYYY-MM-DD`)が付くものと、付かないローリングエイリアス（例 `gpt-4o`, `gpt-4.1`）の両方が一覧に出る。**ローリングエイリアス（日付サフィックス無し）を優先採用**するのが陳腐化回避に最適。
4. 同系列で最新世代を選ぶ際は `created` 降順 → ただし信頼性が低いので、世代番号（4o < 4.1 …）を id からパースする補助比較を併用。

## E. 反証/限界

- ティア概念が API に無い → 「ミドルレンジ」は完全にこちらの定義依存。GPT 系の「中位」の定義が曖昧（反証条件: OpenAI が公式にティアを定義したらそれに従う）。
- `created` は信頼性が低いことが知られる（同一日付の使い回し・更新時刻でない等）。ソートの一次根拠にしない。
- 一覧には fine-tuned モデルや組織所有の派生、廃止予定が混ざる → `owned_by` が `openai`/`system` 以外を除外する保険が有効。
- ローリングエイリアスが将来も列挙される保証は無い（反証条件: 一覧に `gpt-4o` 等のエイリアスが出ない実環境なら、snapshot 群から最新日付を選ぶ経路にフォールバック）。
