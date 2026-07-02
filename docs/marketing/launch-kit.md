# QuickScribe ローンチキット（#59 S9.5）

> このドキュメントは**投稿用アセット一式**。実際の投稿はメンテナ本人が各プラットフォームの
> アカウントで行う（ここが唯一の人手ステップ）。文面は「誠実・非誇張」を原則とする
> （コア価値=思考整理/ニュアンス保持/ローカルプライバシーと整合。ADR-0004）。
>
> **原則**: 各コミュニティのルールを尊重し、スパム的な連投をしない。1コミュニティ1投稿。
> 反応には丁寧に返信する。売り込みでなく「作った理由と学び」を語る。

---

## 0. 事前チェックリスト（投稿前 T-1日）

- [ ] 最新リリースが公開済みで、各OSのインストーラがReleasesにある
- [ ] README のデモGIF・スクリーンショットが最新（`npm run screenshots` → `npm run demo:gif`）
- [ ] ドキュメントサイトが公開・リンク健全（Download / Privacy / About）
- [ ] 「未署名（SmartScreen）」の但し書きがREADME/サイトにある（現状Windows未署名）
- [ ] Issue テンプレ・Discussions・CODE_OF_CONDUCT・SECURITY.md が整備済み
- [ ] プライバシーポリシーが実装と一致（テレメトリなし / ADR-0020）
- [ ] 返信できる時間帯に投稿する（投稿後 数時間はコメント対応する前提）

## 1. 一言サマリ（共通・使い回し）

- **JP**: 話した内容を、ニュアンスを残したまま整形して思考を整理する、ローカル完結のボイスジャーナル。
- **EN**: A local-first voice journal that keeps the nuance of what you said and helps you organize your thinking — not just transcription.

---

## 2. Hacker News（Show HN）

**Title（英語・80字以内・誇張しない）:**
```
Show HN: QuickScribe – A local-first voice journal that preserves nuance
```

**本文（最初のコメントとして投稿推奨）:**
```
I built QuickScribe because most voice tools optimize for transcription accuracy,
but what I actually wanted was to think out loud and later organize my thoughts —
without losing the nuance, hesitation, and tone of how I said it.

It's a desktop app (Tauri + Rust + Svelte). Core ideas:
- Refinement over transcription: it reshapes speech into readable notes while
  keeping your nuance (verbatim / summary / brainstorm styles you can switch).
- Local-first & private by default: recording and local transcription
  (whisper.cpp) stay on device. Cloud STT/LLM are opt-in with your own keys.
  No telemetry.
- Physical-button flow: global hotkey / footswitch to start capturing instantly.
- Your journal is plain Markdown/txt on disk (works with Obsidian etc.).

It's open source (MIT). Windows binaries are currently unsigned (SmartScreen may
warn) — code signing via SignPath is in progress. Feedback welcome, especially on
the "refinement" quality vs. plain transcription.

Repo: https://github.com/Takenori-Kusaka/QuickScribe
```

**Show HN の注意**: タイトルに「Show HN:」必須。過度な宣伝語を避け、技術的に正直に。
投稿後はコメントに素早く・誠実に返信する。

---

## 3. Reddit

**候補サブレディット（ルール確認必須・1つずつ、間隔を空けて）:**
- r/ObsidianMD（Markdown保存・外部ツール連携の角度）
- r/selfhosted / r/privacy（ローカル完結・プライバシーの角度）
- r/journaling / r/productivity（思考整理・習慣の角度）
- r/opensource（OSS紹介）

**投稿タイトル例（角度別）:**
- (privacy) `I built a local-first voice journal — audio & transcription stay on device (open source)`
- (obsidian) `Voice journal that saves plain Markdown you can open in Obsidian — keeps the nuance, not just the transcript`

