# ADR-0003: Google Docs ボイスタイピング自動駆動を採用しない

- Status: Accepted
- Date: 2026-06-15
- Deciders: Takenori Kusaka
- Relates to: [ADR-0002](0002-stt-engine-strategy.md)

## Context

当初構想では「梱包ブラウザで Google Docs の音声入力を自動駆動して文字起こしする」案があった。
無料・高品質という期待があったため、deep research で実現可能性とリスクを検証した。

## Decision

**採用しない。**

## 根拠

1. **精度優位の客観的根拠なし**: 独立ベンチで gpt-4o-transcribe ≈2.5% WER に対し Google Chirp2 ≈11.6%。
   Docs は旧世代 Web Speech API ベースで、製品自体のWERを測った査読データも存在しない。
2. **ToS違反リスク**: Google 利用規約は自動アクセス・スクレイピング・保護機構の回避を明示禁止。
3. **技術的にブロックされる**: Puppeteer/Selenium の自動ログインは "browser may not be secure" で弾かれ、
   `navigator.webdriver` 検出により回避は持続しない。
4. **機能的障壁**: ライブマイク必須・ファイル流し込み不可・公式API不在。

## Consequences

- 梱包ブラウザ＋ブラウザ自動化という最大の保守コスト・脆弱性源が消える。
- 「ブラウザを梱包し既存ブラウザに依存しない」要件は、STTではなく **LLM整形のBYO認証**（既存ブラウザでGemini/Claude認証）に限定して残る。
- システム音声取り込みは仮想オーディオデバイス不要（Win=WASAPIループバック / Linux=PipeWire monitor）。
