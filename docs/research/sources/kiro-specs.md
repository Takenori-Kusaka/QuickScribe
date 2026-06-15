# 原典スナップショット: AWS Kiro 仕様駆動ワークフロー

- 出自: AWS（Kiro = agentic開発IDE）。
- 正典URL: https://kiro.dev/docs/specs/ ／ Feature Specs https://kiro.dev/docs/specs/feature-specs/ ／ Blog https://kiro.dev/blog/kiro-and-the-future-of-software-development/
  （注: 旧URL `/docs/specs/concepts/` は現在404。正は `/docs/specs/`）

## ワークフロー
Prompt → **Requirements → Design → Tasks → Code**（自然言語プロンプトを構造化された仕様3点へ）。

- **requirements.md**: 自然言語の意図を **EARS記法**の要件・受入基準へ変換。各要件は `WHEN [条件/イベント] THE SYSTEM SHALL [期待動作]` のパターン。ユーザーストーリーごとにEARSの受入基準でエッジケースを網羅。
- **design.md**: 技術アーキテクチャ・シーケンス図・実装上の考慮。コンポーネントと相互作用の全体像。
- **tasks.md**: 依存関係で順序付けた離散タスク（任意で網羅的テスト付き）。

## 特徴
- Feature Specs は Requirements-First / Design-First の2変種。
- 要件分析は neuro-symbolic アプローチの3段（refinement → auto-formalization → logical analysis）で、意味的曖昧さ・不整合・網羅性問題を検出。

## QuickScribeでの含意
「Spec(要件)とDesign(設計)の明確な分離」を最も体現する流儀。EARSの採否判断に直結。
DDDのユビキタス言語をrequirements.mdの語彙に一致させると、仕様→設計→実装の語彙貫通が効く。
