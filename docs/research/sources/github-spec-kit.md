# 原典スナップショット: GitHub Spec Kit

- 出自: GitHub 公式 OSS（MIT）。
- 正典URL: https://github.com/github/spec-kit ／ 公式ブログ https://github.blog/ai-and-ml/generative-ai/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit/
- 核心（短い逐語）: 仕様駆動は "specifications become executable, directly generating working implementations rather than just guiding them."（仕様が実行可能になり、実装を導くだけでなく直接生成する）

## 思想（4原則）
1. Intent-driven — 仕様は "what" を "how" の前に定義する
2. ガードレール/組織原則を用いたリッチな仕様作成
3. 一発生成でなく多段階の洗練（multi-step refinement）
4. 仕様解釈に高度なAIモデル能力を活用

## ワークフロー（フェーズ）
Constitution（統治原則）→ Specify（要件・ユーザーストーリー: what/why）→ Clarify（不明点を構造的質問で解消）→ Plan（技術戦略・スタック）→ Tasks（順序付きタスク分解）→ Analyze（成果物間の整合・網羅性検証）→ Implement（体系的に実装）。

## 主要スラッシュコマンド
`/speckit.constitution` `/speckit.specify` `/speckit.plan` `/speckit.tasks` `/speckit.implement`（必須）／`/speckit.clarify` `/speckit.analyze` `/speckit.checklist`（任意）。

## QuickScribeでの含意
各フェーズに「人間の批評チェックポイント」があり、本プロジェクトのPR/レビュー文化と相性が良い。
ただし Spec Kit はやや重量級。探索的フェーズでは Clarify/Analyze を軽く回すなど調整が要る（→Q2で評価）。
