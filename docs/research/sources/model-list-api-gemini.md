# 一次情報: Google Gemini (Generative Language API) モデル一覧 API

- 取得日: 2026-06-18
- 一次ソース:
  - <https://ai.google.dev/api/models> (models.list リファレンス)
  - <https://ai.google.dev/gemini-api/docs/models> (モデル一覧・命名規則)

## A. モデル一覧エンドポイント

| 項目 | 内容 |
|---|---|
| メソッド | `GET` |
| URL | `https://generativelanguage.googleapis.com/v1beta/models` |
| 認証 | クエリパラメータ `?key=$GEMINI_API_KEY`（ヘッダ `x-goog-api-key: $GEMINI_API_KEY` でも可） |
| ページング | `pageSize`（既定50, 最大1000）／`pageToken`（前回レスポンスの `nextPageToken`） |

## B. レスポンス JSON スキーマ（引用: ai.google.dev/api/models）

トップレベル: `models[]` ＋ `nextPageToken`。

```json
{
  "models": [
    {
      "name": "models/gemini-2.5-flash",
      "baseModelId": "string",
      "version": "string",
      "displayName": "string",
      "description": "string",
      "inputTokenLimit": 0,
      "outputTokenLimit": 0,
      "supportedGenerationMethods": ["generateContent", "countTokens"],
      "thinking": true,
      "temperature": 0,
      "maxTemperature": 0,
      "topP": 0,
      "topK": 0
    }
  ],
  "nextPageToken": "string"
}
```

- 各モデルの `name` は `models/<id>` 形式。実 ID は接頭辞 `models/` を除去して使う。
- テキスト生成可否は `supportedGenerationMethods` に `generateContent` を含むかで判定可能。

## C. 並び順

- 公式リファレンスに配列の並び順保証の記載は**なし**。created/created_at に相当する日付フィールドも Model オブジェクトに存在しない（バージョンは `version` 文字列のみ）。
- → API のソート順に依存せず、ID 文字列からバージョン番号をパースして自前で比較する必要がある。

## D. 命名規則（引用: gemini-api/docs/models, verbatim）

> "Gemini models are available in either _stable_, _preview_, _latest_, or _experimental_ versions."

- **Latest エイリアス**: "Points to the latest release for a specific model variation. ... This alias will get hot-swapped with every new release of a specific model variation."
  - 実在エイリアス例: `gemini-flash-latest`, `gemini-flash-lite-latest`
  - **`pro-latest` は公式ドキュメントに記載なし**（flash / flash-lite のみ latest エイリアス確認）。
- ティア対応（ドキュメント記載）:
  - High: `gemini-3.1-pro-preview`, `gemini-2.5-pro`
  - **Mid（中位）: flash 系** — `gemini-3.5-flash`, `gemini-2.5-flash`
  - Low: `gemini-3.1-flash-lite`, `gemini-2.5-flash-lite`

## E. 反証/限界

- API に「ティア(low/mid/high)」表記フィールドは存在しない → 命名規則（pro/flash/flash-lite）に依存した発見的判定のみ。
- Model オブジェクトに作成日が無いため「最新」は ID のバージョン番号で比較するしかなく、新命名体系が来ると破綻しうる（反証条件: 将来 flash を含まない中位ブランドが登場したら判定ロジック要更新）。
- `gemini-flash-latest` がそのまま models.list に列挙されるかは未確認（反証条件: 実 API 呼び出しで latest エイリアスが配列に出るか要検証。出るなら最優先採用、出なければ自前バージョン比較）。
