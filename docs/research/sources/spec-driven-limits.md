# 原典スナップショット: 重い仕様先行の限界（探索的開発での逆効果）

Q2の反証条件（探索的プロダクトで重い仕様先行は逆効果か）を満たす一次事例。著作権配慮で短い逐語＋出典URL。

## Isoform "The Limits of Spec-Driven Development"（2025）
- URL: https://isoform.ai/blog/the-limits-of-spec-driven-development
- 短い逐語:
  - "For stable contracts and well-understood domains, spec-driven approaches can work great. But for exploratory development that comes with evolving requirements, context-driven approaches adapt better."
  - "keeping specs in sync with the code creates a maintenance tax that grows with system complexity"
  - "Being too fixed to a static spec leads to less iteration, creativity, and emergent solutions."

## Kiro Quick Plan（公式 — 軽量に始める指針）
- URL: https://kiro.dev/docs/specs/quick-plan/
- 趣旨: 「要件・設計を反復する必要がなく出力を信頼できる、よく理解された機能」「速度がレビューより重要なラピッドプロトタイピング」に軽量プランを推奨。逆に未知領域で要件が安定する見込みならフル仕様が有効。

## Fowler "Exploring Gen AI" — SDDツールのAI留保
- URL: https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html
- 短い逐語: 「受入条件はAIが解釈するので100%尊重される保証はない」趣旨。テンプレ・プロンプトを揃えても「エージェントが全指示に従わないのを頻繁に見た」。
- 含意: 外側ループ（受入テスト）は実行してRedを確認する運用が必須。

## GOOS 二重ループ / Living Documentation（接続の核）
- GOOS Ch.1（著者スライド）: https://www.greanetree.com/assets/goos-chapter-1.pdf — 外側=受入(進捗)/内側=単体→実装→リファクタ。Walking Skeleton。
- Gojko Adzic "SbE 10 years": https://gojko.net/2020/03/17/sbe-10-years.html — 仕様とテストを単一文書にすると片方だけ古びることが構造的に不可能（Living Documentation）。

## QuickScribeでの含意
本プロダクトはドメイン理解自体が探索対象＝要件が破棄されやすい → 初期は軽量BDD中心。EARSは安定領域のみ。
外側ループはAIの完了詐称を止めるガードレールとして採用（ただし実行必須）。
