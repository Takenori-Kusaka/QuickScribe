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
- テスト: vitest 14→25＋Playwright 4(スクショ2＋a11y Esc 2)。新lib(constants/errors/a11y)カバレッジ良好。

### ⏳ 残り（A案フル達成へ）
- **大物（各々独立・数日規模・集中セッション向き）**:
  - [#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) **i18n多言語(en/zh/es/ja)**: 基盤未導入・日本語ハードコード約300-350文字列。Phase0(SSOT)済→i18n基盤+ja化→Rustエラーのコード化→en→zh/es。13-20人日。
  - [#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) **カバレッジ80%**: 計測基盤は導入済(Phase1)。本体=App.svelte(2500行)の純ロジックを`src/lib`へ抽出してテスト→80%ゲート。6-10人日。
  - [#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403) **perfベンチCI**: RTF/起動/メモリ/日本語精度([#26])・`perf.yml`(手動/メジャー)・回帰ゲート。7-11人日。
- **中規模**: [#399](https://github.com/Takenori-Kusaka/QuickScribe/issues/399) 競合分析(`docs/research/competitive-landscape.md`・vision:38リンク先) / #404後続(設定アコーディオン・頻度順) / #397後続(初回オンボーディング) / #391 残(制限的CSP・モデルSHA256・devDeps更新)。
- **コード品質後続**: #392/#393 残(Rust clippy/rustfmt CI・missing_docs・refine/stt重複排除・App.svelte分割・Provider enum・entry.rs・TSDoc)。

## 2. ユーザーの判断待ち（保留中）
- **Release PR [#418](https://github.com/Takenori-Kusaka/QuickScribe/pull/418)「chore(main): release 0.6.4」**: マージ＝v0.6.4リリース（release.ymlがバイナリ配信）。タイミングはユーザー判断。**次回リリース時に tauri-action が release-please のノートを上書きしないか要確認**（上書き時は release.yml の releaseBody を一行調整）。
- **Dependabot PR群**（#416 vite/vitest等・#360-365 actions/cargo/tauri-js）: 順次レビュー・マージ可。devDeps脆弱性(critical含む)は全てランタイム無影響。
- **SignPath署名**（[#386](https://github.com/Takenori-Kusaka/QuickScribe/pull/386) Draftに組込手順）: 審査待ち。承認メール(@signpath.org)到着→実値化→テストタグ検証。詳細メモ=memory `signpath-signing-status`。署名順序の論点(Authenticode→updater署名再生成)注意。
- **Sponsors**（[#405](https://github.com/Takenori-Kusaka/QuickScribe/issues/405)）: FUNDING.yml設定済、アカウント側Sponsors登録の確認のみ（メンテナ作業）。

## 3. コード地図（要点）
- バックエンド `src-tauri/src/`: `lib.rs`(コマンド層・保存ドメイン build_document/save_document・keyring・`check_input_size`) / `record.rs`(cpal+wasapi, SourceKind) / `stt.rs`(`TranscriptionEngine`抽象+decode_to_16k_mono) / `model.rs`(whisperカタログ) / `refine.rs`(`FormattingEngine`抽象・`<journal>`境界抽出) / `vault.rs`(読み) / `aws_sign.rs` / `audio_save.rs` / `taskbar*.rs`(Win UI)。
- フロント `src/`: `App.svelte`(巨大・状態とUI) / `lib/`= `note.ts` `corrections.ts` `constants.ts`(SSOT) `errors.ts`(errorText) `a11y.ts`(modal action)。
- 設計詳細: `docs/design.md` / NFR: `docs/non-functional-requirements.md` / ADR: `docs/adr/`。
- CI: `.github/workflows/` = ci(build/lint/e2e/transcribe+coverage) / security(CodeQL/cargo audit-deny/npm audit) / release(タグ駆動tauri-action) / release-please / screenshots / pages / test-build。

## 4. プロダクトの芯（不変）
思考整理・自己理解のローカル完結ボイスジャーナル。コア価値=「ニュアンスを残しつつ思考整理する整形の知性」。差別化=用途特化/ニュアンス保持/ローカルプライバシー/物理ボタン統合。スコープ規律=[ADR-0006](docs/adr/0006-scope-completeness-policy.md)（独断縮小禁止・段階実装で全部やりきる）。必読: `CLAUDE.md` / `docs/vision.md`。
