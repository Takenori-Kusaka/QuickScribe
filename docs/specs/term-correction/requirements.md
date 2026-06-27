# 用語補正フェーズ（文字起こし→整形の間） — requirements

> Status: Draft (2026-06-27) / Epic E2-E3 境界。ユーザー要望（手修正の緩和）。
> 例: 「レーメン」→ 実際の発話「LLM」。誤変換を確認・置換してから整形へ。

## ユビキタス言語
- **用語補正（term correction）**: 文字起こし中の誤変換疑い語をAIが検出し、文脈から正しい表記を提案。ユーザーが置換/非置換を選び、文字起こしを更新する工程。

## 受入基準（EARS）
- **R1（event）**: When the user runs "用語チェック", the system shall detect suspected mis-transcribed terms and suggest replacements（既存整形プロバイダを再利用）。
- **R2（ubiquitous）**: Each suggestion shall be individually toggleable（置換する/しない）and the suggested text editable。
- **R3（event）**: When the user applies, the system shall replace selected terms in the transcript（全置換）and continue to refine on the corrected transcript。
- **R4（unwanted）**: If no suspected terms, then the system shall say so and change nothing。
- **R5（ubiquitous・非破壊）**: 校正候補の取得は保存しない（save=false）。本文の整形・書き換えは行わない（検出のみ）。

## BDD 例
```gherkin
Scenario: 誤変換を確認して置換 (R1,R2,R3)
  Given 文字起こしに「レーメン」がある（実際は「LLM」）
  When 「用語チェック」を実行
  Then 「レーメン → LLM」が候補に出る
  And 置換にチェックして適用すると文字起こしの「レーメン」が「LLM」に置換される
```

## テスト（実装済み・src/lib/corrections）
- [x] parseCorrections: 区切り行→候補（理由なし許容/原文=提案や空は除外）
- [x] applyCorrections: replace=trueのみ全置換＋件数、空提案は不適用
- [ ] 実機: 実音声で誤変換候補が出る（CIは鍵なし＝手動）

## 設計メモ
既存 refine_text を save=false＋校正専用 custom_instruction で再利用（独自API追加なし）。出力は <journal> 内に `原文 ||| 提案 ||| 理由` の区切り行。精度が難しいため**置換は任意（既定チェックだが「すべて置換しない」も提供）**。

## 範囲外（後続）
ユーザー辞書（固有名詞の登録で再発防止）・自動適用（高確信のみ）・整形自動連携は後続。
