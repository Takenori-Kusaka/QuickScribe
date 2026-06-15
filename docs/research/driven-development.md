# 駆動開発 実践リサーチ — TDD(t-wada流) / 仕様駆動 / DDD

> Status: Reference (2026-06-16) — ステップ1.1の調査成果。「形だけの駆動開発」に陥らないための一次情報ベースの実践指針。
> この文書は運用規範のADR（仕様駆動・TDD・DDDの運用定義ADR）を書くための根拠資料。

問題意識: これら3手法はいずれも **セマンティック・ディフュージョン**（言葉が広まる過程で本来の意味が薄まる現象）の犠牲になっている。Kent Beck は2023年に「Canon TDD」で定義を書き直し、Eric Evans は「戦術ばかり注目され戦略が無視される」と繰り返し警告し、仕様駆動はAI文脈で意味が三分裂している。各手法の狭義の定義・本来の意図・アンチパターン・正しい手順・統合を一次情報で押さえる。

---

## 1. テスト駆動開発（t-wada流 / Kent Beck原典）

### 1.1 ゴール:「動作するきれいなコード(clean code that works)」
Kent Beck『Test-Driven Development: By Example』(2002)。「まず"動作する"を解決し、次に"きれいなコード"を解決する」→ Red→Green→Refactor のリズムに対応。邦訳は和田卓人訳『テスト駆動開発』(オーム社, 2017)。

### 1.2 Red → Green → Refactor の厳密な意味
- **Red**: まだない機能に対し、**失敗するテストを1つだけ**書き、実際に赤を確認。
- **Green**: できるだけ**素早く**緑に。ベタ書き定数を返す「仮実装」のような“罪”も一時的に許容。
- **Refactor**: 緑を保ったまま**重複を除去**し設計改善。

### 1.3 「テストファースト」と「テスト駆動」は違う（t-wadaの最重要区別）
テストを先に書くこと自体が目的ではない。**テストと実装を反復しながら設計を駆動し、フィードバックで方向修正するプロセス**こそTDD。日本では「自動テスト・テストファースト・開発者テスト」と混同されがち。出典: t-wada「【翻訳】テスト駆動開発の定義」(Canon TDD訳)。

### 1.4 「TDDは設計手法である」
t-wada: 「TDD自体がもたらす利点は"使いやすい"設計ができること」「保守性は、正確にはTDDの中で"自動テストを書くこと"によって得られる効果」。
→ **TDD本体=設計手法 / 自動テスト資産=副産物**。出典: t-wada, Agile Journey 2023/11/30。

### 1.5 歩幅の調整: 仮実装 → 三角測量 → 明白な実装
- **仮実装(Fake It)**: 自信がない時。定数を返して緑にし徐々に本実装へ（歩幅・小）。
- **三角測量(Triangulation)**: 一般化に確信が持てない時。複数テスト例から一般解を測量（歩幅・最小）。
- **明白な実装(Obvious Implementation)**: 自明な時はいきなり本実装（歩幅・大）。
出典: Kent Beck原典各章 / t-wada「50分でわかるテスト駆動開発」SpeakerDeck。

### 1.6 Canon TDD の正確な手順（テストリストが前段）
多くが Red-Green-Refactor と記憶するが、Beck の Canon TDD(2023) はその前に**テストリスト**を置く:
1. **テストリスト**: カバーしたいシナリオを洗い出す
2. **1つだけ**テストを書き失敗を確認
3. **Green**: 今と過去全テストを通す。気づきはリストに追加
4. 必要に応じ**リファクタ**（ここで初めて設計判断）
5. リストが空になるまで2へ
t-wada:「最初のリストはだいたい間違っている」前提で柔軟に直す。

### 1.7 「テストのないコードはレガシーコード」
出典: Michael Feathers『Working Effectively with Legacy Code』(2004) "legacy code is simply code without tests."

### 1.8 アンチパターン（形だけのTDD）
- テストリスト欠落 / リストに実装設計を混入
- カバレッジ至上主義 / アサーションのないテスト
- 偽りのGreen（アサーション削除、実行結果を期待値にコピペ）
- リファクタの省略 or Green過程への混入 / 早すぎる抽象化
- モック過剰で実装に密結合した壊れやすいテスト
- テストファースト原理主義（教条的な他者への強制）

### 1.9 「TDDは死んだのか」論争と t-wada の現代的立場
発端: DHH「TDD is dead」(2014)が test-induced design damage を批判 → Beck×DHH×Fowler対談で「設計の害はTDDでなくまずい設計判断から」「It depends」と総括。
t-wada(2025): **TDDをAI時代のガードレール**として再評価。「AIエージェントはよくコードを壊す」「一つテストを書き、わざと失敗させ、それをクリアにする古典的で厳格なTDDをやってほしい」。Canon TDDのゴール（既存の動作を壊さず新機能も動く）と一致。

---

## 2. 仕様駆動開発（3つの異なる意味を分離せよ）

