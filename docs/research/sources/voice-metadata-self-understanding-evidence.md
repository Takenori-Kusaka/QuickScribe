# 声由来メタデータは自己理解に寄与するか — 一次エビデンス

調査日: 2026-07-10（deep research: 105エージェント／採用23ソース／91クレーム抽出→25を3票制検証・23確定/2反証）
問い: QuickScribe（ローカル完結の思考整理・自己理解ボイスジャーナル）で、(A)話者特定と (B)声・プロソディ由来の感情/情動メタデータが、コア価値「思考整理・自己理解」に**体験として寄与するか**、および (B) の技術的信頼性。
関連ADR: [ADR-0030 感情非対応](../../adr/0030-no-voice-emotion-metadata.md) / [ADR-0031 話者特定オプション](../../adr/0031-speaker-diarization-optional-utility.md)
関連: `speaker-diarization-local.md`（実装可能性）, `transcription-metadata-design.md`, `whisper-metadata.md`

## 総括

エビデンスは QuickScribe の暫定判断（感情=非対応、話者特定=利用幅拡大の実用機能）を**概ね支持**した。

## D1土台: ジャーナリングの効き目の機序（確定 3-0）

- 効き目は catharsis/venting でも文字起こし精度でもなく、**一貫した物語の構築(meaning-making)＋自己距離化(self-distancing)**＝因果語・洞察語の増加に由来。既に整合的物語を持つ人は恩恵を受けない。
- → コア価値「ニュアンスを残しつつ思考を整理する整形の知性」を科学的に支持。
- ⚠️ **反証**: 大きな効果量（d=.47/.66）の主張は本検証で反証・削除。実際の効果量は小〜中、異質性大（I²≈98.5%の研究あり）。過大評価しないこと。
- 一次: Pennebaker & Chung (book chapter) https://c3po.media.mit.edu/wp-content/uploads/sites/45/2016/01/PennebakerChung_FriedmanChapter.pdf
- 一次: Park, Ayduk & Kross 2016, Emotion（2件の縦断RCTで自己距離化↑→情動反応性↓を媒介確認）https://sites.lsa.umich.edu/emotion-selfcontrol-psych/wp-content/uploads/sites/1322/2017/05/Stepping-back-to-move-forward.pdf
- 一次: https://pmc.ncbi.nlm.nih.gov/articles/PMC4345899/ / https://pmc.ncbi.nlm.nih.gov/articles/PMC10415981/

## D1: 話者特定と内省（確定 3-0）

- ソロの内省・自己理解に話者特定が寄与する直接エビデンスは一次文献に**皆無**。会議振り返り/臨床/字幕アクセシビリティの「誰が何を」ユーティリティとして価値が示されるのみ。diarization は話者を分離するだけで役割同定は別タスク。
- 設計補足（確定 3-0）: テキスト単独の話者/著者帰属はトピックを統制すると精度が急落（AUC 0.803→0.547 ほぼチャンス）＝見かけの識別力はトピック相関。**音響ベース diarization が必要**。
- 一次: CHI2025 AI-assisted reflection https://dl.acm.org/doi/10.1145/3706598.3714052
- 一次: SpeechCompass (CHI2025) https://arxiv.org/pdf/2502.08848
- 一次: 話者訂正=会議ユーティリティ https://arxiv.org/pdf/2509.18377
- 一次: 臨床話者役割同定 https://www.medrxiv.org/content/10.1101/2025.08.14.25332837.full.pdf
- 一次: 著者帰属のトピック交絡 TACL2024 https://direct.mit.edu/tacl/article/doi/10.1162/tacl_a_00678/123650/Can-Authorship-Attribution-Models-Distinguish

## D2: 感情ラベリングの効果と害（確定 3-0）

