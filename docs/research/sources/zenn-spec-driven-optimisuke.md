# 原典スナップショット: 仕様駆動の本来の定義となぜ今か（Zenn / optimisuke）

- 著者: optimisuke
- 正典URL: https://zenn.dev/optimisuke/articles/090949f0487326

## 主旨（要約 + 鍵となる短い引用）
仕様駆動を Needs → Requirements → Requirements Specification → Design → Implementation のレイヤで捉え、
**Specification（仕様）と Design（設計）を分けて考える**ことの重要性を説く。

- Requirement: 満たすべき条件そのもの（1つ1つの文）
- Specification: 条件を体系化・明文化した成果物
- Design: その条件をどのように実現するか

「なぜ今か」の論点（AIエージェント時代）:
- 従来のアジャイルは「密でハイコンテキストなコミュニケーション」に依存していたが、AIは「暗黙知に依存できない、非同期・分業が前提になる」。
- ゆえに「何が前提で、何が自由なのか」を明文化する必要がある。
- 著者の整理（短い引用）: 「Spec と Design を分けることは、設計を縛るためではなく、設計を自由にするための整理」。
- 仕様駆動は「ウォーターフォールへの回帰」ではなく「暗黙知依存からの脱却」。

## 参照規格・ツール
- JIS X 0166:2014（ISO/IEC/IEEE 29148 に対応）、EARS の IEEE論文。
- AWS Kiro（Specs Concepts, https://kiro.dev/docs/specs/）。EARS形式: 「While <前提>, when <トリガー>, the <システム> shall <応答>」。

## QuickScribeでの含意
本プロジェクトは単独開発者＋AIエージェントの並列開発であり、この「暗黙知に頼らず仕様で本質を共有する」論点が
そのまま適用される。要件と設計を分離する書式を仕様/issueテンプレに採るか（Q1の意思決定）の一次根拠。
