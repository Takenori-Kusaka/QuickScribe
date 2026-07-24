# S4.1 エントリ永続化（保管庫フォルダ） — requirements

> Status: Draft (2026-06-24) / 対象 Issue: #32（Epic E4 #11）
> 記法: 軽量BDD主 ＋ 固まった機能は EARS（[3.4 仕様計画](../../planning/3.4-spec-and-tdd-plan.md) / [ears-syntax](../../research/sources/ears-syntax.md)）。

## ユビキタス言語

- **保管庫（vault）**: エントリを保存する単一のフォルダ。既定は `<ドキュメント>/QuickScribe`、設定で上書き可能。
- **エントリ（entry）**: 1回の文字起こし／整形結果として保管庫に書き出されるテキストファイル（`<yyyymmdd>-note-<ラベル>.txt`。命名は [ADR-0032](../../adr/0032-content-based-entry-filenames.md)）。

## ユーザーストーリー

思考整理のために、録音・整形した内容を**自分の手元の決まった場所に確実に蓄積**したい。場所は既定で迷わず、必要なら自分のフォルダ（クラウド同期配下など）に変えたい。過去の蓄積を失いたくない。

## 受入基準（EARS）

- **R1（ubiquitous）**: The system shall save entries to the vault folder.
- **R2（state）**: While the vault override is unset, the system shall use `<Documents>/QuickScribe` as the vault.
- **R3（event）**: When the user sets a vault folder in settings, the system shall save subsequent entries to that folder.
- **R4（unwanted）**: If the vault folder does not exist when saving, then the system shall create it before writing.
- **R5（unwanted・データ保護）**: If an entry filename already exists in the vault, then the system shall write to a new unique filename (no silent overwrite).
- **R6（event）**: When the user requests "保管庫を開く", the system shall open the vault folder in the OS file manager.

## BDD 例（Given-When-Then）

```gherkin
Feature: 保管庫へのエントリ永続化

  Scenario: 既定の保管庫に保存する (R1,R2,R4)
    Given 保管庫の上書き設定が未設定
    When エントリを保存する
    Then "<ドキュメント>/QuickScribe" が作成され、その中に <yyyymmdd>-note-<ラベル>.txt が書き出される

  Scenario: 保管庫を上書き設定する (R3)
    Given ユーザーが保管庫フォルダに "D:/Journal" を設定
    When エントリを保存する
    Then "D:/Journal/<yyyymmdd>-note-<ラベル>.txt" が書き出される

  Scenario: 同名の衝突で上書きしない (R5)
    Given 保管庫に "20260624-note-今日のメモ.txt" が既に存在
    When 同じ日に同じ冒頭のエントリを保存する
    Then "20260624-note-今日のメモ-2.txt" として保存され、既存ファイルは保持される

  Scenario: 保管庫をOSファイラで開く (R6)
    Given 保管庫が存在する（無ければ作成）
    When "保管庫を開く" を実行する
    Then OSのファイルマネージャで保管庫フォルダが開く
```

## テストリスト（Canon TDD・内側ループ）

- [ ] `note_filename(ts)` が `note-<ts>.txt` を返す（純粋）
- [ ] `next_unique_name(stem, ext, exists)` 衝突なし→`stem.ext`（純粋）
- [ ] `next_unique_name` 衝突あり→`stem-2.ext`/`stem-3.ext`…（純粋・三角測量）
- [ ] `resolve_save_dir` 上書きあり→そのパス（純粋）
- [ ] `resolve_save_dir` 上書きなし→`<Documents>/QuickScribe`（環境依存はガード）
- [ ] 結合: `save_text_in` が衝突時に既存を残し新名で保存（一時ディレクトリ）

## 範囲外（後続Story）

- 出力形式（Markdown/メタデータ）= S4.2 / 内省タグ = S4.3 / スキーマ版＋migration = S4.4。本Storyは「保管庫の場所決定・作成・非破壊保存・導線」に限定。
