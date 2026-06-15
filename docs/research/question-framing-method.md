# 本質的な問いの立て方 — deep research の問い設計メソッド

> Status: Reference / Binding method (2026-06-16)
> 本プロジェクトのすべての deep research は、この手順書(A)・チェックリスト(B)に従って「問い」を設計してから実行する（[ADR-0007](../adr/0007-research-question-framing-method.md)）。
> 目的: 表層的・要約的な調査を避け、プロダクトのコア価値と意思決定に直結した「良い問い」を立てる。

各フレームワークは提唱者・原典・初出年・核心的主張を一次情報で押さえる。安易な要約で薄めない。

---

## 第1部 良い問いの評価基準

### FINER（Feasible / Interesting / Novel / Ethical / Relevant）
- 提唱: Stephen B. Hulley, Steven R. Cummings ら（UCSF）。原典: *Designing Clinical Research*（Williams & Wilkins, 1988初版）冒頭章「Conceiving the Research Question」。**教科書初出**。
- 着想直後に5基準で吟味: F=実行可能（被験者数/専門性/時間費用/スコープ）, I=研究者自身が答えを知りたいか, N=既存知見の確認/反証/拡張, E=倫理（IRB承認可）, R=知識/実務/将来研究への意義。

### PICO（臨床リサーチクエスチョンの構造化）
- 提唱: Richardson, Wilson, Nishikawa, Hayward。原典: "The well-built clinical question: a key to evidence-based decisions." *ACP Journal Club*. 1995;123(3):A12–A13. PMID 7582737。
- 核心: EBM最初の技能は「よく構築された問いを立てること」。P=患者/集団/問題, I=介入/曝露, C=比較対照, O=意味のある結果。派生: PICOT(時間), PICOS(研究デザイン), PECO(曝露)。

### Heilmeier Catechism（DARPA・正典8問）
- 提唱: George H. Heilmeier（DARPA長官 1975–77）。DARPA公式が正典公開。
1. What are you trying to do? Articulate your objectives using absolutely no jargon.
2. How is it done today, and what are the limits of current practice?
3. What is new in your approach and why do you think it will be successful?
4. **Who cares? If you are successful, what difference will it make?**
5. What are the risks?  6. How much will it cost?  7. How long will it take?
8. What are the mid-term and final "exams" to check for success?
- 出典: https://www.darpa.mil/about/heilmeier-catechism

### GQM（Goal–Question–Metric）
- 提唱: Victor R. Basili ら。原典: Basili & Weiss, *IEEE TSE* SE-10(6), 1984 / "The Goal Question Metric Approach"(1994)。
- 核心: 測定は目的志向。トップダウンで Goal→Question→Metric。「first specify the goals … then trace those goals to the data … finally provide a framework for interpreting the data」。
- 出典: https://www.cs.umd.edu/users/mvz/handouts/gqm.pdf

---

## 第2部 問いを生成・洗練する技法

### QFT（Question Formulation Technique）
- 提唱: Dan Rothstein & Luz Santana（Right Question Institute）。原典: *Make Just One Change*（Harvard Education Press, 2011）。
- 手順: (a) **QFocus**（焦点文。疑問文にしてはならない）→ (b) 四ルール → (c) 大量生成 → (d) open/closed分類と相互変換 → (e) 最重要3問に優先順位 → (f) 振り返り（必須）。
- **四ルール（逐語）**: 1. Ask as many questions as you can. 2. Do not stop to answer, judge, or discuss. 3. Write down every question exactly as stated. 4. Change any statements into questions.
- 出典: https://rightquestion.org/what-is-the-qft/

### Socratic Questioning（Paul-Elder 6分類）
- 提唱: Richard Paul & Linda Elder。原典: *The Thinker's Guide to the Art of Socratic Questioning*（2006）。
- 6分類: ①明確化 ②前提を探る ③理由・根拠 ④視点・観点 ⑤含意・帰結 ⑥問いそのものへの問い（"To answer this, what other questions must we answer first?"）。
- 出典: https://www.criticalthinking.org/files/SocraticQuestioning2006.pdf

