# QuickScribe 引継ぎ資料（HANDOFF）

> 最終更新: 2026-06-29（v1.0.0 リリース準備プログラム進行中・i18n が現在の主作業）。
> **新セッションは本書を最初に読んで続行する。** 応答は日本語。コミット末尾に
> `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。
> 関連メモリ: `v1-0-0-program` / `signpath-signing-status`（recall時に参照）。

---

## 0. 最重要の運用ルール（必読・厳守）

- **main はブランチ保護済み＝直push不可。全変更はPR経由**（docs含む）。
  - 必須ステータスチェック: `build (ubuntu-22.04)` / `build (windows-latest)` / `lint (frontend)` / `e2e (実起動 / Linux)` / `transcribe (決定論テスト / Linux)` / `CodeQL (JS/TS)`。
  - 承認数 **0**（自分のPRをマージ可）。**linear history・会話解決必須・force push禁止**。strict（PRは最新mainに追従要）。
- **ローカルに MSVC Build Tools 無し → Rust(`src-tauri`)はビルド/`cargo test`不可。Rust変更はCI検証依存**。導入済: cargo/rustc(stable-msvc)・CMake・LLVM・ffmpeg・Playwright(chromium)。
- **フロントはローカル検証可（毎PR前に回す）**:
  - `npm run check`(svelte-check) / `npm run lint`(eslint) / `npm run format:check`(prettier) / `npm test`(vitest) / `npm run coverage`(80%ゲート稼働) / `npm run screenshots`(Playwright・描画/設定/英語/a11yを検証)。
- **PR運用フロー**: `git checkout -b feat/x main` → 実装 → `npx prettier --write <files>` → 上記検証 → commit/push → `gh pr create` → CI監視 → `gh pr merge <n> --squash --delete-branch`。
  - **マージ前に必ず mergeStateStatus を確認**（`gh pr view <n> --json mergeStateStatus`）。`BEHIND` の場合は strict保護でブロックされる → ブランチを `git rebase origin/main` → `git push -f` → CI再green後にマージ。
  - **e2e(実起動)はオーディオ依存でフレーク** → 失敗時 `gh run rerun <runid> --failed`（恒久対策=[#412](https://github.com/Takenori-Kusaka/QuickScribe/issues/412)）。
  - **バックグラウンド監視タスクは環境に kill されることがある** → killされたら `gh pr checks <n>` で直接確認。フォアグラウンドの `for`ループ + `sleep 55` でポーリング+マージが確実。

---

## 1. リリース手順（重要な落とし穴あり）

**配信済み: v0.6.3 → v0.6.4 → v0.7.0 → v0.8.0**（最新 = **v0.8.0**）。リリースは release-please + タグ駆動 release.yml。

### ★ release-please の GITHUB_TOKEN 制約（[#427](https://github.com/Takenori-Kusaka/QuickScribe/issues/427) 恒久対策待ち）
release-please は GITHUB_TOKEN で Release PR/タグを作るため、**GitHub のループ防止により他ワークフローを起動しない**。毎回これを手動補助する必要がある:
1. **Release PR（`chore(main): release X.Y.Z`）には必須CIが走らない** → strict保護でブロック → **`gh pr merge <n> --squash --admin --delete-branch`（管理者マージ）** で通す（中身はCHANGELOG+manifestのみでコード無し＝安全）。
2. **マージで作られるタグ push が release.yml を起動しない** → **ユーザートークンでタグを押し直す**:
   `git fetch --tags && git push origin :refs/tags/vX.Y.Z && git push origin refs/tags/vX.Y.Z` → これで release.yml が起動しバイナリ生成。
3. **tauri-action が release-please のノートを定型文で上書きする** → ビルド前に `gh release view vX.Y.Z --json body -q .body > notes.md` で退避し、**ビルド完了後に `gh release edit vX.Y.Z --notes-file notes.md` で復元**。
4. GitHub Actions のランナーは混雑時に長く queued になる（特に Windows/ARM64）。1リリースのバイナリ全揃いに 20〜40分かかることがある。
- **昇格規則**: `release-please-config.json` は `bump-minor-pre-major: true` のみ（`bump-patch-for-minor-pre-major` は除去済）＝0.x で **feat→minor / fix→patch**。特定版へ固定したいときは config 変更 or `Release-As:` フッタ。
- 恒久対策（[#427](https://github.com/Takenori-Kusaka/QuickScribe/issues/427)）: release-please に **PAT/GitHub App トークン** を渡せば 1〜2 が自動化。3 は release.yml の `releaseBody` 調整で解消（ただしタグ時点のworkflowが使われるので既存タグには効かない）。

---

## 2. v1.0.0 プログラム（[Epic #389](https://github.com/Takenori-Kusaka/QuickScribe/issues/389) / 監査=`docs/planning/v1.0.0-readiness.md`）

カットライン= **A案（フル達成後にv1.0.0）** をユーザー承認済み。17観点を個別issue化（**#390〜#406**）。取り残しTask 90件クローズ済。

### ✅ 完了（mainマージ済）
- **基盤**: カバレッジ計測+80%ゲート(#402) / ESLint+Prettier CIゲート(#392/#393) / SSOT `src/lib/constants.ts`(#401 Phase0)。
- **公開品質**: 内部ID"S2.2"除去＋`errorText`(#398) / a11y モーダルdialog・フォーカストラップ・Esc・コントラスト・reduced-motion(#395) / 入力サイズ上限ガード＋対応形式通知(#397)。
- **ガバナンス(#406)**: ブランチ保護・Wiki無効・Dependabot security updates・PVR・Secret scanning・CODEOWNERS・SUPPORT.md・ラベル。
- **リリース運用(#400)**: release-please 導入・実地検証済（3リリース配信）。CHANGELOG.md・`.github/release.yml`・Actions「PR作成許可」有効化。
- **ドキュメント(#390)**: ADR索引(0010-0017)・ADR-0005=Accepted・署名方針をSignPath無料に単一化・vision現行化・`docs/design.md`(アーキ/データフロー図)・`docs/non-functional-requirements.md` 新設。
- **ライセンス(#394)**: `THIRD-PARTY-NOTICES-frontend.md` 生成(`npm run licenses`・`scripts/generate-frontend-notices.cjs`)＋CIドリフト検査＋README導線。**プラットフォーム固有バイナリ(@esbuild等)は除外しOS間決定的に**（svelte-i18n導入時に発覚し修正済）。
- **競合分析(#399)**: `docs/research/competitive-landscape.md`（一次情報）。
- **#402 カバレッジ**: shortcut/entry/provider-config/refine-args/model-cache/discovery を `src/lib` へ抽出。**lib 82.7%・CIゲート稼働**(vitest thresholds lines/statements 75・functions/branches 85)。テスト 14→**61**＋Playwright 6。残=App.svelte本体は @testing-library/svelte 等のコンポーネントテスト基盤が必要。
- **#403 perf**: `.github/workflows/perf.yml`(workflow_dispatch/release)で RTF/RSS 計測→アーティファクト。初回実測 **RTF=0.857**(目標<1.0達成) を `docs/perf/baseline.md` 記録。
- **設定UX(#404)**: 誤分類修正＋**設定グループを native `<details>` アコーディオン化**（折りたたみ・キーボード/SR対応。advanced既定折りたたみ）。残=頻度メジャー順の上位カテゴリ再編（DOM並べ替え・別途）。
- **ヘッダUX**: 「ジャーナル」を**アイコンのみ＋ツールチップ**化（狭い窓でタイトルと重なる問題を解消・歯車と統一）。

### 🌐 #401 i18n（**現在の主作業・実用段階**）
- 実装: `svelte-i18n`（同期init `src/lib/i18n/index.ts`）。`ja.json`＋`en.json`。`$_('key')` で参照（markup/script両方可）。ICU補間 `$_('k',{values:{x}})`。
- **起動ロケール = 保存設定(localStorage `locale`) > OSシステム言語(`getLocaleFromNavigator`) > ja**。対応外は ja へクランプ。設定>アプリ全般>言語 で手動切替（ja/en）。
- **キー化済(8スライス)**: メイン画面/ヘッダ/結果カード/ジャーナルパネル/設定の見出し・ラベル・選択肢/エラー文言17種/AWS・STT詳細・主要tip。**メイン体験は日英完全バイリンガル**。
- **⚠ ルール: 新キーは ja.json と en.json の両方に追加**（漏れると en で ja にフォールバック）。
- **残り（継続キー化の長い尾）**:
  - **複雑な inline-HTML tip 約9種**（ollama注記/momentary/録音ソース/STT警告/local whisper/出力形式/カスタム/タスクバー/自動起動）。`<strong>`/`<code>` 埋込のため ICU化に工夫が要る（{@html}はeslint `svelte/no-at-html-tags`=errorで不可 → 構造を分割するか emphasis を諦める）。
  - 用語補正UI・モデルhint等の細部。
  - **Rustエラーのコード化（Phase2・大規模）**: バックエンドの日本語エラー文字列を識別子化し、フロントの `errors.*` カタログで翻訳する。`validateRefineConfig`(provider-config.ts)の返すテキストも同様にコード化が必要（現状 `cfgErr` は ja のまま）。
  - **zh / es カタログ**（翻訳追加）。
- **i18n作業の進め方（推奨手順）**: ①対象文字列を grep → ②ja.json/en.json に同一キーを追加 → ③App.svelte を `$_()` に置換（多数なら `python3` のバッチ置換が速い） → ④`prettier --write` → ⑤`npm run check`/`lint`/`screenshots`（en/ja/設定スクショで描画確認） → ⑥PR。

### ⏳ その他の残り
- #402後続(App.svelteのコンポーネントテスト) / #403後続(日本語精度#26・起動時間・回帰ゲート)。
- #397後続(初回オンボーディング) / #404後続(頻度順カテゴリ再編)。
- #391残: **制限的CSP**・whisperモデル**SHA256検証**・devDeps更新。
- #392/#393残: Rust clippy/rustfmt CI・missing_docs・refine/stt重複排除・App.svelte分割・Provider enum・entry.rs抽出・TSDoc。

---

## 3. ユーザーの判断待ち（保留中）
- **[#427](https://github.com/Takenori-Kusaka/QuickScribe/issues/427) release-please 完全自動化**: PAT/App トークン登録（メンテナ作業）。§1 参照。
- **Dependabot PR群**（#416 vite/vitestメジャー・#360-365 actions/cargo/tauri-js）: 古い基底で要リベース。cpalメジャー(#364)等は#391で制御更新推奨。**devDeps脆弱性(critical含む12件)は全てランタイム無影響**（配布バイナリに含まれない）。
- **SignPath署名**（[#386](https://github.com/Takenori-Kusaka/QuickScribe/pull/386) Draft）: 審査待ち。承認メール(@signpath.org)到着→実値化→テストタグ検証。署名順序の論点（Authenticode→updater署名再生成）注意。memory `signpath-signing-status`。
- **Sponsors**（[#405](https://github.com/Takenori-Kusaka/QuickScribe/issues/405)）: FUNDING.yml設定済、アカウント側Sponsors登録の確認のみ。

---

## 4. コード地図
- **バックエンド `src-tauri/src/`**: `lib.rs`(コマンド層・保存ドメイン build_document/save_document・keyring・`check_input_size`) / `record.rs`(cpal+wasapi, SourceKind) / `stt.rs`(`TranscriptionEngine`抽象+decode_to_16k_mono) / `model.rs`(whisperカタログ) / `refine.rs`(`FormattingEngine`抽象・`<journal>`境界抽出) / `vault.rs`(読み) / `aws_sign.rs`(SigV4) / `audio_save.rs` / `taskbar*.rs`(Win UI)。**※Rust文字列のエラーは多数が日本語で、i18n Phase2でコード化対象**。
- **フロント `src/`**: `App.svelte`(巨大・状態とUI・i18n化進行中) / `lib/`= `note.ts` `corrections.ts` `constants.ts`(SSOT) `errors.ts`(errorText) `a11y.ts`(modal action) `shortcut.ts` `entry.ts` `provider-config.ts` `refine-args.ts` `model-cache.ts` `discovery.ts` `i18n/`(index.ts/ja.json/en.json)。
- **テスト/検証**: `src/lib/*.test.ts`(vitest 61) / `e2e/screenshots.spec.ts`(Playwright: メイン/英語/ジャーナル/設定スクショ) / `e2e/a11y.spec.ts`(dialog/Esc) / `e2e/mocks/`(Tauri IPCモック) / `e2e/vite.screenshot.config.ts`。
- **CI `.github/workflows/`**: ci(build/lint/e2e/transcribe+coverage) / security(CodeQL/cargo audit-deny/npm audit) / release(タグ駆動tauri-action) / release-please / screenshots / perf / pages / test-build。
- **設計**: `docs/design.md` / `docs/non-functional-requirements.md` / `docs/adr/`（0001-0017）/ `docs/planning/v1.0.0-readiness.md`。

---

## 5. プロダクトの芯（不変）
思考整理・自己理解のローカル完結ボイスジャーナル。コア価値=「ニュアンスを残しつつ思考整理する整形の知性」。差別化=用途特化/ニュアンス保持/ローカルプライバシー/物理ボタン統合。スコープ規律=[ADR-0006](docs/adr/0006-scope-completeness-policy.md)（独断のスコープ縮小禁止・リスクは段階実装で吸収し全部やりきる）。必読: `CLAUDE.md` / `docs/vision.md`。

## 6. 次セッションの即・着手候補
1. **i18n継続**（推奨・現在の主作業）: §2の「残り」を上から。まず複雑inline-HTML tipのキー化（構造分割で `<strong>` を保持 or 諦める判断）、次に用語補正UI、その後 **Rustエラーのコード化(Phase2)**、最後に zh/es。
2. v1.0前必須の残: #391(CSP/モデルSHA256)・#404後続(カテゴリ再編)・#397後続(オンボ)。
3. ユーザー判断待ち項目（§3）の消化。
