# LLMプロンプト評価・最適化フレームワーク — 一次エビデンス

調査日: 2026-07-10（deep research: 104エージェント／採用22ソース／98クレーム抽出→25を3票制検証・24確定/1反証）
問い: QuickScribe の文字起こし後生成AI処理（整形/用語補正/タグ/翻訳）のプロンプト・コンテキスト工学と、期待応答に対する精度・安定性を評価するフレームワーク。
関連: [ADR-0024 評価基盤(CER＋ニュアンス)](../../adr/0024-evaluation-redesign-cer-and-nuance.md) / `scripts/asr_eval/` / Zenn章 `evaluating-formatting-intelligence.md`

## D1: プロンプト最適化（DSPy / ACE）

**DSPy（確定 3-0）**: 「最適化文字列を抽出して別ランタイムで使う」用途は作者 Omar Khattab が公式に非推奨。「DSPy is a programming model, not an optimizer ... avoid managing strings」。署名＋入力＋デモを実行時にアダプタが LM 依存で描画する結合設計で、静的抽出は挙動を忠実再現しない。
- ただし MIPROv2/COPRO/BootstrapFewShot が命令とデモを最適化（program+metric+少量train）。結果は人間可読な plain-text JSON で凍結・再ロード可。→ **「dev-timeで最適化→JSONから命令とデモを手動抽出→各プロバイダへ移植」なら成立（ドロップイン不可）**。JSONはバージョン敏感(dspy>=3.0後方互換)。
- 一次: https://github.com/stanfordnlp/dspy/issues/8043 / https://dspy.ai/api/optimizers/MIPROv2/ / https://dspy.ai/faqs/ / https://github.com/stanfordnlp/dspy/blob/main/docs/docs/learn/optimization/optimizers.md
- ⚠️反証: issue#201で「save()で移植可」という主張は 1-2 で反証（save は状態のみ・ロジック非含）。

**ACE (Agentic Context Engineering)（確定 3-0）**: arXiv:2510.04618（Stanford/SambaNova/Berkeley, 2025-10, ICLR2026）。コンテキストを「進化するplaybook」として Generator/Reflector/Curator の増分デルタ更新（helpful/harmfulカウンタ・決定的マージ）。offline(システムプロンプト)＋online(エージェントメモリ)双方。
- **QuickScribeコア価値への直撃**: 反復全書き換えの2失敗を名指し — **brevity bias**(要約で領域洞察が脱落)・**context collapse**(反復書き換えで詳細が浸食)。AppWorld例: 18282→122トークンに崩壊、精度66.7%→57.1%。「要約せずニュアンスを残す」への警告。
- 効果: MIPROv2/GEPAを +8.6%(finance)/+10.6%(agent)/+10.9%(offline平均) 上回る。⚠️**著者ベンチ・第三者再現なし・offline優位はground-truthラベル使用・信頼フィードバック無しだと劣化/有害化**。主観品質タスクは明確な実行信号が無く直接適用は未実証。
- 一次: https://arxiv.org/abs/2510.04618 / https://github.com/ace-agent/ace

## D2-D4: 評価妥当性（LLM-as-judge / 較正）

**LLM-judge の80%一致の裏側（確定 3-0）**: Zheng et al. MT-Bench/Chatbot Arena（arXiv:2306.05685, 2023）。強い判定器(GPT-4)は人手と80%超一致＝人間同士と同水準。**ただしタイ除外値**。タイ・順序不一致込みでは約66%に低下。位置バイアス(順序入替で一貫66%)・冗長バイアス(内容増無しでも長答を90%超選好)・自己贔屓を実測。一般対話での測定＝整形品質タスクへは新規較正なしに一般化不可。
- 一次: https://arxiv.org/pdf/2306.05685

**相関でなくKappaで較正（確定 3-0）**: "The Judge's Verdict"（arXiv:2510.09738, NVIDIA, 2025-10）。相関だけでは不十分（r=1.0でも系統的に甘/辛でありうる）。例: r=0.95でもκ=0.45・z=−15.2で人間らしさ不合格。**二段検証ハーネス**: Step1 相関足切り(Pearson r≥0.80)→Step2 人手3人に対するκをz化(人間同士κ=0.801基準、|z|<1で人間らしい・z>1で超一貫)。判定力はモデルサイズでなく学習戦略に依存（54中27がTier1）。→ 判定器は大モデルを盲信せず二段テストで実測選定。
- 一次: https://arxiv.org/pdf/2510.09738

## D5: QuickScribeへの含意

- 既存三層評価(ADR-0024)に接続: 自動(CER/BERTScore/NLI/圧縮率・`scripts/asr_eval`のbootstrap_ci再利用)＋判定(ペアA/B・順序入替・複数合議・判定器と整形器の分離)＋人手(黄金セット)。
- DSPy: dev-time最適化→JSON凍結→手動移植＋強い評価ループ（条件付き採用）。
- runtime自己進化: ACEのデルタ更新思想は参考、主観タスクではフィードバック代理を人手ゲートで補完。自動進化はニュアンス破壊(brevity/collapse)に注意。
- 判定器: 必ず Cohen's Kappa＋z-score較正。スコア単独で意思決定せず人手スポットゲート併用＝PDCA。dev-timeハーネスに閉じ実行時外部送信を増やさない。

## 未解決（openQuestions）

1. DSPy最適化プロンプトのマルチプロバイダ手動移植後の品質劣化/保持（一次実測なし）。
2. ACEの増分更新・collapse回避思想を、明確な実行信号が無い主観整形タスクにどう適用するか（フィードバック代理＝人手/自己一貫性/参照スコア）。
3. Judge's Verdict二段ハーネスの閾値(r≥0.80/κ=0.801)が整形ルーブリック評価に妥当か＝独自人手アノテーションで再較正要。
4. 少数n(dev-timeベンチは小規模)での判定スコアの信頼区間・分散・pass^k安定性・温度/シード/モデル差頑健性の具体手順。

## caveat

ACE・Judge's Verdict は2025-10の新しいプレプリントで第三者独立再現は未確認。**スコアは単独で意思決定に使わず、必ず Cohen's Kappa＋z-score較正と人手スポットゲートを併用する**こと。
