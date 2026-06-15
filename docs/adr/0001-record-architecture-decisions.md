# ADR-0001: アーキテクチャ決定の記録方法（ADRの採用）

- Status: Accepted
- Date: 2026-06-15
- Deciders: Takenori Kusaka

## Context

「なぜその開発・設計にするのか」のポリシーを効果的に管理したい。単一ファイルに詰め込むと
コンテキストが肥大し、Claude Code 等のツールで効果的に活用できない。適切な粒度で分割し
ツリー構造で管理する必要がある。

## Decision

[Michael Nygard 形式の ADR](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) を採用する。

- 1決定 = 1ファイル。`docs/adr/NNNN-<kebab-title>.md`。
- 番号は連番、不可逆（決定の撤回は新ADRで Supersede する）。
- セクション: Context / Decision / Consequences / (任意) Alternatives。
- ステータス: Proposed → Accepted → Deprecated / Superseded by ADR-XXXX。
- 粒度が大きいテーマ（例: テスト戦略全体）はディレクトリでサブツリー化してよい。

## Consequences

- 決定の「理由」が版管理され、後から参加する人間/エージェントが文脈を最小コンテキストで把握できる。
- ファイルが増えるため、`docs/adr/README.md` に索引を維持する運用が要る。
