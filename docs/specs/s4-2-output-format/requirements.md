# S4.2 出力形式（プレーンテキスト / Markdown ＋ メタデータ） — requirements

> Status: Draft (2026-06-27) / 対象 Issue: #33（Epic E4 #11）。
> 記法: 軽量BDD ＋ EARS。コア価値「見返したときの想起しやすさ」を高める。

## ユビキタス言語
- **出力形式（output format）**: エントリの保存形式。`txt`（プレーンテキスト・既定）/ `md`（Markdown）。
- **フロントマター（front-matter）**: Markdown 先頭の YAML メタデータ（作成日時・種別・整形スタイル）。読み返しと将来の横断発見（S4.3）の基盤。

## 受入基準（EARS）
- **R1（state）**: While the output format is `txt`, the system shall save entries as `.txt` with the content only（現行挙動・後方互換）。
- **R2（state）**: While the output format is `md`, the system shall save entries as `.md` with a YAML front-matter（created / type / style）followed by the content.
- **R3（ubiquitous）**: The front-matter shall always include `created`（ISO8601）and `type`（transcript / refined / note）。`style` is included only for refined entries.
- **R4（unwanted・データ保護）**: If front-matter values contain risky characters, then the system shall keep them safe（値はYAMLとして妥当な形に）。
- **R5（event）**: When the user changes the output format, the system shall apply it to subsequent saves（既存ファイルは変更しない）。

## BDD 例
```gherkin
Scenario: Markdownで整形結果を保存 (R2,R3)
  Given 出力形式が Markdown
  When 整形結果を保存する
  Then note-<ts>.md が作られ、先頭に created/type: refined/style のfront-matterが付く

Scenario: プレーンテキスト（既定・後方互換） (R1)
  Given 出力形式が txt（既定）
  When 保存する
  Then note-<ts>.txt が本文のみで作られる
```

## テストリスト（Canon TDD）
- [ ] `doc_extension("md")="md" / 他="txt"`（純粋）
- [ ] `build_document(txt)` は本文そのまま（純粋）
- [ ] `build_document(md, refined+style)` は front-matter に created/type/style を含む（純粋）
- [ ] `build_document(md, transcript)` は style を含まない（純粋・三角測量）
- [ ] front-matter の値が YAML 安全（改行・コロンを含む値の取り扱い）

## 範囲外（後続）
タグ（S4.3）・スキーマ版/migration（S4.4）。本増分は「txt/md 出力＋基本メタデータ」に限定。録音ソース種別のメタ付与は後続（現状の保存経路に未配線のため）。
