# ADR-0007: deep research の問い設計メソッドを標準化する

- Status: Accepted
- Date: 2026-06-16
- Deciders: Takenori Kusaka

## Context

本プロジェクトの初期 deep research（STT・競合・駆動開発）で、調査の「問いの立て方」が雑で、
表層的・要約的な結果に流れ、プロダクトの本質や意思決定から遠ざかる事象が起きた。
調査の質は、実行（検索・要約）よりも先に立てる**問いの質**で決まる。

一次情報調査により、良い問いの設計には確立した原典フレームワークが存在することを確認した
（Heilmeier Catechism / GQM / FINER / QFT / Socratic / リフレーミング / 5 Whys / First Principles /
MECE・イシュー / VOI / Popper / JTBD ほか）。

## Decision

**すべての deep research は、実行前に [問い設計メソッド](../research/question-framing-method.md) の
手順(A)で問いを設計し、チェックリスト(B)を通過させてから実行する。**

最低限、各調査ブリーフは次を満たすこと:

1. **ジョブ接続**: 「When … I want to … so I can …」でコア価値（[ADR-0004](0004-product-positioning-voice-journal.md)）に錨を打つ。
2. **意思決定駆動（VOI + Heilmeier #4）**: 各問いが「どの意思決定を変えるか／Who cares・what difference」に答える。変えないなら捨てる。
3. **反証可能性（Popper）**: 各問いに「何が分かれば考えが変わるか（disconfirming evidence）」を添える。
4. **構造化**: MECEなサブ問いへ分解し、各サブ問いに目的・出力形式・ソース・境界を明示する。
5. **原典主義**: 一次情報・提唱者本人の発信を最優先。安易な要約で薄めず、可能なら**原典をスナップショット保存**する。
6. **収束（GQM/FINER）**: 最重要3問に絞り、Goal→Question→Metric に落とし、FINERで吟味する。

## Consequences

- 調査ブリーフ（サブエージェントへのプロンプト）は、この6点を内包する形で書く。
- 調査成果は `docs/research/` に保存し、参照した**原典は可能な範囲で `docs/research/sources/` に原文保存**する（[ADR-0006](0006-scope-completeness-policy.md) のスコープ完全性と整合: 要約で削らない）。
- 行動規範として [CLAUDE.md](../../CLAUDE.md) にも要約を反映する。
- これ以降、過去の浅い調査（特に仕様駆動開発）はこのメソッドで再設計・再実行する。
