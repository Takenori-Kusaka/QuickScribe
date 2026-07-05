# ADR-0024: 評価基盤の再設計（日本語CER刷新＋ニュアンス保持の客観計測）

- Status: Proposed（設計スパイクの成果。Deciders 承認で Accepted → 実装へ）
- Date: 2026-07-05
- Deciders: Takenori Kusaka
- 関連: [ADR-0007 調査規律](0007-research-question-framing-method.md) / [ADR-0022 モデルカタログ精選](0022-model-catalog-curation.md) / [ADR-0004 コア価値]
- 対象 Issue: #578（日本語CER評価の刷新）/ #577（ニュアンス保持整形の客観計測）/ #403（既存CERベンチ）
- 一次情報: 本ADR末尾の「一次情報」節（コーパスのライセンス/入手性、指標の日本語妥当性を一次情報で確認済み）

## 背景・課題

現状の日本語精度評価（`perf.yml` の `ja-accuracy` ジョブ）は次の弱点を持つ。連載記事（#576）の「正直な弱点」でも明示した。

1. **N=3・本人音読のパブリックドメイン作品**で測っている。話者1名＝話者分散ゼロ、音読は実利用の自発的独白と音響・言語特性が乖離。統計的にも代表性的にも脆弱。
2. **相対/回帰指標のみ**で、公開ベンチの絶対CERが無く他STTと比較できない。
3. コア価値「要約せずニュアンスを残す整形」に**客観指標が無く**、第三者に「本当に要約と違うのか」を数値で示せない。

「サンプルの作り方や評価方法自体が不適切では」という批判に、今の評価は耐えられない。評価方法から刷新する。

なお、現状が自前録音を選んだのは**第三者音声をリポジトリに同梱・再配布するリスクを避ける**ためだった（fixtures/README）。本ADRの方針（公開コーパスを**CI実行時にDLし、成果物/リポジトリには同梱しない**）はこの原則と両立する。

## 決定

### 1. 日本語CER評価（#578）

- **コーパスは公開・ローカルDL可のものを CI 実行時に取得し、リポジトリには同梱しない**（ライセンス遵守＋リポジトリ肥大回避）。
  - **主軸: Common Voice ja（CC0）**。ライセンスが完全にクリーンで CI 無人自動化の障害が最小。多話者（validated）で実利用に近いCERが測れる。公式配布のアカウント化を避け **CC0 再ホストの HF ミラー（版数固定＋Actionsキャッシュ）** から `streaming` で数百発話を取得する。
  - **回帰ベースライン: JSUT basic5000**（第二スライス）。単一話者・クリーン読み上げで安定。**非商用・再配布禁止**のため、音声を成果物/リポジトリに同梱せず CI 実行時DLに限定する運用を実装で担保する。
  - **軽量スモーク: FLEURS(ja)（CC-BY-4.0）**。完全自動DLで定点観測に使う。
  - **自発独話の代表性補完: TEDxJP-10K**（人手検証済み・独話）。ただし音源がYouTube由来でCIのDLが壊れやすいため、**初回DLをキャッシュ/成果物保存**する運用が前提（採否は第三スライスで判断）。
- **正規化パイプライン（参照文・仮説文に同一適用）**: 括弧内注記除去 → `neologdn`（長音/ダッシュ/繰り返し/半角カナ正規化）→ NFKC → ラテン小文字化 → 約物・記号除去 → **空白完全除去（日本語CERの必須事項）** → 文字単位。CER計算は `jiwer`。
  - **数字表記・かな種別の吸収は既定OFF**。吸収するとCERは下がるが**モデルの実出力差を隠す**ため。コア価値「実差を消さない」とも整合。
- **統計**: N=数百発話・複数コーパスで**層別CER**（話者/様式）。**ブートストラップ95%CI**（発話単位リサンプリング）を算出しCIログに記録。2システム比較は matched-pairs。**CIが重なる差は「差なし」**と扱う。
- **フィラー保持率を独立計測**。Whisperはフィラー・言い淀みを削除する傾向があり、ニュアンス保持を掲げる本プロダクトでは減点対象。CER前に一律除去するとこの失敗を隠すため、別指標として出す。
- **既存の自前録音fixtureは削除しない**（[ADR-0006] 独断縮小の禁止）。録音形式(Opus)そのままのdogfooding smokeとして残し、公開コーパスの統計的・比較的ベンチを**加える**。

### 2. ニュアンス保持整形の客観計測（#577）

自動指標は「意味が保たれたか」までで、「迷い・トーン・含意まで残る＝要約と違う」は自動のみでは測れない。**自動層＋判定層＋人手層の三層**で段階導入する。

- **自動層（CI・ローカル完結）**:
  - **意味保持 = BERTScore-ja**。まず導入コスト最小の多言語BERT（`bert-base-multilingual-cased`・MeCab不要）で開始し、日本語特化精度が実測で必要と判明した時点で tohoku 系（`tohoku-nlp/bert-base-japanese-v3`）へ差し替える。
  - **faithfulness/含意 = 日本語NLIの entailment 確率**（原文→整形後方向で「原文の情報が整形版に残っているか＝要約で捨てていないか」を測る）。**AlignScore は英語専用のため不採用**、日本語NLIモデル（`akiFQC/...jnli...` または依存最小の多言語 `mDeBERTa` NLI）で代替する。
  - **圧縮率**（文字数比・内容語保存率・フィラー保存率）。ニュアンス保持なら1.0近傍という反証可能な仮説。
  - 全モデルを事前DL・キャッシュし `TRANSFORMERS_OFFLINE=1` でオフライン実行。**依存最小化のため多言語モデル（mBERT/mDeBERTa）で統一**して開始する。
