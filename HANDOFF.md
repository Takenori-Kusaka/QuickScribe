# QuickScribe 引継ぎ資料（HANDOFF）

> 最終更新: 2026-06-28（v1.0.0 リリース準備プログラム進行中）。新セッションはこれを読んで続行する。
> 応答は日本語。コミット末尾に `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`。
> malformed対策: 応答はツール呼び出しから開始（前置きテキスト禁止）。2回続いたら新セッションへ。

## 0. 最重要の運用ルール（変更点）
- **main はブランチ保護済み＝直push不可。全ての変更はPR経由**（docs含む）。必須チェック: build(ubuntu/win)/lint(frontend)/e2e/transcribe/CodeQL(JS-TS)。承認数0（自分のPRをマージ可）。linear history・会話解決必須・force push禁止。
- **ローカルに MSVC Build Tools 無し → Rust(`src-tauri`)はビルド/`cargo test`不可**。Rust変更はCI検証依存。導入済: cargo/rustc(stable-msvc)・CMake・LLVM・ffmpeg・Playwright(chromium)。
- フロントはローカル検証可: `npm run check`(svelte-check) / `npm run lint`(eslint) / `npm run format:check` / `npm test`(vitest) / `npm run coverage` / `npm run screenshots`(Playwright)。
- PR運用: branch → push → CI監視(バックグラウンドで pending=0までループ) → `gh pr merge <n> --squash --delete-branch`。**e2e(実起動)はオーディオ依存でフレークあり** → 失敗時 `gh run rerun <id> --failed`（[#412](https://github.com/Takenori-Kusaka/QuickScribe/issues/412)で恒久対策予定）。

## 1. v1.0.0 プログラム（[Epic #389](https://github.com/Takenori-Kusaka/QuickScribe/issues/389) / 監査=`docs/planning/v1.0.0-readiness.md`）
カットライン= **A案（フル達成後にv1.0.0）** をユーザー承認済み。17観点を個別issue化（#390-406）。取り残しTask 90件はクローズ済（オープン issue 192→大幅減）。

### ✅ 完了（mainマージ済）
- 基盤: カバレッジ計測(#402 Phase1) / ESLint+Prettierゲート(#392/#393) / SSOT `src/lib/constants.ts`(#401 Phase0)
- 公開品質: 内部ID"S2.2"除去＋エラー文言`errorText`(#398) / a11y モーダルdialog・フォーカストラップ・Esc・コントラスト・reduced-motion(#395) / 入力サイズ上限ガード＋対応形式通知(#397)
- ガバナンス(#406): ブランチ保護・Wiki無効・Dependabot security updates・PVR・Secret scanning・CODEOWNERS・SUPPORT.md・ラベル
- リリース運用(#400): **release-please 導入・実地検証済**（Release PR自動生成を確認）。CHANGELOG.md・`.github/release.yml`。Actions の「PR作成許可」も有効化済。
- ドキュメント(#390): ADR索引(0010-0017追記)・ADR-0005=Accepted・署名方針をSignPath無料に単一化・vision現行化・**design.md**(アーキ/データフロー図)・**non-functional-requirements.md** 新設。
- ライセンス(#394): `THIRD-PARTY-NOTICES-frontend.md`生成(`npm run licenses`)＋CIドリフト検査＋README導線。
- 設定UX(#404): タスクバー/自動起動の誤分類を「アプリ全般」へ修正。
- リリース(#400): **release-please→v0.6.4 を実配信**(ノート＋7プラットフォームバイナリ)。
- ライセンス/競合: npm帰属(#394)・競合分析(#399)済。
- **#402 カバレッジ(進行中・lib抽出群)**: shortcut/entry/provider-config/refine-args/model-cache/discovery を `src/lib` へ抽出。**lib カバレッジ 82.7%・CIゲート稼働**(vitest thresholds lines75/branches85)。テスト 14→**61**＋Playwright 4。残=App.svelte本体はコンポーネントテスト基盤(@testing-library/svelte)が必要。
- **#403 perfベンチ(動作確認済)**: `.github/workflows/perf.yml`(workflow_dispatch/release)で RTF/RSS 計測→アーティファクト。初回実測 **RTF=0.857**(<1.0達成) を `docs/perf/baseline.md` 記録。

### ⏳ 残り（A案フル達成へ）
- **最大の未着手=[#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) i18n多言語(en/zh/es/ja)**: 基盤未導入・日本語ハードコード約300-350文字列。Phase0(SSOT)済→i18n基盤+ja化→**Rustエラーのコード化**→en→zh/es。13-20人日。**専用セッション推奨**。
- #402後続(App.svelte本体のコンポーネントテスト) / #403後続(日本語精度#26・起動時間・回帰ゲート)。
- **中規模**: #404後続(設定アコーディオン) / #397後続(初回オンボ) / #391残(制限的CSP・モデルSHA256・devDeps更新)。
- **コード品質後続**: #392/#393残(Rust clippy/rustfmt CI・missing_docs・refine/stt重複排除・App.svelte分割・Provider enum・entry.rs・TSDoc)。

## 2. ユーザーの判断待ち（保留中）
- **release-please 完全自動化=[#427](https://github.com/Takenori-Kusaka/QuickScribe/issues/427)**: release-please は GITHUB_TOKEN のため Release PR にCIが走らず・タグで release.yml が起動しない。v0.6.4 は**管理者マージ＋タグ再push＋ノート復元**で手動補助した。**恒久対策=PAT/App トークンを release-please に渡す**(ユーザー作業)。また tauri-action が release-please のノートを上書きするため、release.yml の releaseBody 調整も要対応。
- **Dependabot PR群**（#416 vite/vitestメジャー・#360-365 actions/cargo/tauri-js）: 古い基底で要リベース。Actions更新(#360/361/362)はrebase要求済。cpalメジャー(#364)等は#391で制御更新推奨。devDeps脆弱性(critical含む)は全てランタイム無影響。
- **SignPath署名**（[#386](https://github.com/Takenori-Kusaka/QuickScribe/pull/386) Draftに組込手順）: 審査待ち。承認メール(@signpath.org)到着→実値化→テストタグ検証。詳細メモ=memory `signpath-signing-status`。署名順序の論点(Authenticode→updater署名再生成)注意。
- **Sponsors**（[#405](https://github.com/Takenori-Kusaka/QuickScribe/issues/405)）: FUNDING.yml設定済、アカウント側Sponsors登録の確認のみ（メンテナ作業）。

## 3. コード地図（要点）
- バックエンド `src-tauri/src/`: `lib.rs`(コマンド層・保存ドメイン build_document/save_document・keyring・`check_input_size`) / `record.rs`(cpal+wasapi, SourceKind) / `stt.rs`(`TranscriptionEngine`抽象+decode_to_16k_mono) / `model.rs`(whisperカタログ) / `refine.rs`(`FormattingEngine`抽象・`<journal>`境界抽出) / `vault.rs`(読み) / `aws_sign.rs` / `audio_save.rs` / `taskbar*.rs`(Win UI)。
- フロント `src/`: `App.svelte`(巨大・状態とUI) / `lib/`= `note.ts` `corrections.ts` `constants.ts`(SSOT) `errors.ts`(errorText) `a11y.ts`(modal action)。
- 設計詳細: `docs/design.md` / NFR: `docs/non-functional-requirements.md` / ADR: `docs/adr/`。
- CI: `.github/workflows/` = ci(build/lint/e2e/transcribe+coverage) / security(CodeQL/cargo audit-deny/npm audit) / release(タグ駆動tauri-action) / release-please / screenshots / pages / test-build。

## 4. プロダクトの芯（不変）
思考整理・自己理解のローカル完結ボイスジャーナル。コア価値=「ニュアンスを残しつつ思考整理する整形の知性」。差別化=用途特化/ニュアンス保持/ローカルプライバシー/物理ボタン統合。スコープ規律=[ADR-0006](docs/adr/0006-scope-completeness-policy.md)（独断縮小禁止・段階実装で全部やりきる）。必読: `CLAUDE.md` / `docs/vision.md`。