---

## 第3部 問題の枠組み(framing)と再枠組み(reframing)

### Wedell-Wedellsborg — リフレーミング
- 原典: *What's Your Problem?*（HBR Press, 2020）/ "Are You Solving the Right Problems?" *HBR* 2017。
- データ: C-suite 106名で「85%が自社は問題診断が下手」「87%がその欠陥は重大コスト」。
- 根本原因分析との違い: 5 Whysは「最初の問題理解の中を深掘り」、リフレーミングは「**まったく別の捉え方はないか**」。目的は「真の問題」発見でなく「**より良く解ける問題**」の確認。
- 5レンズ: 枠の外を見る / 目標を問い直す / うまくいった例外 / 自分の役割 / 当事者の視点。象徴例「遅いエレベーター→鏡を設置」。
- 出典: https://hbr.org/2017/01/are-you-solving-the-right-problems

### How Might We（HMW）
- **真の起源**: IDEO/d.schoolは通説。実際は1971年 Min Basadur（P&G）が造語。Irish Spring逸話「How can we make a better green-stripe bar?→How might we create a more refreshing soap of our own?」。
- 広すぎ/狭すぎを避け sweet spot を狙う。出典: https://www.basadur.com/the-origin-of-how-might-we/

### Dewey の探究理論
- 原典: *How We Think*(1910) / *Logic: The Theory of Inquiry*(1938)。
- 探究=不確定な状況を確定した状況へ統制的に変換する。最初の仕事は不確定状況を**問題として立てること**。
- **「問題がうまく立てられれば半ば解かれている」**: Dewey原文「It is a familiar and significant saying that a problem well put is half-solved.」（Dewey自身が"よく知られた言い回し"と前置き。語は "well put"）。
- 出典: https://www.gutenberg.org/files/37423/37423-h/37423-h.htm

### 「アインシュタイン55分」逸話 ← 誤帰属（検証済み）
- Quote Investigator: 「There is no substantive evidence that Einstein ever made a remark of this type.」初のEinstein帰属は1973年。引用するなら Dewey が適切。
- 出典: https://quoteinvestigator.com/2014/05/22/solve/

---

## 第4部 深さ・根本原因へ掘る技法

### 5 Whys（なぜなぜ分析）
- 原典: 大野耐一『トヨタ生産方式』(1978)。機械停止の例 → 真因はストレーナー欠如。「By asking why five times … we can get to the real cause … hidden behind more obvious symptoms.」
- **厳密なやり方**: 現地現物で事実確認 / 症状でなく真因に恒久対策 / 逆読みで因果検証。
- **形骸化した誤用**: 単一連鎖しか追わない / 人を責める（本来はプロセスの欠陥を探す）/ 早期打ち切り / 現場確認せず推測。

### First-Principles Thinking
- 起源: アリストテレス『形而上学』。archē=起源/原理「A first principle is the first basis from which a thing is known.」。
- 現代: Musk「reason from first principles rather than by analogy」。最も基本的な真実まで分解し「確実に真と言えること」から積み上げる。

### MECE / ピラミッド原則 / 仮説駆動（Minto）
- 原典: *The Minto Pyramid Principle*(1985)。MECE=漏れなくダブりなく。結論先行（answer-first）。イシューツリーで大問をMECE分解し末端に検証可能なサブ問題。day-1 hypothesis。

### 安宅和人『イシューからはじめよ』(英治出版, 2010)
- イシュー度（いま答えを出す必要性）×解の質 の2×2。右上＝バリューのある仕事。まず横軸（正しい問い）を上げよ。「犬の道」（気合いで解の質だけ上げる）を避ける。

---

## 第5部 意思決定駆動の問い