- **ベネフィット側**: affect labeling は扁桃体・辺縁系の負情動反応を低減、右腹外側PFC活動を増加（Lieberman et al. 2007, Psychological Science）。頑健な神経基盤。
- **害/条件依存側**: 一律に有益ではない。低強度刺激では逆に苦痛増（Levy-Gigi & Shamay-Tsoory 2022）。自己生成ラベルは遅延減少/一時増加、**意図的な自己関連ラベリングは晩期情動処理(LPP)と覚醒評定を増強**（Herbert et al. 2013）。神経的減衰は主観的に感じられないという乖離もある。
- → **機械が付けた断定的情動ラベルの提示は害のリスク**。D2は「見送りまたは慎重なopt-in」。
- ⚠️ 留保: 害の証拠は主にラボ刺激（画像/顔）。「機械ラベル≒提供ラベル」は合理的類推だが直接検証はされていない。
- 一次: Lieberman 2007 https://journals.sagepub.com/doi/10.1111/j.1467-9280.2007.01916.x
- 一次: 2024レビュー「ラベリングは常にヒーローではない」 https://www.sciencedirect.com/science/article/abs/pii/S007974212400001X
- 一次: 生成主体の差 https://pmc.ncbi.nlm.nih.gov/articles/PMC9799301/
- 一次: Herbert et al. 2013 ERP https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3719026/
- 一次: Torre & Lieberman 2018 レビュー https://pmc.ncbi.nlm.nih.gov/articles/PMC5866771/

## D3: SER の arousal vs valence（確定 3-0 / 一部 2-1）

- **arousal（覚醒）は音響/プロソディから回復可能、valence（感情価）は主に言語内容依存で音響単独では脆弱**。韻律中立な合成音声で valence 予測はセンチメント/否定辞に強く反応、arousal/dominance には言語特徴が影響しない。
- → 「感情を音響の断定的分類器としては非対応、arousal のみ非断定可視化」というD3判断は支持。
- ⚠️ 留保: 現代トランスフォーマは音声中の語彙を暗黙学習し valence を一定回復しうる（「音響で回復不能」は純プロソディに限り厳密に真）。実環境/自発発話/話者差/言語横断（特に日本語）の定量劣化は本バッチで裏取り不足。
- 一次: Triantafyllopoulos et al., Interspeech2022 https://arxiv.org/pdf/2204.00400
- 一次: Wagner et al., TPAMI2023（transformer が valence ギャップを埋める＝純音響では弱い） https://arxiv.org/pdf/2203.07378

## D4: 日本語プロソディと形式的言語（一次ソース収集済み・3票検証は未投入）

synthesis の要約は「未解決」としたが、fetch 段階で**強い一次音声学ソース**を取得済み（正式な3票検証には未投入のため「強く示唆／未確定」の扱い）。「申し訳ございません」問題（同一語が煽り/真摯の両方になる）を裏付ける:

- 日本語は同一文でも丁寧さを**文末F0・話速**で区別（Ofuka et al. 2000, Speech Communication） https://www.sciencedirect.com/science/article/abs/pii/S0167639300000091
- 会話日本語の formality は音響韻律（f0/強度/調音速度/ポーズ）に符号化（Cambridge JIPA） https://www.cambridge.org/core/journals/journal-of-the-international-phonetic-association/article/prosodic-properties-of-formality-in-conversational-japanese/0241CAF768C7978490278A1F0CE75BC9
- 丁寧語は遅い話速＋気息性の声質でマーク https://degruyter.com/view/journals/jjl/34/1/article-p127.xml?language=en
- プロソディ単独で話者の意図(speech act)を識別可（Journal of Memory and Language 2016） https://www.sciencedirect.com/science/article/abs/pii/S0749596X16000024
- テキスト単独では sincere/sarcastic を過小決定・プロソディが曖昧性解消に必須 https://arxiv.org/pdf/2606.09717

## Open Questions（将来の再検討ゲート）

1. D4を査読済み一次で正式確立（プロソディ→分類器でなくLLM入力チャネル設計の妥当性に直結）。
2. D3の実環境 arousal/valence 精度差を Interspeech/ICASSP 実環境ベンチで複数確認。
3. D2の実運用: 非断定的 arousal アークの本人提示が、断定ラベルと異なり歪曲を避けつつ気づきを高めるか。
4. D1逆側: 多声対話（治療/1on1/家族）の自己レビューで話者ラベルが内省を助けるHCI/臨床研究の有無。
