# Architecture Decision Records (ADR)

本プロジェクトの設計・開発ポリシーの「なぜ」を記録する。形式は [ADR-0001](0001-record-architecture-decisions.md) を参照。

| # | Title | Status |
|---|---|---|
| [0001](0001-record-architecture-decisions.md) | ADRの採用 | Accepted |
| [0002](0002-stt-engine-strategy.md) | 文字起こし(STT)エンジン戦略 | Accepted |
| [0003](0003-reject-google-docs-automation.md) | Google Docs自動駆動を採用しない | Accepted |
| [0004](0004-product-positioning-voice-journal.md) | ポジショニング=思考整理ボイスジャーナル | Accepted |
| [0005](0005-tech-stack.md) | アプリケーション技術スタック（Tauri 2 + Svelte 5） | Accepted |
| [0006](0006-scope-completeness-policy.md) | スコープ完全性ポリシー（独断縮小を禁ず） | Accepted |
| [0007](0007-research-question-framing-method.md) | deep researchの問い設計メソッド標準化 | Accepted |
| [0008](0008-licensing-and-distribution.md) | ライセンス(MIT)・無料・ポータブル配布・無償OSS署名 | Accepted |
| [0009](0009-release-versioning-and-1.0-scope.md) | リリースのバージョニングと v1.0.0 スコープ | Accepted |
| [0010](0010-v0.1.0-gate-legal-must-only.md) | v0.1.0 のゲート条件を「法的MUSTのみ」に限定 | Accepted |
| [0011](0011-aws-providers-bedrock-and-claude-platform.md) | AWSプロバイダ追加（Bedrock / Claude Platform on AWS）＋デュアル認証 | Accepted |
| [0012](0012-windows-multiarch-multisimd-distribution.md) | Windows配布（マルチアーキ x64/ARM64 ＋ SIMD ＋ CPUガード） | Accepted |
| [0013](0013-system-audio-loopback-and-source-unification.md) | システム音ループバックと「録音ソース」抽象の統一（S1.3） | Accepted |
| [0014](0014-physical-trigger-integration.md) | 物理トリガー連携（ホットキー橋渡し＋start/stop分離＋モーメンタリ） | Accepted |
| [0015](0015-introspective-tags-and-cross-entry-discovery.md) | 内省タグと横断発見（S4.3・段階実装） | Accepted |
| [0016](0016-cloud-stt-providers.md) | クラウドSTTプロバイダ（Groq/OpenAI/Deepgram/Azure） | Accepted |
| [0017](0017-schema-versioning-and-migration.md) | スキーマ版管理と非破壊マイグレーション（S5.3/S4.4） | Accepted |
| [0018](0018-output-language-translation.md) | 整形出力言語（翻訳）を専用の永続設定・1パスで提供（2パス撤回） | Accepted |
| [0019](0019-privacy-indicator-and-offline-mode.md) | プライバシー状態の可視化と「オフラインにする」導線 | Accepted |
