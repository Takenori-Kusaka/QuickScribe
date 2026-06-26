# S3.3（拡張）カスタム整形パターン — requirements

> Status: Draft (2026-06-24) / 対象 Issue: S3.3（Epic E3）の拡張。
> 記法: 軽量BDD ＋ EARS。設計核心: [llm-output-control.md](../../research/sources/llm-output-control.md) の出力制御規律を**カスタムでも不変**に保つ。

## ユビキタス言語
- **カスタム整形パターン（custom style）**: ユーザーが定義する整形指示（名前＋指示文）。組み込み4種（構造化/逐語/要約/ブレスト）と並んで選択でき、録音後の自動整形・結果からの再整形チップにも現れる。

## 設計の核心（コア価値の保護）
整形の品質ガード（**事実を捏造しない**／本文だけを `<journal>…</journal>` で囲む／前置き・後書きを出さない）は**システム指示と共通テンプレートに固定**し、カスタムは**指示ブロック（箇条書き部分）だけ**を差し替える。→ 独自パターンでもコア規律が外れない（「リッチすぎず簡便」を保ちつつ自由度を出す）。

## 受入基準（EARS）
- **R1（event）**: When the user creates a custom pattern (name + instruction), the system shall add it to the style list (settings dropdown & restyle chips).
- **R2（event）**: When the user selects a custom pattern and refines, the system shall use the custom instruction in place of the built-in style instruction.
- **R3（ubiquitous・不変条件）**: The system shall always apply the common rules (no fabrication, `<journal>` body-only output) regardless of custom content.
- **R4（unwanted）**: If the custom instruction is empty/blank, then the system shall fall back to the default style.
- **R5（event）**: When the user deletes a custom pattern, the system shall remove it and reset the selection to default if it was selected.
- **R6（state）**: While stored, custom patterns shall persist locally (localStorage) — 端末内のみ（プライバシー）。

## BDD 例
```gherkin
Scenario: カスタムパターンで整形 (R1,R2,R3)
  Given カスタムパターン「議事録（決定事項とToDoを分ける）」を作成
  When そのパターンを選んで整形する
  Then 指示に沿った整形がされ、かつ本文だけが<journal>境界で出力され捏造もない

Scenario: 空指示はフォールバック (R4)
  Given カスタム指示が空白
  When 整形する
  Then 既定スタイル(構造化)で整形される
```

## テストリスト（実装済み・refine.rs）
- [x] custom_instruction が user_prompt に反映され、既定指示には引きずられない（捏造禁止は不変）
- [x] 空白custom は style 既定へフォールバック
- [x] system_prompt（journalタグ境界）はカスタムでも不変

## 範囲外（後続）
パターンの共有/エクスポート・テンプレート配布・パターンごとのモデル指定は後続。本増分は「ローカルでのCRUD＋整形適用」に限定。
