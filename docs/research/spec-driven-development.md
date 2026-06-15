# 仕様駆動開発(SDD) 深掘りリサーチ — 本来の定義・なぜ今か・我々の運用

> Status: Reference (2026-06-16) — [ADR-0007](../adr/0007-research-question-framing-method.md) の問い設計メソッドに従い、3つの中核問い（意思決定含意＋反証条件付き）で再設計した調査。
> 原典は [sources/](sources/) に忠実スナップショット保存。逐語引用は短く出典明記。

設計した中核問い:
- **Q1**: Spec(仕様)とDesign(設計)を分離する"本来の定義"は何か。なぜ今(AI時代)再評価されるか。
- **Q2**: Spec Kit / Kiro / BDD の3系統は、"探索的でドメインが曖昧な"思考整理ボイスジャーナルにどう適合/不適合か。
- **Q3**: 既採用のTDD(t-wada)/DDDと、仕様駆動を形骸化させず接続する具体手順は。

---

## Q1. 本来の定義と「なぜ今か」

### 本来の定義: 仕様 ≠ 設計
要求工学の標準（JIS X 0166:2014 = ISO/IEC/IEEE 29148）に沿い、**Needs → Requirements → Requirements Specification → Design → Implementation** の階層で捉える（[Zennスナップショット](sources/zenn-spec-driven-optimisuke.md)）。

- Requirement = 満たすべき条件そのもの（1文）
- Specification = 条件を体系化・明文化した成果物
- Design = その条件を**どう実現するか**

著者Naosuke氏の核心（短い逐語）: 「**Specification（仕様）は Design（設計）ではありません。**」。
EARSは「個々の要求を明確に書く道具であって、仕様そのものではない」と位置づけられる（[EARSスナップショット](sources/ears-syntax.md)）。

**これは著者の主観でなく国際標準が明文で規定している**（[ISO 29148スナップショット](sources/iso-29148.md)）。ISO/IEC/IEEE 29148 Clause 5.2.7（短い逐語）:
> "Requirements should state 'what' is needed, not 'how'."

29148は階層 Needs → Stakeholder Requirements(StRS) → System Requirements(SyRS) → Software Requirements(SRS) → Design を定義し、**JIS X 0166:2014 はその一致規格(IDT)**。要件と設計の分離は要求工学の標準的規範である。

GitHub Spec Kit も同じ思想を英語側から裏付ける（短い逐語）:
- "specifications define the **'what'** before the **'how'**"
- "specifications become executable"（仕様が実行可能になり実装を直接生成）
- "We're moving from 'code is the source of truth' to **'intent is the source of truth.'**"
（[Spec Kitスナップショット](sources/github-spec-kit.md)、GitHub Blog: Den Delimarsky）

### なぜ今か: AIエージェント時代の「暗黙知が通じない」
最も凝縮した一次逐語（Zenn, Naosuke）:
> 「AI Agent を『優秀なジュニアエンジニア』のように、並列に大量に働かせようとすると、**暗黙知に依存できない、非同期・分業が前提になる**という状況になります。」

英語側の裏付け（GitHub Blog, Den Delimarsky、短い逐語）:
> "They're exceptional at pattern completion, but **not at mind reading.**"

日本語の補強（Qiita, @ju-kosaka、短い逐語）: 「**人間同士なら会話で補完できた暗黙知が、AIには通じません。**」。同記事タイトルは「仕様駆動開発は、ウォーターフォールへの回帰ではない。」。

→ 従来アジャイルは「密でハイコンテキストな会話」で暗黙知を補完していた。AIエージェントを並列・非同期に働かせる世界では、その前提が崩れ、**何が前提で何が自由かを明文化する**必要が生じる。これがSDD再評価の本質。「設計を縛るためでなく、設計を自由にするための整理」（Zenn）。

### 意思決定への含意（Q1）
**要件(requirements)と設計(design)を分離する書式**を、仕様/issueテンプレに採る。受入基準はEARSで書く。
→ 既存のIssueテンプレ（Story/Task）に「要件(EARS受入基準)」と「設計」を別セクションで持たせる改訂を行う。

### 反証条件の検証（Q1）
「原典が分離でなく一体記述を推奨していれば再考」→ **反証されず**。標準(29148)・Zenn・Spec Kit・Kiroいずれも仕様と設計の分離（whatをhowより先に）を支持。**分離方針を採用してよい。**

---

## Q2. 3系統の適合性（探索的・ドメイン曖昧なプロダクトに対して）

| 系統 | 定義/ワークフロー | 重さ | 探索的プロダクトへの適合 |
|---|---|---|---|
| **GitHub Spec Kit** | Constitution→Specify→Clarify→Plan→Tasks→Analyze→Implement。intent-driven、仕様が実行可能（[snapshot](sources/github-spec-kit.md)） | 重い | フェーズが多くガードレール厚め。要件が固まらない初期は Specify/Clarify の往復が増えやすい |
| **AWS Kiro** | Prompt→Requirements(EARS)→Design→Tasks→Code。requirements.md/design.md/tasks.md（[snapshot](sources/kiro-specs.md)） | 中〜重 | 3ファイル分離が明快。neuro-symbolicな形式化は曖昧さ検出に有効だが、探索段階では過剰になりうる |
| **BDD/ATDD（古典）** | Given-When-Then の具体例で振る舞いを合意（[driven-development.md](driven-development.md) §2.1）。Specification by Example | 軽い | 例ベースで探索と相性が良い。ドメインが曖昧でも「具体例」から合意形成できる |

