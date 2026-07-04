---
title: "ローカル完結ボイスジャーナルをTauri+Rust+Svelteで作る：文字起こし精度ではなく“整形の知性”に賭けた設計判断"
emoji: "🎙"
type: "tech"
topics: ["tauri", "rust", "svelte", "whisper", "設計"]
published: false
---

<!--
この記事は articles/ 配下を「単一ソース」として管理する（Zenn CLI/GitHub連携の慣習）。
Qiita 版は `npm run qiita:build` で public/ に機械変換する。画像は必ず絶対URLで参照する
（相対パスは Qiita 転載でリンク切れになるため）。詳細は docs/process/article-publishing-policy.md。
-->

> 検証時点: 2026-07-05 / 対象リリース: v1.0.0（2026-07-04 公開）/ リポジトリ: [Takenori-Kusaka/QuickScribe](https://github.com/Takenori-Kusaka/QuickScribe)。
> 本文の数値・主張はすべてリポジトリの一次情報（ADR・実装コード・CI 実測）に紐づける。数値には必ず**計測条件**を併記する。

## 10秒で読む価値

音声入力アプリを作るなら、普通は「文字起こし精度」を主戦場に据える。この記事はその逆を採った設計の記録である。**文字起こしは差し替え可能なコモディティ入力に格下げし、価値の重心を「ニュアンスを残したまま思考を整理する整形の知性」に置く**。この決定は4つの領域に波及した。アーキテクチャ（4つの差し替え可能境界）・技術選定（何を**採らなかった**か）・ベンチマーク設計（相対指標としての日本語CER）・プライバシー既定（ローカルファースト）である。以下ではそれぞれを、意思決定記録（ADR）と実装コードに紐づけて示す。

対象読者は、音声/デスクトップアプリの設計判断を自分で下す人。「Whisperを組み込んでみた」の一段上、**どこを抽象化し、どこを捨て、その判断をどう検証可能にするか**に関心がある人を想定する。

## 1. 課題設定：なぜ既存ツールでないのか

まず「作る理由」を一次情報で固める。競合ランドスケープ（自リポジトリの[competitive-landscape.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/research/competitive-landscape.md)、取得日 2026-06-28・各社公式を一次情報として優先）を整理すると、市場は次の2極に寄っている。

- **会議特化**（Otter / Granola）：要約とアクション抽出に最適化。方向性は「決定事項を圧縮する」。
- **汎用音声入力**（OpenWhispr / superwhisper / MacWhisper / Windows Fluid Dictation）：カーソル位置へ清書テキストを挿入する。整形は cleanup（清書）にとどまる。

ここに構造的な空白がある。**「ローカル完結 × ジャーナリング特化 × 要約ではなくニュアンス保持の思考整理」の交差点**に該当する製品が、調査範囲では見当たらない。隣接する最有力は Day One（ジャーナル × AI）だが、テキスト中心のジャーナルに AI を後付けした構造で、ローカル完結が不完全である。清書は「消す」方向、要約は「捨てる」方向。QuickScribe が狙うのは**「残して育てる」**方向であり、ベクトルが逆になる。

重要なのは、この分析が「単軸では勝てない」と正直に認めている点だ。ローカルプライバシーそのものは OpenWhispr や MacWhisper（"never phones home"）が既に埋めており、コモディティ化している。だから価値は**4軸の束ね方**に宿る、という仮説から設計を始める（[vision.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/vision.md)）。

そして製品の核心課題を一行で固定する。**「リッチすぎると簡便でなくなる」バランス**である。機能を足すたびにこの問いへ戻る。この制約が、後述する「モデルを削らずラベルで導く」判断（ADR-0022）や「トグルを増やさないプライバシー可視化」（ADR-0019）に直接効いてくる。

## 2. アーキテクチャ：差し替え可能性を一級市民にする

QuickScribe は Tauri 2 のデスクトップアプリで、**Rust バックエンド**と **Svelte 5 フロントエンド（WebView）**が Tauri の `invoke`（コマンド）と `event`（イベント）で通信する。全体像は次の通り（図は CI で Mermaid から生成し、絶対URLで参照している。理由は §6）。

![QuickScribe 全体アーキテクチャ](https://raw.githubusercontent.com/Takenori-Kusaka/QuickScribe/main/docs/assets/diagrams/design-1.png)

設計の背骨は、コア価値（整形の知性）を守るために「価値でない部分」を**差し替え可能な抽象境界**として切り出したことにある。[ADR-0005](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0005-tech-stack.md) は4つの境界を一級市民として定義する。

| 抽象境界 | 役割 | 実装場所 | 差し替え例 |
|---|---|---|---|
| `TranscriptionEngine` | 音声 → テキスト（コモディティ） | `src-tauri/src/stt.rs` | ローカル whisper / Groq・OpenAI / Deepgram / Azure |
| `FormattingEngine` | テキスト → 整形（**価値の本体**） | `src-tauri/src/refine.rs` | Gemini / Anthropic / OpenAI / Ollama / Bedrock / Claude Platform on AWS |
| 録音ソース `SourceKind` | 音声取得 | `src-tauri/src/record.rs` | マイク / システム音ループバック / mix |
| 保管ドメイン | 読み書き | `lib.rs`（書き）/ `vault.rs`（読み） | Markdown/テキスト・frontmatter・スキーマ版 |

ポイントは、これが単なる「インターフェースを切りました」ではなく、**依存性逆転（DIP）を製品戦略に接続している**ことだ。`FormattingEngine` を Strategy 化しておくと、「ローカル完結 vs クラウド」というプライバシー方針の将来変更に、アーキテクチャが縛られずに追従できる。実際に後から既定をクラウド（Gemini）からローカル（Ollama）へ切り替える判断（[ADR-0021](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0021-local-first-defaults.md)）が入ったが、境界のおかげで**新プロバイダは trait 実装の追加で済み、既存コードを触らない**（OCP）。

コード上でも境界は素直だ。STT 側は `pub trait TranscriptionEngine` を定義する。設定から実装を選ぶファクトリは `engine_for(cfg: SttConfig) -> Box<dyn TranscriptionEngine>` だ。整形側も対称だ。`pub trait FormattingEngine` と `engine_for(provider: &str) -> Box<dyn FormattingEngine>` を置く。実装は `RefineProvider` enum が束ねる（Gemini/Anthropic/OpenAi/Ollama/Bedrock/ClaudePlatformAws の6種）。整形スタイル（Structured/Verbatim/Summary/Brainstorm）は `RefineStyle` として型で表現し、「逐語 ⇄ 要約 ⇄ ブレスト」の行き来を一級の概念にしている。

コア・データフロー（録音 → 文字起こし → 用語補正 → 整形 → 保存）は次の通り。

![コア・データフロー](https://raw.githubusercontent.com/Takenori-Kusaka/QuickScribe/main/docs/assets/diagrams/design-2.png)

秘密情報（APIキー・AWS資格情報）は **OS keyring**（Windows Credential Manager / Linux Secret Service）に格納し、バックエンドへはメモリ内注入のみで永続化しない。フロントの localStorage には非秘密設定だけを置く。この分離も「思考の生データを外に出さない」というプライバシー約束の一部である。

## 3. 技術選定：何を採り、何を**採らなかったか**

技術選定は「採った理由」より「採らなかった理由」に説明可能性が宿る。QuickScribe は判断を1決定1ファイルの ADR に残す規律（[ADR-0001](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0001-record-architecture-decisions.md)）を敷いており、検証時点で23件が Accepted になっている。代表的な「不採用」を3つ挙げる。

### 3.1 Google Docs 音声入力の自動駆動 → 不採用（ADR-0003）

初期構想に「梱包ブラウザで Google Docs の音声入力を自動駆動する」案があった。無料・高品質という期待があったが、[ADR-0003](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0003-reject-google-docs-automation.md) で棄却した。理由は積み上げると防御不能だった。

- **精度優位の客観的根拠がない**：独立ベンチで gpt-4o-transcribe ≈2.5% WER に対し Google Chirp2 ≈11.6%。Docs は旧世代の Web Speech API ベースで、製品自体の WER を測った査読データも存在しない。
- **ToS違反リスク**：Google 利用規約は自動アクセス・スクレイピング・保護機構の回避を明示的に禁じている。
- **技術的に持続しない**：Puppeteer/Selenium の自動ログインは "browser may not be secure" で弾かれ、`navigator.webdriver` 検出で回避は続かない。
- **機能的障壁**：ライブマイク必須・ファイル流し込み不可・公式API不在。

この不採用が効いたのは、「梱包ブラウザ＋ブラウザ自動化」という**最大の保守コストと脆弱性源が丸ごと消えた**点だ。ブラウザ依存は STT からは外し、LLM整形の BYO 認証（既存ブラウザでの Gemini/Claude 認証）に限定して残した。副産物として、システム音取り込みも仮想オーディオデバイス不要になった（Win=WASAPI ループバック / Linux=PipeWire monitor）。

### 3.2 STT 既定＝ローカル whisper.cpp（ADR-0002）

既定エンジンはローカルの `whisper.cpp` に置いた（[ADR-0002](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0002-stt-engine-strategy.md)）。理由は、完全オフライン・ToSフリーで動く既定体験がプライバシー約束と整合し、実在デスクトップアプリ8本中5本が whisper.cpp を採用するデファクトだからだ。クラウド（Groq/Deepgram/Azure）は**鍵を入れた人だけのプラグイン**にした。ここでも「不採用」を記録している。Parakeet/Canary（NVIDIA）は英語精度が世界最高だが日本語非対応で不採用。Moonshine は日本語モデルが非商用ライセンスで商用利用不可。Vosk は軽量だが精度が Whisper 系に劣るため、低スペック向けオプション候補に留めた。

### 3.3 実行基盤＝Tauri（Electron 不採用・ADR-0005）

[ADR-0005](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0005-tech-stack.md) は Tauri 2 + Svelte 5（SPA・SvelteKit 不使用）+ Rust を採用した。Electron を**常駐メモリ・起動速度・依存ツリーの小ささ**で不採用にした点が肝で、非機能要件（軽量・監査容易性・プライバシー）から逆算した判断である。Tauri は Chromium 非同梱の OS ネイティブ WebView を使うため、「既存ブラウザに依存しない」を軽量に達成できる。加えて **Vibe（Tauri + whisper-rs）**という whisper 連携の先行実証があり、これが Wails/Qt/.NET に対する決め手になった。

ここで Rust のコストにも正直でありたい。ADR は「Rust はバックエンド配線/FFI に限定し、整形ロジック（プロンプト・逐語/要約スタイル）は TS 側/LLM 側へ寄せる」というガードレールを明記している。**本体価値（整形の知性）に工数を集中するため、あえて Rust の守備範囲を狭める**という設計判断だ。

## 4. 日本語CERベンチをCIで回す：ただし“何を測っていないか”を明示する

「文字起こし精度は価値でない」と言い切るなら、精度は**回帰監視の対象**として扱えば十分になる。QuickScribe は日本語 CER（文字誤り率）ベンチを CI（`.github/workflows/perf.yml` の「日本語精度 CER」ジョブ）で回している。本人音読のパブリックドメイン作品3点を `QS_LANG=ja` で認識し、`scripts/cer_ja.py`（NFKC・約物空白除去・文字単位 Levenshtein / 参照長）で CER を算出する。CI 実測のベースライン（ubuntu-22.04）は次の通り。

| モデル | 平均CER | 位置づけ |
|---|---|---|
| ggml-tiny | 56.9% | 日本語で base に完全劣位 |
| ggml-base | 44.0% | 頑健な既定 |
| kotoba-whisper v2.0 q5 | 38.3% | 日本語推奨（素材により幻覚で悪化しうる） |

ここで**この指標の限界を正直に明示する**ことが、玄人読者に対する誠実さだと考える。

- **これは絶対精度ではない。** 原文へのルビ混入で絶対CERは悲観側に振れる。ベンチ文書自身が「絶対精度の主張には使うな」と明記している。
- **N=3 の相対/回帰指標である。** 使い道は「モデル間の相対比較」と「回帰ゲート」に限る。回帰基準は `docs/perf/ja-cer-baseline.json`（margin=5pt）で管理する。
- **平均は分布を隠す。** kotoba-q5 はサンプルによっては幻覚（反復・脱落）で base より悪化する。だから既定を kotoba-q5 に寄せつつ、頑健な base をフォールバックに残す設計にした。

この「測っていないものを明示する」姿勢が、そのまま製品判断に接続する。[ADR-0022](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0022-model-catalog-curation.md) は、実測で tiny が日本語劣位だと分かっても**モデルを削除しない**。tiny は英語・下書き・低スペック機での最速選択肢として価値が残るからだ。代わりにモデルカタログ（`src-tauri/src/model.rs`・全6モデル）を用途優先で並べ替える（`base → kotoba-q5 → kotoba → small → medium → tiny`）。そして UI ラベルに「tiny=日本語は低精度で非推奨」と言語別の適否を明示する。**「削る」のではなく「正しい選択肢へ導く」**——核心課題「リッチすぎると簡便でなくなる」への一貫した回答である。

RTF（実時間比）についても条件を厳密に添えておく。CI 実測で **RTF 0.857**（実時間以内）を確認している。ただしこれは限定条件の値だ。内訳は **ggml-tiny・Linux x64・決定的AVX2ベースライン・音源長18.88s・GitHub Actions（ubuntu-22.04）**（[perf/baseline.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/perf/baseline.md)）。「アプリ全般の速度」に一般化してはいけない。同じベンチのピークRSSにも但し書きが要る。あの値は `cargo test` ハーネス全体（テストプロセス＋ビルド成果物＋whisperコンテキスト）のものであって、配布アプリのアイドルメモリではない。この注記もドキュメント側に残している。**「測った条件の外へ数字を持ち出さない」**規律は、記事とコードのどちらにも等しく効かせる。

## 5. プライバシー設計：既定を「正直」にする

差別化4軸の1つがローカルプライバシーだが、前述の通りこれ単体はコモディティだ。だからこそ**既定と表示の誠実さ**で差をつける。ここには失敗と是正の履歴がある。

初期 UI は「話した言葉はこの端末から出ません」と表示していた。ところが当時の既定整形はクラウド（BYO鍵）だったため、これは**実態に反する過剰主張**だった。[ADR-0019](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0019-privacy-indicator-and-offline-mode.md) はこれを是正した。**プライバシー状態インジケータ**を設定先頭に常時表示し、`isFullyLocal = (整形=ローカルOllama) かつ (STT=ローカルwhisper)` のときだけ「オンデバイス完結」と表示する。それ以外は「クラウド送信あり」と正直に出す。加えてクラウド時のみ「オフラインにする」ワンクリック導線（provider=ollama / stt=local へ即切替）を用意した。ここでも「トグルを1つに留め、選択肢を増やさない」核心課題が効いている。

その後 [ADR-0021](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0021-local-first-defaults.md) で Decider が既定自体をローカルファーストへ動かした。整形の既定を Ollama（ローカル）に変更し、日本語UIユーザーの whisper 既定モデルを kotoba-q5 にした。トレードオフも ADR に正直に書いてある。ローカル整形は Ollama 稼働が前提で、未導入だと初回整形が失敗する。だから握り潰さず明確な i18n エラー（`E_REFINE_*`）で通知し、オンボーディングに導線を添え、クラウドへの即切替も残す。**リスクを「削る」のではなく「設計・段階実装で対処する」**（[ADR-0006](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0006-scope-completeness-policy.md) のスコープ完全性ポリシー）という原則が、ここでも貫かれている。

非機能面の裏付けも列挙しておく（[non-functional-requirements.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/non-functional-requirements.md)）。テレメトリ・解析は持たない（[ADR-0020](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0020-metrics-and-telemetry-stance.md)）。クラウド連携はオプトインのみ。巨大入力には 500MB のサイズ上限ガード（`MAX_INPUT_BYTES`）を置き、対応音声は7形式（mp3/wav/m4a/flac/ogg/opus/aac）。UI は4言語（ja/en/zh/es）を出荷済み。品質ゲートは CI 実測に基づき、フロントは全指標80%のカバレッジ閾値、Rust も lines 80% を下限（実測は約86.9%）に設定している。テストは Rust 側で約125関数、フロントで27ファイルが動く。これらは「価値の本体でない部分こそ機械で守る」という配分の表れである。

## 6. 図をCIで生成し、絶対URLで貼る（記事の再現性・エバーグリーン性）

技術記事の腐りやすさは、①図が本文と乖離する、②相対パス画像が転載でリンク切れになる、の2点に集約される。QuickScribe ではこれを仕組みで潰した。

- **図は Mermaid を単一ソースにし、CI で SVG/PNG に書き出す**（`scripts/render-diagrams.mjs` + `@mermaid-js/mermaid-cli`）。`docs/design.md` の ```mermaid``` ブロックがそのまま `docs/assets/diagrams/design-1.png` / `design-2.png` になる。図の更新は本文（design.md）の更新と同期する。
- **記事内の画像は絶対URL**（`https://raw.githubusercontent.com/Takenori-Kusaka/QuickScribe/main/...`）で参照する。相対パスは Qiita 転載でリンク切れになるためだ。この方針は記事テンプレと運用ポリシーに明文化した。
- 校正は **textlint（preset-ja-technical-writing）**を CI で回し、記事変更時にゼロエラーを必須にする。
- Zenn/Qiita の front-matter とリンク形式の差は、**単一ソース（Zenn記事）→ Qiita（qiita-cli 形式）への機械変換**（`scripts/zenn-to-qiita.mjs`）で吸収する。Zenn は GitHub 連携で `articles/` を自動同期するため push 型 CI は要さない。Qiita は qiita-cli の GitHub Action で公開できる形を用意し、**実際の公開トリガはメジャーバージョンアップ時のみ**という運用にした（詳細は [記事公開・再デプロイポリシー](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/process/article-publishing-policy.md)）。

なお、より重厚な C4/Structurizr DSL 生成は将来検討とし、まずは Mermaid→SVG/PNG の軽量経路で「CI 生成・所定配置・絶対パス」という要件を満たすことを優先した。ここでも「リッチすぎると簡便でなくなる」を適用している。

## まとめ

QuickScribe の設計は、一貫して1つの問いから導かれている。**「価値の本体はどこか、そして本体でない部分をどれだけ機械化・抽象化・差し替え可能にできるか」**。

- 文字起こしは `TranscriptionEngine` の背後に隠し、精度は**相対的な回帰指標**として CI で監視する。
- 整形の知性を `FormattingEngine` の Strategy として一級化し、プライバシー方針の変更にアーキテクチャを縛られないようにする。
- 技術選定は「不採用の理由」を ADR に残し、説明可能性を担保する。
- 数値は必ず**計測条件の内側**でだけ語り、「何を測っていないか」を明示する。

「精度で殴らない音声アプリ」という一見不利な立ち位置は、抽象境界と正直なベンチマークに支えられて初めて成立する。もし同種のプロダクトを設計するなら、最初に決めるべきは Whisper のモデルではなく、**自分の製品にとってのコモディティ境界をどこに引くか**だと思う。

---

**一次情報・関連ADR**

- 設計: [design.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/design.md) / [vision.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/vision.md) / [ADR索引](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/README.md)
- 実測: [perf/baseline.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/perf/baseline.md) / [non-functional-requirements.md](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/non-functional-requirements.md)
- 判断: ADR-0002 / 0003 / 0005 / 0006 / 0019 / 0020 / 0021 / 0022
