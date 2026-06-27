# S4.3 内省タグ（Phase 0: タグ付与） — requirements

> Status: Draft (2026-06-27) / 対象 Issue: #34（Epic E4 #11）/ [ADR-0015](../../adr/0015-introspective-tags-and-cross-entry-discovery.md)
> 記法: 軽量BDD ＋ EARS。Phase 0=タグ付与（一覧/横断発見は後続）。

## ユビキタス言語
- **内省タグ（tag）**: エントリに付ける短い語。後から思考を束ねて見返す入口。Markdownでは frontmatter `tags`、プレーンテキストでは末尾 `Tags:` 行に保存。

## 受入基準（EARS）
- **R1（event）**: When the user enters tags and refines/saves, the system shall attach the tags to the saved entry's metadata.
- **R2（state, md）**: While output format is `md`, tags shall be written as YAML `tags: ["a","b"]` in front-matter（Obsidian互換）。
- **R3（state, txt）**: While output format is `txt`, tags shall be appended as a trailing `Tags: a, b` line（形式に依らず残す）。
- **R4（ubiquitous）**: The system shall normalize tag input（カンマ/空白区切り・重複除去・空除去・先頭`#`除去）。
- **R5（unwanted）**: If no tags are entered, then the system shall save as before（タグ行/フィールドを付けない・低摩擦）。

## BDD 例
```gherkin
Scenario: タグ付きで整形結果を保存(md) (R1,R2)
  Given 出力形式がMarkdown、タグ欄に "仕事, 不安"
  When 整形する
  Then 保存される .md の front-matter に tags: ["仕事","不安"] が入る

Scenario: タグ未入力は従来どおり (R5)
  Given タグ欄が空
  When 保存する
  Then タグ関連の行は付かない
```

## テストリスト（実装済み・lib.rs）
- [x] md: tags 配列が front-matter に入る
- [x] txt: 末尾に `Tags: ...` 行が付く
- [x] tags 無しは md/txt とも従来どおり（行を出さない）
- [x] フロント `parseTags`: 区切り/重複/空/先頭# の正規化（型レベルで担保、目視確認）

## Phase 1（実装済み）: 保管庫エントリの一覧＋タグ/全文絞り込み
- ヘッダの📁から保管庫パネルを開く。`list_entries` が .txt/.md を解析(frontmatter/Tags行)し created 降順で一覧。
- 受入: R7 一覧表示 / R8 タグチップで絞り込み(AND) / R9 本文・タグ・名前の全文検索 / R10 クリックで本文閲覧。
- テスト(vault.rs 純粋): frontmatter解析(quoted/箇条書きtags) / txt末尾Tags / プレーン / preview整形。

## 範囲外（後続Phase）
- Phase 2: 複数エントリからの横断発見（AI抽出＝コア価値の最終形）。