### 探索的プロダクトでのリスク（一次的論点）
Spec/Kiro系の「重い仕様先行」は、要件が事前に固まりにくい探索的プロダクトでは **過剰形式化** の罠（フィードバックサイクルを遅らせ、ウォーターフォール問題が再来する）に触れやすい（[driven-development.md](driven-development.md) §2.3 の落とし穴②）。一方でBDDの「具体例先行」は探索と両立する。

### 意思決定への含意（Q2）
**ハイブリッド採用**を提案:
- 骨格は Kiro流の **requirements.md / design.md / tasks.md の3分離**（明快で我々のIssue階層と対応）。
- 要件記述は **EARS**（安定した不変条件・受入基準）＋ **BDD具体例(Given-When-Then)**（探索的な振る舞い）を併用。
- Spec Kit の **Clarify/Analyze** フェーズは「重い儀式」としてではなく、AIエージェントへの**曖昧さ解消の軽い往復**として取り入れる。

### 反証条件の検証（Q2）
「探索的プロダクトに重い仕様先行が逆効果という強い一次事例が出れば軽量側へ」→ 過剰形式化の一般的警告は複数の一次/二次源で確認（部分的に反証方向）。**よって重いSpec Kitフル儀式は初期不採用、BDD具体例を主、EARSを従、の軽量ハイブリッドに倒す。**（※ Fowler "specs not prompts" 等の追加一次事例は404で未取得＝今後補完）

---

## Q3. TDD(t-wada)/DDD との非形骸的接続

### 二重ループ（GOOS）で仕様駆動とTDDを入れ子にする
[driven-development.md](driven-development.md) §2.4 の通り（Freeman & Pryce, GOOS 2009）:
- **外側ループ = 受入テスト（仕様）**: EARS/BDDで書いた受入基準を先に Red にする。
- **内側ループ = 単体テスト（TDD）**: Canon TDD（テストリスト→1つ→Green→Refactor、歩幅調整）で外側を緑にする。
- **Walking Skeleton**: ビルド〜配布〜E2Eを通せる最薄スライスを先に通す。

### DDDのユビキタス言語を「貫通」させる
最重要規律（[driven-development.md](driven-development.md) §4.2）: **ユビキタス言語を requirements(EARS/BDD)・テスト名・ドメインモデルに一致**させる。
- requirements.md の語彙＝ドメインモデルの語彙＝単体テスト名の語彙。
- 用語が変われば仕様・テスト・モデルを**同時更新**（片方だけ直すと言語が分裂しBounded Contextが崩壊）。

### 意思決定への含意（Q3）
開発ワークフローの運用定義: **「requirements(EARS+BDD受入) → 受入テスト(外側ループRed) → TDD(内側ループ) → DDDモデル還流」** を1機能の標準サイクルとする。

### 反証条件の検証（Q3）
「二重ループが単独開発で過剰なら簡略化」→ GOOS自体は重厚だが、**コアドメインに限定適用**すれば単独開発でも過剰にならない（[ADR-0006](../adr/0006-scope-completeness-policy.md) に従い"機能を削る"のではなく"適用範囲を段階化"）。**二重ループは採用、ただしコアドメイン優先で展開。**

---

## 結論: QuickScribe が採用すべき仕様駆動の運用方針（提案）

1. **要件と設計を分離**。Issueテンプレ(Story/Task)に「要件(EARS受入基準＋必要に応じBDD具体例)」と「設計」を別セクションで持たせる。
2. **EARS記法を受入基準に採用**（安定不変条件）＋ **Given-When-Then** を探索的振る舞いに併用。
3. **requirements.md / design.md / tasks.md の3分離**（Kiro流）を機能ディレクトリ単位で持つ。重いSpec Kitフル儀式は初期不採用、Clarify/Analyzeは軽い往復として活用。
4. **二重ループ**（外側=受入/内側=TDD）を標準サイクルに。Walking Skeleton先行。
5. **ユビキタス言語を仕様・テスト名・モデルに貫通**。用語変更は同時更新。
6. これらを **運用定義ADR（次の一手）** として確定する。

## 未解決・今後の補完（透明性）
- Zenn記事の長文逐語は WebFetch の要約傾向で安定取得できず（コア2文のみ逐語確認）。
- Fowler/Böckeler "specs not prompts" は推定URLが404で一次裏付けに含められず → 連載インデックスから再特定が必要。
- 「非同期・分業前提」の直接逐語は現状 Zenn(Naosuke) が唯一。英語圏一次発信の追加裏付けが望ましい。

## 出典
[sources/](sources/) のスナップショット参照。主URL: Zenn https://zenn.dev/optimisuke/articles/090949f0487326 ／ Spec Kit https://github.com/github/spec-kit ／ GitHub Blog https://github.blog/ai-and-ml/generative-ai/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit/ ／ Kiro https://kiro.dev/docs/specs/feature-specs/ ・ https://kiro.dev/blog/introducing-kiro/ ／ Qiita https://qiita.com/ju-kosaka/items/3674294dc301f5dcf453