- **判定層（リリース前/週次）**: **ペア弁別方式**。同一音声から「①ニュアンス保持整形版」と「②要約版」を生成し、含意・迷い・トーンをどちらが保つかをA/B判定。LLM-as-a-judge はバイアス（位置・冗長・自己選好）較正必須＝順序入替え・複数合議・整形モデルと判定モデルの分離・人手Likertとの相関（κ/Spearman）を添える。
- **人手層（マイルストーン毎）**: 多様な独白音声20–50本に「保持すべきニュアンス要素」をアノテートした**黄金評価セット**をローカル保存し、自動指標/LLM判定の較正に使う。

### 3. スパイクの本体＝指標の自前妥当性検証

BERTScore-ja／日本語NLIとも、**日本語での人手相関を保証する一次論文は本調査では未確認**。よって第一スライスでは**少数の人手アノテーションとの相関を並行取得**し、指標の妥当性を自前で裏取りする。これを省くと「測っているつもりで測れていない」に陥る。

## 段階実装（[ADR-0006]: 最終ゴール不変・削らず分割）

- **第一スライス（v1.1.0）**: #578＝Common Voice ja（CC0ミラー）＋正規化パイプライン＋jiwer CER＋ブートストラップCIをCIに。既存fixtureはsmokeとして残す。#577＝BERTScore-ja＋日本語NLIの自動層をローカルで試作し、少数人手相関で妥当性を裏取り（スパイク）。
- **第二スライス（v1.2）**: JSUT basic5000（非同梱DL遵守を実装で担保）を回帰ベースラインに追加。フィラー保持率を独立指標化。#577の自動層をCIへ。
- **第三スライス（v1.3）**: 黄金評価セット構築→ペアA/Bで「整形版はニュアンス保持でN本中M本、要約版に勝つ（有意）」を対外提示。TEDxJP-10K（キャッシュ運用）採否判断。

## 結果・トレードオフ

- **Pro**: 公開ベンチで他STTと比較可能な絶対CER＋信頼区間を持てる。コア価値の主張（ニュアンス保持≠要約）を反証可能な形で検証できる。すべてローカル完結・ライセンスクリーン。
- **Con/リスク**: CI時間・依存（Python評価スタック・HFモデルDL）が増える。CC0ミラーは非公式のため版固定＋キャッシュ＋`trust_remote_code`のサプライチェーン注意が必要。指標の日本語妥当性は自前検証コスト（人手相関）を伴う。
- **却下案**:
  - **CSJ（自発独話の学術標準）** → 有償・利用許諾契約・USB物理郵送でCI自動取得が原理的に不可能。手元購入の研究用途に限定。
  - **AlignScore** → 英語専用・日本語検証なし。日本語NLIで代替。
  - **BLEU系** → 人手相関が低く不採用（BERTScoreを主軸）。
  - **数字/かな吸収を既定ON** → モデルの実出力差を隠すためOFF（ニュアンス保持思想と不整合）。

## 「考えが変わる条件」（反証）

- CC0ミラーが消える/信頼できないと判明 → 公式Common Voice（要アカウント）か JSUT へ主軸を移す。
- 多言語BERT/NLIが日本語で人手と相関しないと自前検証で判明 → tohoku系/日本語特化NLIへ差し替え、それでも駄目なら自動層を判定層（LLM-judge）中心へ再設計。
- CI時間が許容を超える → コーパスをサブサンプルし、フル評価は週次dispatchへ分離。

## 一次情報

- コーパス: [Common Voice CC0ミラー fsicoli](https://huggingface.co/datasets/fsicoli/common_voice_17_0) / [CV公式移行告知](https://huggingface.co/datasets/mozilla-foundation/common_voice_17_0) / [JSUT Terms](https://sites.google.com/site/shinnosuketakamichi/publication/jsut) / [FLEURS](https://huggingface.co/datasets/google/fleurs) / [ReazonSpeech](https://huggingface.co/datasets/reazon-research/reazonspeech) / [CSJ入手](https://clrd.ninjal.ac.jp/csj/en/subscription.html) / [TEDxJP-10K](https://github.com/laboroai/TEDxJP-10K)
- 指標: [bert_score utils.py（lang2model実物）](https://github.com/Tiiiger/bert_score/blob/master/bert_score/utils.py) / [tohoku BERT v3](https://huggingface.co/tohoku-nlp/bert-base-japanese-v3) / [AlignScore（英語専用）](https://github.com/yuh-zha/AlignScore) / [akiFQC 日本語NLI](https://huggingface.co/akiFQC/bert-base-japanese-v3_nli-jsnli-jnli-jsick) / [mDeBERTa 多言語NLI](https://huggingface.co/MoritzLaurer/mDeBERTa-v3-base-xnli-multilingual-nli-2mil7)
- 正規化: [Whisper normalizers/basic.py](https://github.com/openai/whisper/blob/main/whisper/normalizers/basic.py) / [neologdn](https://github.com/ikegami-yukino/neologdn) / [jiwer transforms](https://jitsi.github.io/jiwer/reference/transforms/)
- 統計/手法: Blockwise Bootstrap (arXiv 1912.09508) / TST評価メタ評価 (arXiv 2502.04718) / AlignScore (arXiv 2305.16739) / G-Eval (arXiv 2303.16634) / Whisperのフィラー除去 (arXiv 2509.20655)
