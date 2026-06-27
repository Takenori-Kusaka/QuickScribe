# 設定・エントリのスキーマ版管理＋非破壊migration — requirements

> Status: Draft (2026-06-27) / 対象 Issue: #38（S5.3 設定スキーマ）, #35（S4.4 データ版＋migration）
> 方針: [ADR-0017](../../adr/0017-schema-versioning-and-migration.md)。記法: 軽量BDD＋EARS。

## 受入基準（EARS）

### 設定（#38）
- **R1（event）**: When settings are loaded, the system shall validate enum-like values and clamp invalid ones to defaults（破損耐性）。
- **R2（ubiquitous）**: The system shall stamp `settingsVersion` and preserve unknown keys（非破壊）。
- **R3（unwanted）**: If `refineStyle` references a deleted custom pattern, then the system shall fall back to default。

### エントリ（#35）
- **R4（state, md）**: While saving md, the system shall write a `schema: <N>` front-matter marker。
- **R5（ubiquitous）**: The parser shall tolerate missing/unknown schema（legacy=0）and shall not rewrite existing files（非破壊）。
- **R6（ubiquitous・既存不変）**: 同名衝突は一意名へ退避（上書きしない）／書き込みは一時名→rename。

## BDD 例
```gherkin
Scenario: 壊れた設定値を既定へ（R1）
  Given localStorage の outputFormat が "xml"（不正）
  When 起動して設定を読み込む
  Then outputFormat は "txt"（既定）に補正される

Scenario: md エントリに版マーカー（R4）
  When Markdownでエントリを保存する
  Then 先頭フロントマターに schema: 1 が入る
```

## テスト
- [x] build_document md に `schema: 1` が入る（lib.rs）
- [x] validateSettings の enum クランプ（型レベル＋目視）。
- 既存の非破壊保存テスト（save_document_does_not_overwrite_existing）で R6 を担保。

## 範囲外
具体的な版N→N+1の破壊的移行は発生時に別ADRで実装。本増分は版マーカー＋検証＝土台のみ。
