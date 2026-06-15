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

### 探索的プロダクトでのリスク（一次事例）
重い仕様先行が逆効果になる一次事例を確認した（[sources/spec-driven-limits.md](sources/spec-driven-limits.md)）:
- **Isoform "The Limits of Spec-Driven Development"（2025）**: 安定した契約・既知ドメインには有効だが、要件が進化する探索的開発には context-driven の方が適応的。「仕様をコードと同期し続けることはシステム複雑度とともに増す維持税(maintenance tax)を生む」「静的な仕様に固定されすぎると反復・創造・創発的解が減る」。
- **Kiro Quick Plan（公式）**: 「要件や設計を反復する必要がなく出力を信頼できる、よく理解された機能」「速度がレビューより重要なラピッドプロトタイピング」に軽量プランを推奨。

**精緻化（重要）**: 「探索的＝とにかく軽量」は単純化しすぎ。一次情報の正確な論旨は「**要件が頻繁に反復・破棄される段階で重い仕様化はメンテ税で逆効果**」。QuickScribeは"思考整理"というドメイン理解自体が探索対象＝要件が破棄されやすい → **初期は軽量が妥当**。

### 意思決定への含意（Q2）
**軽量BDD中心のハイブリッド**を採用:
- 受入条件は **BDDの Given-When-Then を外側ループの中核**に（最軽量・探索に強い）。
- **EARSは「固まった機能のみ」に条件付き採用**（安定領域だけ requirements.md に昇格）。探索段階は自由記述＋受入例。
- 章立て構造のみ Kiro流の **Requirements / Design / Tasks** を踏襲。Spec Kit/Kiroのフル自動ワークフローは初期は不採用（探索阻害）。

### 反証条件の検証（Q2）
「探索的プロダクトに重い仕様先行が逆効果という強い一次事例が出れば軽量側へ」→ **反証成立（軽量側へ）**。Isoform・OpenSpec実験・Kiro Quick Plan公式が、進化的要件下での重い仕様先行のコスト超過を一次情報で示した。

---

## Q3. TDD(t-wada)/DDD との非形骸的接続

### 二重ループ（GOOS）で仕様駆動とTDDを入れ子にする
Freeman & Pryce, GOOS（2009、著者公開スライドで逐語確認）: 機能実装は受入テストから始め、その下で単体テスト→実装→リファクタのサイクルを回す。**外側ループ＝実証可能な進捗の尺度、内側ループ＝開発者を支える**。Walking Skeleton＝「自動でビルド・デプロイ・E2Eできる最薄の実機能スライス」。
- **外側ループ = 受入テスト（仕様）**: BDD/EARSで書いた受入基準を先に Red にする。
- **内側ループ = 単体テスト（TDD）**: Canon TDD（テストリスト→1つ→Green→Refactor、歩幅調整）で外側を緑にする。

**t-wadaのテスト分類と完全整合**（gihyo連載・本人執筆、短い逐語）: 「受け入れテストとは進捗管理のためのテスト」「単体テスト…は…Developer Testing にカテゴライズ」。GOOSの外側=進捗/内側=開発者テストと一致。

**Living Documentation で形骸化を構造的に防ぐ**（Gojko Adzic, Specification by Example、短い逐語）: 「単一のドキュメントが仕様とテストの両方を表すなら、片方だけ更新し忘れることは不可能になる」。受入テスト＝仕様＝実行されるドキュメント。

### DDDのユビキタス言語を「貫通」させる
最重要規律（[driven-development.md](driven-development.md) §4.2）: **ユビキタス言語を requirements(EARS/BDD)・テスト名・ドメインモデルに一致**させる。
- requirements.md の語彙＝ドメインモデルの語彙＝単体テスト名の語彙。
- 用語が変われば仕様・テスト・モデルを**同時更新**（片方だけ直すと言語が分裂しBounded Contextが崩壊）。

### 意思決定への含意（Q3）
開発ワークフローの運用定義: **「requirements(EARS+BDD受入) → 受入テスト(外側ループRed) → TDD(内側ループ) → DDDモデル還流」** を1機能の標準サイクルとする。