**本文テンプレ:**
```
I've been building QuickScribe, a desktop voice journal focused on *organizing
your thinking*, not just transcription. It keeps the nuance/tone of what you said
and reshapes it into readable notes (you can switch verbatim/summary/brainstorm).

Why it might fit here:
- Local-first: recording + local transcription (whisper.cpp) stay on device.
  Cloud is opt-in with your own API key. No telemetry.
- Your entries are plain Markdown/txt on disk.
- Global hotkey / footswitch to capture the moment a thought hits.

It's MIT-licensed. Windows builds are currently unsigned (signing in progress).
Happy to answer questions — and genuinely want feedback on the refinement quality.
Repo: https://github.com/Takenori-Kusaka/QuickScribe
```
**Reddit の注意**: 各サブのセルフプロモ規約を必ず読む。コミュニティに価値を返す姿勢で。
コメントで質問に答える。複数サブへ同文コピペ連投はしない（角度を変える）。

---

## 4. Product Hunt

- **Name**: QuickScribe
- **Tagline（60字以内）**: `Local-first voice journal that keeps your nuance`
- **Topics**: Productivity, Open Source, Privacy, Artificial Intelligence
- **Description**:
```
QuickScribe turns thinking out loud into organized notes — while keeping the
nuance, hesitation and tone of how you actually said it. Recording and local
transcription stay on your device (whisper.cpp); cloud STT/LLM are opt-in with
your own keys. Your journal is plain Markdown on disk. Open source (MIT).
```
- **Maker's first comment**:
```
Hi PH! I'm the maker. I built this because I wanted to *think out loud and
organize my thoughts* — not just get a transcript. The differentiator is the
"refinement" step that preserves nuance, plus local-first privacy and a physical
hotkey/footswitch flow. It's open source and I'd love your feedback on where the
refinement helps (or doesn't). Note: Windows builds are currently unsigned;
signing is in progress.
```
**PH の注意**: 平日の PT 12:01 AM 公開が定石。ハンター不要（自分でも可）。当日はコメント対応。

---

## 5. LinkedIn（Xの代替 / 本人アカウント）

**投稿（一人称の物語・ハッシュタグ控えめ）:**
```
「文字起こし」より「思考の整理」が欲しかった。

生成AIで開発を続ける中で、頭の中を声に出して吐き出し、あとで見返しながら考えを
整理したい場面が増えました。でも既存ツールの多くは"精度"に最適化されていて、
自分の言い淀みやニュアンスごと整理してくれるものがなかった。

そこで QuickScribe を作りました。ローカル完結のボイスジャーナルで、
・話したニュアンスを残したまま整形（逐語/要約/ブレストを切替）
・録音とローカル文字起こしは端末内、クラウドは任意（自分の鍵）、テレメトリなし
・物理ボタン/ホットキーで思いついた瞬間に記録
・記録はプレーンなMarkdown（Obsidian等でそのまま開ける）

オープンソース(MIT)です。触ってみた感想、特に"整形"の質についてのフィードバックを
もらえたら嬉しいです。
👉 https://github.com/Takenori-Kusaka/QuickScribe

#OpenSource #Privacy #Productivity #音声 #ジャーナリング
```

---

## 6. ローンチ48hの進め方

1. **T0（平日朝・自分が対応可能な時間）**: Show HN 投稿 → 直後に最初のコメント（背景）。
2. **T0+2〜3h**: Product Hunt 公開（PT深夜に合わせるなら別日調整）。
3. **T0 当日〜翌日**: Reddit を角度別に1サブずつ（間隔を空けて）。
4. **LinkedIn**: 当日中に本人アカウントで。
5. **全期間**: コメント/Issue/Discussions に**丁寧かつ迅速に**返信。批判も感謝で受ける。
6. **計測**: 翌日 `npm run metrics` でDL数の変化を確認（ADR-0020）。

## 7. Do / Don't

- **Do**: 正直に（未署名・未完成部分も明記）。作った理由と学びを語る。フィードバックを歓迎。
- **Don't**: 誇張・比較でのdisり・同文の連投・自作自演の投票依頼。コミュニティ規約違反。

---

**残る唯一の人手ステップ**: 上記の各プラットフォームへ、本人アカウントで実際に投稿すること。
文面はこのキットをそのまま/微調整して使用可能。
