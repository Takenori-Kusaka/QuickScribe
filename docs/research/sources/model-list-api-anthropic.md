# 一次情報: Anthropic (Claude) モデル一覧 API

- 取得日: 2026-06-18
- 一次ソース: <https://platform.claude.com/docs/en/api/models-list>
  （旧 `https://docs.anthropic.com/en/api/models-list` から 301 リダイレクト）

## A. モデル一覧エンドポイント

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| URL | `https://api.anthropic.com/v1/models` |
| 認証 | `x-api-key: $ANTHROPIC_API_KEY` ＋ `anthropic-version: 2023-06-01` |
| ページング | `before_id` / `after_id`（カーソル）／`limit`（既定20, 範囲1〜1000） |

## B. レスポンス JSON スキーマ

トップレベル: `data[]`, `first_id`, `last_id`, `has_more`。
各モデル: `id`, `type`("model"固定), `display_name`, `created_at`(RFC3339文字列), `max_input_tokens`, `max_tokens`, `capabilities{...}`。

引用（公式レスポンス例の抜粋）:

```json
{
  "data": [
    {
      "id": "claude-opus-4-6",
      "created_at": "2026-02-04T00:00:00Z",
      "display_name": "Claude Opus 4.6",
      "max_input_tokens": 0,
      "max_tokens": 0,
      "type": "model",
      "capabilities": { "thinking": { "supported": true } }
    }
  ],
  "first_id": "first_id",
  "has_more": true,
  "last_id": "last_id"
}
```

## C. 並び順（重要）

- 公式リファレンスに明記: **"More recently released models are listed first."**（新しい順＝降順）。
- → `data[0]` 側が最新。さらに `created_at`（RFC3339）でも自前ソート可能。

## D. 「最新 sonnet（中位）」選択ロジック

- ティア対応（命名規則）: opus = 上位 / **sonnet = 中位** / haiku = 下位。
- model id 命名: `claude-sonnet-4-5` のように `claude-<tier>-<major>-<minor>`。
- アルゴリズム: `data[]` を走査し id が `sonnet` を含むものに絞り、配列が新しい順なので**先頭の sonnet** を採用。保険として `created_at` 降順ソート後の先頭でも同義。

## E. 反証/限界

- `created_at` は「リリース日不明時は epoch 値になり得る」と明記 → 日付ソートが当てにならないケースあり。並び順保証（新しい順）を主に使い、created_at は補助に留める。
- 廃止予定モデルも一覧に混在しうる（反証条件: deprecated フラグは現スキーマに無いため API だけでは除外不可。固定フォールバック必須）。
- sonnet が将来も中位ブランドである保証は命名規則のみ（反証条件: ブランド改称時は要更新）。
