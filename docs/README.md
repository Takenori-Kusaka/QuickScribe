# QuickScribe ドキュメント目次

> QuickScribe の設計・要件・運用文書の総合インデックス。
> プロダクトの本質・判断基準は [vision.md](vision.md) と [adr/](adr/README.md) を正とする。

## 1. プロダクト定義（なぜ作るか）

| 文書 | 内容 |
|---|---|
| [vision.md](vision.md) | プロダクトビジョン・コア価値（ニュアンス保持整形）・差別化軸 |
| [research/competitive-landscape.md](research/competitive-landscape.md) | 競合分析（8製品比較・差別化の空白） |
| [planning/3.1-lean-canvas-and-planning.md](planning/3.1-lean-canvas-and-planning.md) | リーンキャンバス・企画 |
| [planning/3.2-stakeholders-and-research.md](planning/3.2-stakeholders-and-research.md) | ステークホルダー・調査計画 |
| [planning/3.3-personas-and-storybook.md](planning/3.3-personas-and-storybook.md) | ペルソナ・ストーリーブック |

## 2. 要件（何を作るか）

| 文書 | 内容 |
|---|---|
| [planning/4-backlog.md](planning/4-backlog.md) | 全体バックログ（Epic/Story 一覧） |
| [non-functional-requirements.md](non-functional-requirements.md) | 非機能要件（ISO/IEC 25010 対応: 性能・信頼性・セキュリティ・プライバシー・a11y・i18n） |
| [specs/](specs/) | 機能仕様（受入基準・BDD 形式。機能別サブディレクトリ） |

## 3. 設計（どう作るか）

| 文書 | 内容 |
|---|---|
| [design.md](design.md) | アーキテクチャ設計（構成図・データフロー・抽象境界） |
| [adr/README.md](adr/README.md) | ADR 索引（設計判断の「なぜ」。1決定1ファイル） |
| [design/brand-colors.md](design/brand-colors.md) | ブランドカラーパレット（トークン定義・ロゴとUIの対応・生hex禁止の運用） |
| [research/](research/) | 技術調査（問い設計メソッド・一次情報は sources/ に保存） |

## 4. 品質・テスト（どう検証するか）

| 文書 | 内容 |
|---|---|
| [planning/3.4-spec-and-tdd-plan.md](planning/3.4-spec-and-tdd-plan.md) | 仕様駆動・TDD 計画 |
| [planning/3.5-waterfall-and-quality.md](planning/3.5-waterfall-and-quality.md) | 品質保証計画（ISO/IEC 25010 対応表） |
| [perf/baseline.md](perf/baseline.md) | パフォーマンス・日本語精度（CER）ベースライン実測 |
| [planning/v1.0.0-readiness.md](planning/v1.0.0-readiness.md) | v1.0.0 最終監査の記録 |

## 5. 運用・リリース（どう届けるか）

| 文書 | 内容 |
|---|---|
| [planning/3.6-release-ops.md](planning/3.6-release-ops.md) | リリース運用計画 |
| [process/versioning-and-channels.md](process/versioning-and-channels.md) | バージョニング（SemVer/release-please）とチャネル方針 |
| [process/release-channels.md](process/release-channels.md) | stable/nightly チャネル Runbook・updater 鍵分離 |
| [process/distribution-and-signing.md](process/distribution-and-signing.md) | 配布とコード署名 |
| [process/windows-local-rust-build.md](process/windows-local-rust-build.md) | Windows ローカル Rust ビルド手順 |
| [process/demo-screenshot.md](process/demo-screenshot.md) | デモ GIF・スクリーンショット生成 |

## 6. ユーザー向けガイド・マーケティング

| 文書 | 内容 |
|---|---|
| [guide/physical-triggers.md](guide/physical-triggers.md) | 物理ボタン（トリガー）連携ガイド |
| [planning/3.7-marketing.md](planning/3.7-marketing.md) | マーケティング計画 |
| [marketing/launch-kit.md](marketing/launch-kit.md) | ローンチキット（Show HN 等の素材） |

関連: リポジトリ直下の [README.md](../README.md)（利用者向け）・[CONTRIBUTING.md](../CONTRIBUTING.md)・[SECURITY.md](../SECURITY.md)。公式サイトのソースは [site/](../site/)。
