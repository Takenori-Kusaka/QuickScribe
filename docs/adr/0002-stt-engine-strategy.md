# ADR-0002: 文字起こし(STT)エンジン戦略

- Status: Accepted
- Date: 2026-06-15
- Deciders: Takenori Kusaka

## Context

STT は本プロダクトのコア価値（整形の知性）ではなく、差し替え可能な入力。2026年時点の
deep research（実在デスクトップアプリ8本の採用調査、独立WERベンチ、ライセンス/ToS精査）に基づき決定する。

## Decision

- **既定エンジン: ローカル `whisper.cpp` + `large-v3-turbo`（量子化）**。
  単一バイナリ・Win/Linux・CPU実用・MIT・日本語は large-v2 同等。8本中5本が whisper.cpp 採用＝デファクト。
- **日本語特化要件には `kotoba-whisper-v2.0`（Apache 2.0）** を同ランタイム上で。日本語CERで large-v3 を上回る。
- **クラウドはプラグイン化**（鍵を入れた人だけ）: Groq turbo（$0.04/時）/ Deepgram（低遅延・オンプレ可）/ Azure（完全オフラインコンテナ）。
- エンジンは Strategy パターンで抽象化し、ユーザーが切替可能にする。

## Alternatives considered

- **Parakeet / Canary (NVIDIA)**: 英語精度世界最高だが日本語非対応 → 不採用。
- **Moonshine**: 低遅延だが日本語(非英語)モデルが非商用ライセンス → 日本語商用利用不可。
- **Vosk**: 軽量CPU・Apache2.0だが精度が Whisper 系に劣る → 低スペック向けオプション候補に留める。

## Consequences

- 完全オフライン・ToSフリーで動く既定体験を提供でき、プライバシー約束（[vision](../vision.md)）と整合。
- 採用前に JSUT/CSJ/Common Voice ja での日本語自前ベンチが必須（各社日本語WER/CER非公開）。
- 量子化モデルの同梱でインストーラサイズが増える（配布戦略で扱う）。
