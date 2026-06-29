# 整形出力言語・翻訳オプションの設計調査

> 作成: 2026-06-30 / 関連: #401(i18n) / コア価値=「ニュアンスを残しつつ思考を整理する整形の知性」
> 問い設計: [question-framing-method](question-framing-method.md)（ADR-0007）に従い、ジョブ接続・意思決定駆動・反証可能性・MECE分解で設計。
> 方法: deep research（22ソース・89クレーム抽出 → 25クレームを3票敵対的検証 → 22確証/3棄却）。

## 1. 調査の目的（意思決定駆動）

「整形（AIによる思考整理）時に、出力テキストを指定言語へ翻訳するか」のオプション追加にあたり、実装前に次の決定を裏付ける:

1. 出力言語を **UI言語に追従** させるか **専用の出力言語設定** を別に持つか。
2. 翻訳のデフォルト挙動（既定ON/OFF・デフォルト言語）と UX（翻訳ONチェックボックス→言語ピッカー出現）の是非。
3. **ニュアンス保持（コア価値）と翻訳の原理的対立** への対処（整形＋翻訳を1パスか2パスか／原文保持／システムプロンプトへの言語指示）。
4. 用語・固有名詞・コードスイッチング（混在発話）の扱い。

## 2. 競合横断比較

| 製品 | 出力言語の翻訳 | 文字起こし言語と出力言語の分離 | UI言語と出力言語の分離 | 既定値 | 出典 |
|---|---|---|---|---|---|
| **AudioPen** | あり（58言語聞取→100+言語出力） | あり（出力言語は専用設定） | — | 既定値は出典から未確定 | [faq](https://www.audiopen.ai/faq) |
| **Granola** | あり（要約言語） | あり | あり（UIは英語固定／要約言語のみ可変） | `English`強制 or `Auto`=多数派言語 | [multi-language](https://docs.granola.ai/help-center/customising-granola/multi-language) |
| **Otter.ai** | あり（Otter Chat 経由＝事後・別経路） | あり（翻訳は文字起こしパイプと別） | あり（UIは英語固定） | 文字起こしは一度に1言語 | [help](https://help.otter.ai/hc/en-us/articles/26660468516631-Transcribe-conversations-in-English-Spanish-French-German-Japanese-or-Chinese-Simplified) |
| **Audionotes** | あり（AI生成物の出力言語） | あり（言語設定変更は認識調整のみ・既存内容は翻訳しない） | — | Settings既定 or 生成時セレクタ | [supported-languages](https://www.audionotes.app/supported-languages) |

**観察**: 主要競合は一様に「文字起こし言語」と「AI出力（整形/要約）言語」を**別設定として分離**。UI言語とコンテンツ出力言語も混同せず、Granola/Otter は UI を英語固定にしたまま出力言語のみ可変にしている。

## 3. 検証済みの知見（信頼度付き）

### A. 当初案を支持する findings
- **【high】専用の出力言語設定は定石。** 競合（AudioPen/Granola/Audionotes/Otter）が一様に文字起こし言語と出力言語を分離。→ QuickScribe の「専用出力言語設定」は妥当。
- **【high】UI言語とコンテンツ出力言語は分離すべき。** Granola「UI は英語のみ／要約言語は別途設定」、Otter「選択言語が影響するのは文字起こし処理のみ・UI は英語のまま」。
- **【high】「チェックボックスで有効化→言語ピッカー出現」は確立パターン。** Microsoft UX ガイド（progressive disclosure：前提完了後に追従コントロールを表示）と Nielsen Norman Group が正当化。翻訳は一部ユーザーのみ必要なオプションなので既定OFFで隠し、有効化時に言語選択を露出させる設計に合致。
  - 出典: [MS](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-progressive-disclosure-controls) / [NN/g](https://www.nngroup.com/articles/progressive-disclosure/)

### B. 当初案の**再考**を促す findings（重要）
- **【high】整形と翻訳の「同一1パス」は品質リスク。** クロスリンガル要約の研究では、整形と翻訳を分離した**多段（2パス）パイプライン**が単一パスを上回る傾向。良質なMTがあれば end-to-end（同一パス）の優位は消える。
  - 出典: [arXiv 2409.00414](https://arxiv.org/html/2409.00414v1)（separating translation and summarization）/ [arXiv 2410.20021](https://arxiv.org/pdf/2410.20021)（多段 SITR が単一パスを ROUGE で 98–103% 上回る）
- **【high】最先端LLMでも比喩・言葉遊び・文化固有表現は頻繁に誤訳。** データ量の多い高リソース言語でも figurative expressions/wordplay を誤訳し、相当な人手修正を要する。プロンプト感度・ハルシネーションのリスクも。→ **ニュアンス保持をコア価値とする QuickScribe で翻訳を雑に1パスで委ねるのは原理的リスク。**
  - 出典: [arXiv 2509.21577](https://arxiv.org/pdf/2509.21577) / [arXiv 2305.14328](https://ar5iv.labs.arxiv.org/html/2305.14328)
- **【medium】システムプロンプトはターゲット（出力）言語で書く方が文化固有翻訳タスクで高性能**な兆候（出力言語の指示の差し込み方に示唆）。ただしタスク性質が異なるため限定適用。

### C. プライバシー / ローカル制約
- **【high】Whisper の `task=translate` はターゲットが英語固定。** 任意言語への出力は whisper 単体では不可 → 別途の翻訳エンジン（ローカルLLM or クラウドLLM）が必須。
  - 出典: [whisper README](https://github.com/openai/whisper) / [whisper-large-v3 card](https://huggingface.co/openai/whisper-large-v3)
- 含意: ローカル完結（[product-positioning](competitive-landscape.md) のコア差別化）を保つには英語以外の出力にローカルLLM翻訳が要るが品質/サイズの現実的制約あり。**クラウドLLM翻訳を足すならオプトイン必須**（既存のクラウドSTT/整形の扱いと一貫）。

### D. コードスイッチング（混在発話）
- **【high】製品・研究とも未対応が多い。** Otter は一度に1言語のみ（例外: 仏+英）。混在発話は多言語ユーザーで一般的だが「largely overlooked」。→ QuickScribe で扱うなら**明示設計**が必要（主要言語へ正規化 vs 原文混在保持をユーザー意図でどう決めるか）。
  - 出典: [Otter help](https://help.otter.ai/hc/en-us/articles/26660468516631-Transcribe-conversations-in-English-Spanish-French-German-Japanese-or-Chinese-Simplified) / [CroCoSum arXiv 2303.04092](https://arxiv.org/pdf/2303.04092)

## 4. 棄却された主張（敵対的検証で false）

- 「translate-then-summarize は十分品質のMTがあれば end-to-end を39言語で**一貫して**上回る」→ **1-2で棄却**（"consistently" は過剰一般化。方向性は支持されるが普遍的でない）。
- 「最適戦略はタスク枠組みのみ翻訳し生成は英語で行うハイブリッド」→ **0-3で棄却**（QuickScribe は出力が指定言語のテキストを要求するため不適用）。
- 「コードスイッチング入力は LLM 理解を明確に低下させる」→ **1-2で棄却**（劣化は言語ペア×モデル固有で単調でない。一律のリスクとは言えない）。

## 5. QuickScribe への推奨

コア価値（ニュアンス保持）・ローカルプライバシーと整合する形:

1. **専用の「整形出力言語」設定を持つ**（UI言語とは別。当初案どおり・定石と一致）。
2. **翻訳ONチェックボックス → 言語ピッカー出現**（progressive disclosure・当初案どおり）。既定OFF。
3. **デフォルト出力言語 = UI言語**（妥当だが、競合の既定値は未確定＝§6。実装時に最終判断）。
4. **整形と翻訳は2パス推奨に傾く（要再検証）。** 「①原語で整形（ニュアンス保持を最優先）→ ②整形済みテキストを翻訳」。理由: 同一1パスは比喩・文化固有表現の誤訳リスクが高く、コア価値と衝突。
   - ただし学術エビデンスは短文要約/低リソース言語が中心で **ja↔en 高リソース・長文への外挿は未検証**。実装前に ja↔en で1パス vs 2パスの小規模比較（スパイク）を行う価値がある。
5. **原文（整形済み原語テキスト）は常に保持・保存。** 翻訳はあくまで派生物として扱い、後から原語でニュアンスを検証可能にする（[ADR-0006](../adr/0006-scope-completeness-policy.md) のスコープ規律＝削るのでなく段階実装で対処）。
6. **クラウドLLM翻訳はオプトイン必須**。ローカルLLM翻訳の品質が ja↔en で実用水準かは別途検証（§6）。

## 6. 未解決の論点（実装前に詰める）

- 競合の**既定値**（翻訳デフォルトON/OFF・デフォルト言語をUI追従か前回値か）は出典から確定できず。実機/最新スクショで再確認。
- **ローカルLLMの ja↔en 翻訳品質**が実用水準か（モデルサイズ・速度・品質のトレードオフ）。達しない場合、ローカル完結訴求とクラウド翻訳オプトインの線引き。
- **2パス時の簡便性との両立**（「リッチすぎると簡便でなくなる」核心課題）。原文併記/保存をUIでどう最小表現するか。
- **コードスイッチング**を主要言語へ正規化するか原文混在を保持するか、ユーザー意図の推定/設定方法。

## 7. 反証可能性（当初案を覆す条件）

- 「専用出力言語設定」案 → 競合が実は UI言語追従に倒している実機証拠が出れば再考。
- 「2パス推奨」案 → ja↔en 長文で1パスがニュアンス保持・品質で同等以上というスパイク結果が出れば1パスを採用。
- 「既定OFF」案 → 想定ユーザーの大半が翻訳を常用するなら、隠さず常時表示に倒す。

---

### 出典一覧（一次情報優先）

**競合（一次）**: AudioPen FAQ / Granola docs / Otter help / Audionotes — 上記リンク参照。
**UXパターン**: [MS progressive disclosure](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-progressive-disclosure-controls)（primary）/ [NN/g](https://www.nngroup.com/articles/progressive-disclosure/)（secondary）。
**LLM整形×翻訳（学術・一次）**: [2409.00414](https://arxiv.org/html/2409.00414v1) / [2410.20021](https://arxiv.org/pdf/2410.20021) / [2509.21577](https://arxiv.org/pdf/2509.21577) / [2305.14328](https://ar5iv.labs.arxiv.org/html/2305.14328) / [2507.22923](https://arxiv.org/html/2507.22923) / [2303.04092](https://arxiv.org/pdf/2303.04092) / [2506.14012](https://arxiv.org/pdf/2506.14012)。
**ローカル/プライバシー**: [whisper](https://github.com/openai/whisper) / [whisper-large-v3](https://huggingface.co/openai/whisper-large-v3) / [local-first(Ink & Switch)](https://www.inkandswitch.com/essay/local-first/)。