### Value of Information（VOI / EVPI）
- 原典: Ronald A. Howard, "Information Value Theory," *IEEE Trans. SSC* 2(1), 1966。源流 Raiffa & Schlaifer(1961)。
- 核心: **情報は意思決定を変えうるときにのみ価値を持つ**。判定:「この問いの答えはどの意思決定を変えるか? どの答えでも行動が変わらないなら VOI=0（問う価値なし）」。

### 反証的証拠（Popper）
- 原典: *Conjectures and Refutations*(1963)。「Every genuine test of a theory is an attempt to falsify it, or refute it.」
- 接続: 良い問いは「**自分が間違っていると分かる条件(disconfirming evidence)を探す**」もの。

---

## 第6部 陥りやすい罠
- 複合質問(loaded)・二重質問(double-barreled)・誘導質問(leading): 前提埋め込み/2主題/特定回答へ誘導。
- XY問題: 解法Yの方法でなく、達成したい目標Xを問え。
- 街灯効果(streetlight effect): 答えのある所でなく探しやすい所を探す誤り。
- 確証バイアス: Wason(1960) 2-4-6課題。反証例を試さない。
- フレーミング効果: Tversky & Kahneman(1981) Asian disease。利得/損失フレームで選好逆転。

---

## 第7部 AI時代/AIリサーチエージェント文脈
- **Least-to-Most**(Zhou 2022): 複雑問題を簡単なサブ問題列に分解し順に解く。
- **Self-Ask**(Press 2022): 答える前にフォローアップ質問を自問（外部検索に委譲可）。
- **DecomP**(Khot 2022): サブタスクを専用ハンドラへ委譲、必要なら再帰分解。
- **Anthropic マルチエージェント研究**(2025): 各サブタスクに「目的・出力形式・使うツール/ソース・境界」を明示。複雑度でスケール。
- **JTBD接続**: ジョブ=「the progress that a person is trying to make in a particular circumstance」(Christensen)。Job Story「When [situation], I want to [motivation], so I can [expected outcome].」。ODI outcome statement（minimize/maximize+指標+対象+文脈、解決策非依存・測定可能）。

---

## (A) 本質的 deep research の問い設計 手順書

- **Step 0 リフレーミング**: 解こうとする問題を一文で書き、5レンズで別の捉え方を試す。「真の問題」でなく「より良く解ける問題」を探す。
- **Step 1 QFocus定義**: トピックを疑問文でない焦点文にする。
- **Step 2 ジョブ接続**: 「When ___, I want to ___, so I can ___」でコア価値に錨を打つ。
- **Step 3 発散**: QFT四ルールで問いを大量生成（評価・例示を挟まない）。
- **Step 4 深掘り次元**: Socratic 6分類を当て、重要な問いは厳密な5 Whys（現場の事実に当たる/人を責めない/症状で止めない）+ First Principles。
- **Step 5 構造化**: MECEなイシューツリーへ。答えの仮説を先に。AI研究では answerable な sub-question 列に分解し各々に目的・出力形式・ソース・境界を明示。
- **Step 6 意思決定で篩**: 各問いに Heilmeier「Who cares? what difference?」+ VOI「どの意思決定を変えるか? 変えないなら捨てる」。
- **Step 7 反証可能性**: 各問いに「何が分かれば考えが変わるか(disconfirming)」を一行添える。
- **Step 8 収束**: 最重要3問 + GQMで Goal→Question→Metric に落とす。
- **Step 9 FINERチェック → 実行 → ズレたらStep 0へ**。

