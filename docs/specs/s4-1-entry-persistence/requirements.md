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
- **R7（unwanted・データ可視性）**: If an entry file cannot be decoded as valid UTF-8, then the system shall still list the entry (invalid bytes are shown as the replacement character), regardless of file size or where the invalid bytes occur.
- **R8（event）**: When the user stops a recording, the system shall determine whether to save the audio from the "音声を保存する" setting **as of that moment**（以後の設定変更は、既に停止した録音の保存有無を変えない）.

> R7 の意図: 保管庫はユーザー所有のプレーンファイルで、外部エディタで編集される前提（[ADR-0032](../../adr/0032-content-based-entry-filenames.md)）。文字化けは普通に起きる。コア価値「思考整理・自己理解」にとって最悪の失敗は**「書いたはずの思考が一覧から黙って消える」**ことであり、化けた形でも見えて開ける方が正しい。
> 「ファイルサイズや不正バイトの位置によらず」が要件の核心である。同じ壊れ方をしたエントリが大きさや壊れた位置で見える／見えないに分かれるのは、ユーザーにとって理解不能な挙動になる。
>
> R8 の意図: ユーザーの期待は「録音したときの設定で保存される」。文字起こしジョブの実行時点の設定で決めると、キューの滞留時間次第で結果が変わる（[#663](https://github.com/Takenori-Kusaka/QuickScribe/issues/663) / [#668](https://github.com/Takenori-Kusaka/QuickScribe/pull/668)）。

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

  Scenario: 壊れたエントリも一覧に出る (R7)
    Given 保管庫に不正なUTF-8バイトを含むエントリが2件ある（1件は12KB以下、1件は12KB超）
    When エントリ一覧を取得する
    Then 2件とも一覧に現れる（不正バイトは置換文字として表示される）

  Scenario: 読み込み窓の境界は壊れとみなさない (R7)
    Given 12KB超のエントリで、多バイト文字が部分読み込みの境界にまたがる
    When エントリ一覧を取得する
    Then プレビューに置換文字が混入しない

  Scenario: 音声保存の有無は録音停止時点で確定する (R8)
    Given 「音声を保存する」が OFF の状態で録音し、停止した
    When 文字起こしジョブが実行される前に「音声を保存する」を ON へ切り替える
    Then その録音の音声は保存されない

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