### AIエージェント文脈での再評価（と留保）
外側ループはAI時代に**AIの「完了詐称」を止めるガードレール**として有効。GitHub Spec Kit（spec-driven.md）: 仕様を「受動的文書からアクティブな品質ゲートへ」変える（TDDが要件を実行可能テストへ変えるのと同様）。Justin Searls の dual-loop BDD は、AIが「テスト未実行で成功宣言」する問題への対策として失敗する統合テストから内側へ駆動する。
**留保（Fowler, sdd-3-tools）**: 「これだけのファイル・テンプレート・プロンプトがあっても、エージェントが全指示に従わないのを頻繁に見た」「受入条件はAIが解釈するので100%尊重される保証はない」。→ 外側ループは「あれば安心」でなく**実行してまずRedを確認**する運用が必須。

### 反証条件の検証（Q3）
「二重ループが単独開発で過剰なら簡略化」→ **明確には反証されず、むしろ条件付き支持**。「ソロで過剰」と名指す原典は未発見。逆にAI開発では外側ループが完了詐称の機械的ガードレールとして有効（prove_it, spec-kit）。**よって二重ループは採用、ただし1フィーチャー1受入テストに絞った軽量運用＋コアドメイン優先で展開**（[ADR-0006](../adr/0006-scope-completeness-policy.md) に従い"機能を削る"のではなく"適用範囲を段階化"）。

---

## 結論: QuickScribe が採用すべき仕様駆動の運用方針（提案）

1. **要件と設計を分離**。Issueテンプレ(Story/Task)に「要件(EARS受入基準＋必要に応じBDD具体例)」と「設計」を別セクションで持たせる。
2. **EARS記法を受入基準に採用**（安定不変条件）＋ **Given-When-Then** を探索的振る舞いに併用。
3. **requirements.md / design.md / tasks.md の3分離**（Kiro流）を機能ディレクトリ単位で持つ。重いSpec Kitフル儀式は初期不採用、Clarify/Analyzeは軽い往復として活用。
4. **二重ループ**（外側=受入/内側=TDD）を標準サイクルに。1フィーチャー1受入テストの軽量運用。Walking Skeleton先行。
5. **ユビキタス言語を仕様・テスト名・モデルに貫通**。用語変更は同時更新。Living Documentation（仕様=テスト）で形骸化を構造的に防止。
6. **AIエージェント運用契約**: エージェントに「受入テストを先に書き、Redで失敗することを確認してから内側ループへ」を義務づけ、AIの完了詐称（Fowler/Searlsの指摘）を機械的に止める。
7. これらを **運用定義ADR（次の一手）** として確定する。

### 指摘した通説の誤り
- EARSの「Optional=IF」「Unwanted=SHALL NOT」は**誤り**。正準は **Optional=WHERE / Unwanted behaviour=IF…THEN**（調査中も要約ツールが一度取り違えた）。
- 「Spec-Driven Developmentは新概念」は不正確。本質は**29148/EARSのWHAT/HOW分離の再適用**。
- 「探索的＝とにかく軽量」は単純化しすぎ（正：要件が頻繁に破棄される段階で重い仕様化が逆効果）。
- 「仕様駆動でAIを確実に拘束できる」は誤り（AIは100%遵守しない＝受入テストの実行が不可欠）。

## 未解決・今後の補完（透明性）
- Zenn記事の長文逐語は WebFetch の要約傾向で安定取得できず（コア数文のみ逐語確認）。
- ISO/IEC/IEEE 29148 本文の条番号付き完全逐語は有料ページのため未達（"Implementation free" は趣旨裏取り）。最終引用時は2018正本での照合を推奨。
- DDD Reference本体・GOOS一部逐語・t-wada「二重ループ」用語の一次資料は未特定（趣旨は裏取り済み）。
- （解決）Fowlerの該当記事は `exploring-gen-ai/sdd-3-tools.html` に存在（旧URL404は誤り）。

## 出典
[sources/](sources/) のスナップショット参照。主URL: Zenn https://zenn.dev/optimisuke/articles/090949f0487326 ／ Spec Kit https://github.com/github/spec-kit ／ GitHub Blog https://github.blog/ai-and-ml/generative-ai/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit/ ／ Kiro https://kiro.dev/docs/specs/feature-specs/ ・ https://kiro.dev/blog/introducing-kiro/ ／ Qiita https://qiita.com/ju-kosaka/items/3674294dc301f5dcf453