### 2.1 意味(a): BDD / 受入テスト駆動(ATDD)
- Dan North の **BDD**(2006): 「test」を「behaviour」に置換したのが核心。
- **Given-When-Then**(Gherkin/Cucumber): ドキュメントと自動テストを兼ねる **living documentation**。
- **Specification by Example**(Gojko Adzic, 2011): 具体例でギャップを埋め、実行可能な受入テストにする。
→ 「仕様」= 合意した**具体例ベースの受入基準**を自動実行可能に固定。

### 2.2 意味(b): 形式仕様
- **TLA+**(Leslie Lamport): 並行/分散システムを数学で記述、**設計の根本的欠陥をコード化前に排除**。
- **Alloy**(Daniel Jackson)。
→ 「仕様」= 数学的記述で実装前に正しさを検証。受入基準とは層が異なる。

### 2.3 意味(c): AIコーディング文脈の Spec-Driven Development (2024-2026)
- **GitHub Spec Kit**(MIT): 「仕様が実行可能になり、実装を直接生成する」。intent-driven（"what" を "how" の前に）。フェーズは版で進化（公式ブログ4段階 specify→plan→tasks→implement / READMEは Constitution→Specify→Clarify→Plan→Tasks→Analyze→Implement の7段階）。各段に人間の批評チェックポイント。
- **AWS Kiro**: requirements.md（**EARS記法**: WHEN/WHILE … the <system> SHALL <response>）→ design.md → tasks.md の3フェーズ。EARSは Alistair Mavin ら(Rolls-Royce, RE'09 2009)。
- **BDDとの違い**(Thoughtworks): BDDは仕様をコラボ用シナリオに、SDDは要件をAIへの構造化Markdown入力に形式化。
- **落とし穴**: ①仕様ドリフト/陳腐化（実装速度がドキュメント更新を上回る→CI/CDで機械検知）、②過剰形式化（ウォーターフォール問題の再来）、③仕様の良し悪しを評価する標準が未確立。

### 2.4 仕様駆動とTDDの関係（Double-loop / Outside-In TDD）
GOOS(Freeman & Pryce, 2009):
- **外側ループ = 受入テスト**: 機能レベルの仕様を先に固定（=意味(a)）。時間〜日単位。
- **内側ループ = 単体テスト**: 外側を緑にするため Red-Green-Refactor。分単位。
- **Walking Skeleton**: ビルド・デプロイ・E2Eを通せる最薄の実機能スライスを先に作る。
→ **ATDD/BDD(外側) ⊃ TDD(内側)** の入れ子。AIのSDDは外側の「仕様先行」をコード生成へ拡張したもの。

---

## 3. ドメイン駆動設計（DDD）

### 3.1 戦略的設計こそ本体、戦術は従
Eric Evans:「すべてにDDDを適用するな。コンテキストマップを描き、押し進める所と押し進めない所を決めよ」。
- **ユビキタス言語**: 開発者とドメインエキスパートが共有する厳密な共通言語。会話で使うことがモデルのテスト。
- **境界づけられたコンテキスト**: モデルを複数に分割。境界は**ユビキタス言語が変わる地点**。
- **コンテキストマップ**: コンテキスト間の関係を描く。

### 3.2 戦術的設計
エンティティ / 値オブジェクト / 集約 / リポジトリ / ドメインサービス / ドメインイベント。
**集約4ルール**(Vaughn Vernon): ①真の不変条件を一貫性境界内に（1Tx=1集約）②小さく ③他集約はID参照 ④境界外は結果整合性。

### 3.3 アンチパターン
- **貧血ドメインモデル**: 振る舞いのないゲッター/セッターの塊。ロジックが全てサービスに。OO原理に反し手続き型に退行。
- **DDD-lite**: 戦略（Bounded Context/ユビキタス言語/コアドメイン）を省き戦術だけ導入する失敗。「完璧な集約だが間違ったドメインをモデル化」。

### 3.4 ドメイン発見: イベントストーミング
Alberto Brandolini。協働ワークショップで Bounded Context・サブドメイン・集約を発見（まず戦略の原則と整合）。

### 3.5 日本語コミュニティ一次情報
- 増田亨(@masuda220):『現場で役立つシステム設計の原則』、Learning DDD邦訳解説。「分けて、つなぐ」。
- 松岡幸一郎(@little_hand_s):『ドメイン駆動設計 モデリング/実装ガイド』。

---

## 4. 3手法の統合（一気通貫ワークフロー）

```
【DDD戦略】イベントストーミング → Bounded Context / コアドメイン / ユビキタス言語 を確定
        │   （この語彙が以降すべての層の用語になる）
        ▼
【仕様駆動=外側ループ】ユビキタス言語で受入基準を記述
        │   ・BDD: Given-When-Then(Gherkin)  ・AI: requirements(EARS)→design→tasks
        │   ・受入テスト(Red)を1つ立てる
        ▼
【TDD=内側ループ】受入を緑にするため単体で Red→Green→Refactor
        │   ・Canon TDD: テストリスト→1つ書く→Green→Refactor
        │   ・歩幅: 仮実装 / 三角測量 / 明白な実装
        │   ・テスト名・変数名にユビキタス言語を使う
        ▼
【DDDモデルへ反映】Refactorで得た知見を Entity/値オブジェクト/集約へ還流
        ▼
   受入テスト緑 → 外側ループの次の1件へ
```

**最重要規律**: ユビキタス言語を **ドメインモデル・受入シナリオ・単体テスト名** に貫通させる。用語が変わったら**仕様・テスト・モデルを同時更新**（片方だけ直すと言語が分裂しBounded Contextが崩壊）。

> 注: 単独/小規模での「軽量化」は、本プロジェクトでは [ADR-0006](../adr/0006-scope-completeness-policy.md) に従い安易に適用しない。儀式の削減は最終ゴールの機能集合を削らない範囲でのみ検討する。

---

## 5. 「形だけにしないため」のチェックリスト

### TDD(Canon TDD / t-wada流)
- [ ] テストリストを先に書いたか（飛ばすと「テストファースト」止まり）
- [ ] テストは1つずつか / Redを実際に確認したか
- [ ] Refactor局面を省いていないか
- [ ] 偽りのGreen（アサーション削除・期待値コピペ）をしていないか
- [ ] 歩幅を意識調整したか（明白な実装/仮実装/三角測量）
- [ ] テストが実装に密結合していないか（モック過剰でリファクタで壊れないか）
- [ ] カバレッジを目的化していないか
- [ ] TDDを設計手法として使えているか
- [ ] AI協業時、「厳格なTDD」「既存動作を壊さない」をガードレール指示したか

### 仕様駆動
- [ ] 使っている「仕様駆動」が (a)BDD/ATDD / (b)形式仕様 / (c)AI-SDD のどれか自覚しているか
- [ ] 受入基準を具体例(Given-When-Then / EARS)で固定しているか
- [ ] 仕様が実行可能 or 自動テストと紐づいているか（living documentation）
- [ ] 仕様が外側ループ、TDDが内側ループとして入れ子か
- [ ] 仕様ドリフトをCIで機械検知できるか
- [ ] 過剰形式化でフィードバックを遅らせていないか
- [ ] 仕様の語彙がユビキタス言語と一致しているか

### DDD
- [ ] 戦略的設計（ユビキタス言語/Bounded Context/コンテキストマップ）を先にやったか
- [ ] コアドメインに集中しているか
- [ ] ユビキタス言語がコード・テスト・会議で実際に使われているか
- [ ] ドメインオブジェクトが振る舞いを持つか（貧血モデルでないか）
- [ ] 集約は真の不変条件に基づき小さく、他集約はID参照か
- [ ] 境界外で結果整合性を使っているか
- [ ] イベントストーミング等でドメインを発見してから戦術に落としたか
- [ ] Bounded Contextの境界がユビキタス言語が変わる地点と一致しているか

### 統合（共通）
- [ ] ユビキタス言語がモデル・受入シナリオ・単体テスト名を貫通しているか
- [ ] 外側→内側→DDDモデル還流のループが閉じているか
- [ ] 用語変更時に仕様・テスト・モデルを同時更新しているか

---

## 主要出典
- TDD: [Canon TDD](https://newsletter.kentbeck.com/p/canon-tdd) / [t-wada訳](https://t-wada.hatenablog.jp/entry/canon-tdd-by-kent-beck) / 『TDD by Example』(Beck 2002) / [TDD Live 50min](https://speakerdeck.com/twada/tdd-live-in-50-minutes) / [Agile Journey 2023](https://agilejourney.uzabase.com/entry/2023/11/30/103000) [2025](https://agilejourney.uzabase.com/entry/2025/08/29/103000) / [TDD is dead](https://dhh.dk/2014/tdd-is-dead-long-live-testing.html) / [Is TDD Dead?](https://martinfowler.com/articles/is-tdd-dead/) / Feathers『Legacy Code』(2004)
- 仕様: [Introducing BDD](https://dannorth.net/introducing-bdd/) / [Gherkin](https://cucumber.io/docs/gherkin/reference/) / [Specification by Example](https://gojko.net/books/specification-by-example/) / [TLA+](https://lamport.azurewebsites.net/tla/tla.html) / [GitHub Spec Kit](https://github.com/github/spec-kit) / [Kiro](https://kiro.dev/) / [EARS](https://alistairmavin.com/ears/) / [Thoughtworks SDD](https://www.thoughtworks.com/en-us/insights/blog/agile-engineering-practices/spec-driven-development-unpacking-2025-new-engineering-practices) / GOOS(2009)
- DDD: [BoundedContext](https://martinfowler.com/bliki/BoundedContext.html) / [UbiquitousLanguage](https://martinfowler.com/bliki/UbiquitousLanguage.html) / [AnemicDomainModel](https://martinfowler.com/bliki/AnemicDomainModel.html) / [Effective Aggregate Design](https://www.dddcommunity.org/library/vernon_2011/) / [DDD Matters Today](https://www.infoq.com/articles/eric-evans-ddd-matters-today/) / [eventstorming.com](https://www.eventstorming.com/) / [増田亨](https://speakerdeck.com/masuda220) / [松岡](https://little-hands.hatenablog.com/)
