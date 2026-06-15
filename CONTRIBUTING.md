# Contributing to QuickScribe

## ブランチ戦略（GitHub Flow）

- `main` は常にリリース可能な状態を保つ。
- 作業は短命なトピックブランチで: `feat/*` `fix/*` `docs/*` `test/*` `chore/*`。
- 実装・重要設計は **Draft PR** で開始 → CI green → レビュー → Squash merge。
- `main` へのマージで CI がリリースを自動化する（リリース計画は後日 ADR 化）。

## コミット規約（Conventional Commits）

`<type>: <要約>` 形式。type は `feat|fix|docs|test|refactor|chore|perf|build|ci`。
末尾に `Co-Authored-By` トレーラを付ける。

## テスト方針（TDD / t-wada流）

Red → Green → Refactor。テストを先に書く。仮実装・三角測量・明白な実装を使い分ける。
PR には必ずテストを伴う（ドキュメントのみの変更を除く）。

## 設計判断は ADR に

「なぜそうするか」は [docs/adr/](docs/adr/) に 1決定1ファイルで記録する。
形式は [ADR-0001](docs/adr/0001-record-architecture-decisions.md)。

## レビュー

レビューは独立した視点で行う（本プロジェクトでは未知見のレビューAgentをバックグラウンドで起動）。
レビュー観点: 正しさ / コア価値との整合 / 簡便さ / 非機能（セキュリティ・プライバシー・パフォーマンス）。