## (B) 良い問いチェックリスト
- [ ] どの具体的意思決定を変えるか言えるか（VOI）。どの答えでも行動が同じ＝VOI0ではないか
- [ ] 「Who cares? 成功したら何が変わるか」に答えられるか（Heilmeier#4）
- [ ] 別の枠組みを試したか（Reframing）。解法Yでなく目標Xか（XY問題）。探しやすい所でなく答えのある所か（街灯効果）
- [ ] 専門用語抜きで目的を述べられるか。誘導/複合/前提埋め込みでないか。open/closedを意図的に使い分けたか
- [ ] 症状でなく真因か（厳密な5 Whys・現場の事実）。前提をFirst Principlesまで分解したか。イシュー度は高いか
- [ ] 「何が分かれば考えが変わるか」を言えるか（Popper）。確証バイアス/フレーミング効果に陥っていないか
- [ ] FINER（実行可能/興味深い/新規/倫理的/意義）を満たすか
- [ ] ユーザーのJob（状況下のprogress）に接続し、測定可能な指標に落ちるか（JTBD/GQM/ODI）

## (C) 悪い問い→良い問い（本プロジェクト文脈の例）
1. 「どうすればもっと使ってもらえるか?」→「『整理された』と感じた回と感じない回で行動・発話に何の差があるか? どんな差が出れば"利用頻度でなく整理体験こそが価値"という仮説が覆るか?」
2. 「文字起こし精度を何%上げれば満足するか?」→「ユーザーはどんな progress を遂げようとし、その進歩にとって精度は本当にボトルネックか、それとも"見返したときの想起しやすさ"が本質か?」
3. 「録音画面UIをどう改善?」→「思考整理の決定的瞬間はどの工程（録音中/聞き返し/要約閲覧）で起きるか? 最もイシュー度が高い工程はどこか?」
4. 「継続率が低くオンボーディングも分かりにくいのはなぜ? どう直す?」→ 厳密な5 Whys:「なぜ初週で離脱→なぜ2回目を録音しない→なぜ録音する状況が訪れない→なぜ"書く状況"を本人が認識しない…」を行動ログ/インタビューで検証。
5. 「競合機能を全部洗い出す」→「次Qの『要約自動生成 vs 手動タグUI』の投資判断を変える証拠は何か? 競合のどの事実が分かればこの判断が反転するか?」

---

## 出典一覧（主要）
DARPA Heilmeier https://www.darpa.mil/about/heilmeier-catechism ／ GQM https://www.cs.umd.edu/users/mvz/handouts/gqm.pdf ／ QFT https://rightquestion.org/what-is-the-qft/ ／ Socratic https://www.criticalthinking.org/files/SocraticQuestioning2006.pdf ／ Reframing https://hbr.org/2017/01/are-you-solving-the-right-problems ／ HMW起源 https://www.basadur.com/the-origin-of-how-might-we/ ／ Dewey https://www.gutenberg.org/files/37423/37423-h/37423-h.htm ／ Einstein検証 https://quoteinvestigator.com/2014/05/22/solve/ ／ 5 Whys https://www.lean.org/the-lean-post/articles/five-whys-animation/ ／ First Principles https://fs.blog/first-principles/ ／ Minto/MECE https://www.mckinsey.com/alumni/news-and-events/global-news/alumni-news/barbara-minto-mece-i-invented-it-so-i-get-to-say-how-to-pronounce-it ／ VOI https://en.wikipedia.org/wiki/Value_of_information ／ Popper https://plato.stanford.edu/entries/popper/ ／ フレーミング効果 https://mpra.ub.uni-muenchen.de/78270/1/MPRA_paper_78270.pdf ／ XY問題 https://mywiki.wooledge.org/XyProblem ／ Anthropic多エージェント https://www.anthropic.com/engineering/built-multi-agent-research-system ／ JTBD https://hbr.org/2016/09/know-your-customers-jobs-to-be-done ／ ODI https://strategyn.com/outcome-driven-innovation-process/

## 一次資料の限界（透明性）
FINER各語の逐語定義・PICO原典本文・安宅の逐語・JTBDミルクシェイク逸話の逐語は二次資料で再構成（原本参照推奨）。Einstein「55分」は誤帰属確定。
