# CI ゲート定義およびローカル再現手順 (Issue #684 / S7.7)

> Status: Reference (2026-08-17)
> 関連ポリシー: [ADR-0006](../adr/0006-scope-completeness-policy.md) (スコープ完全性), [ADR-0034](../adr/0034-dependency-vulnerability-release-criteria.md) (依存脆弱性基準)

## 概要

開発機（ローカル環境）ではすべての検証が通っていたにもかかわらず、GitHub へのプッシュ後に CI（GitHub Actions）だけでテストや検査が失敗する事態（「ローカル green ⇆ CI red」）を防ぐため、リポジトリの全 CI ゲートを SSOT として定義し、ローカルでの確実な再現手順を文書化します。

---

## 1. CI ゲート一覧とローカル再現マトリクス (SSOT)

CI 上で実際に実行されている検証ゲート、およびそれらのローカル（Windows/Docker）における再現可否の一覧です。

| ジョブ名 (ワークフロー) | 検証内容 / コマンドの実体 | 開発機 (Win) | コンテナ (Linux) | 備考・実行環境の実体の版 |
| :--- | :--- | :---: | :---: | :--- |
| `lint` (ci.yml) | ESLint 構文/スタイルチェック | ✅ 回せる | ✅ 回せる | `node: v20.20.2`, `npm: 10.8.2`<br>`npm run lint` |
| `lint:css` (ci.yml) | Stylelint (ブランドカラー・生hex禁止) | ✅ 回せる | ✅ 回せる | `npm run lint:css` |
| `format:check` (ci.yml) | Prettier コードフォーマット検査 | ✅ 回せる | ✅ 回せる | `npm run format:check` |
| `docs:check` (ci.yml) | TSDoc (Typedoc警告のエラー化) | ✅ 回せる | ✅ 回せる | `npm run docs:check` |
| `licenses` (ci.yml) | フロントエンドライセンス notices 同期チェック | ✅ 回せる | ✅ 回せる | `npm run lint:ci` 内でチェック。<br>差分があれば失敗（AC2）。 |
| `check` (ci.yml) | Svelte-check 型/コンパイル検査 | ✅ 回せる | ✅ 回せる | `npm run check` |
| `test` (ci.yml) | フロントエンド単体テスト（Vitest） | ✅ 回せる | ✅ 回せる | `npm run test` |
| `eval-core` (ci.yml) | 日本語ASR評価コア単体テスト（Python） | ✅ 回せる | ✅ 回せる | `python: 3.11`<br>`cd scripts/asr_eval && python test_asr_eval.py` |
| `cargo-test` (ci.yml) | Rust バックエンド単体テスト（Cargo） | ✅ 回せる | ❌ 回せない | `rustc: stable`<br>Windows 上は `scripts/cargo-win.ps1 test --workspace` を推奨（MSVC/Clang依存のためコンテナ上は不可）。 |
| `e2e` (ci.yml) | WebdriverIO 実起動 E2E テスト | ⚠️ 部分的 | ❌ 回せない | GUI 依存、`webkit2gtk`、`xvfb`、ビルド済み Tauri バイナリを必要とするためコンテナでは不可。 |
| `npm-audit` (security.yml) | 本番依存関係の脆弱性検査 | ✅ 回せる | ✅ 回せる | `npm audit --audit-level=high --omit=dev` |
| `cargo-audit` (security.yml) | Rust 依存脆弱性データベース照合 | ❌ 回せない | ✅ 回せる | ローカルは `cargo-audit` のインストールが必要。CI は `cargo audit` を実行。 |
| `cargo-deny` (security.yml) | Rust 依存ライセンス / advisories 検査 | ❌ 回せない | ✅ 回せる | ローカルは `cargo-deny` のインストールが必要。CI は `cargo deny check` を実行。 |

---

## 2. フロントエンド検証のローカル一括実行

ライセンス notices の更新漏れ（`THIRD-PARTY-NOTICES-frontend.md` の再生成差分）を含め、フロントエンドのすべての静的解析ゲートをローカルで一括実行するための統合コマンドを用意しました。

```bash
# 以下のスクリプトは ESLint / Stylelint / Prettier / TSDoc / Licenses 生成 & 差分チェックを直列実行します。
npm run lint:ci
```
※コミット前にこのコマンドを実行することで、CI の `lint (frontend)` ジョブで赤くなることを 100% 回避できます。

---

## 3. Docker を用いた Linux/Node 20 環境での再現・検証手順

CI の Linux コンテナ環境と同等の条件下で依存解決やモジュール解決（ESM 読み込みエラー等）をデバッグする場合、以下の手順を実行します。

### 3.1. 依存解決・モジュール読み込み系ゲートの検証

コンテナ内の独立した環境で、パッケージのクリーンインストールと ESM 解決を検証します。

```powershell
# PowerShell から実行する場合 (コンテナとホストの node_modules 干渉を防ぐため、クリーンな一時領域で実行します)
mkdir -Force .scratch/reproduction-sandbox
Copy-Item package.json, package-lock.json -Destination .scratch/reproduction-sandbox/
cd .scratch/reproduction-sandbox

# CI と同一バージョンの Node.js 20.20.2 コンテナを起動し検証
docker run --rm -v "${PWD}:/w" -w /w node:20.20.2 sh -c '
  npm ci --ignore-scripts
  npm audit --audit-level=high --omit=dev
  npm ls brace-expansion minimatch
  node node_modules/@wdio/cli/bin/wdio.js --version
'

# 検証完了後に一時ディレクトリをクリーンアップ
cd ../..
Remove-Item -Recurse -Force .scratch/reproduction-sandbox
```

---

## 4. 既知の失敗例（RUSTSEC/ESM問題）の再現検証 (AC3)

本検証手順が本当に機能しているかを、PR #682 以前の「実際に失敗した過去のコミット」を用いて証明します。

### 4.1. 失敗する状態の再現（最小の破損コードでの実証）

```powershell
# 1. 破損している過去のコミット (ad16fc4) から package ファイルを抽出
mkdir -Force .scratch/broken-sandbox
cd .scratch/broken-sandbox
git show ad16fc4:package.json > package.json
git show ad16fc4:package-lock.json > package-lock.json

# 2. コンテナ内で実行
docker run --rm -v "${PWD}:/w" -w /w node:20.20.2 sh -c '
  npm ci --ignore-scripts
  node node_modules/@wdio/cli/bin/wdio.js --version
'
```

#### 期待される失敗出力（再現確認 ✅）
```text
file:///w/node_modules/minimatch/dist/esm/index.js:1
import expand from 'brace-expansion';
       ^^^^^^
SyntaxError: The requested module 'brace-expansion' does not provide an export named 'default'
```
このエラーは、当時の CI ジョブ（run 30228065759 / job 89861657114）に表示されたクラッシュログと完全に一致し、手順が正しいことを実証しています。

### 4.2. 修正後の同一手順（正常パス確認 ✅）

修正後のコミット（`06afeb3`）で同様に再現を行うと、すべて正常終了します。

```powershell
# 1. 修正済みコミットから package ファイルを抽出
cd .scratch/broken-sandbox
git show 06afeb3:package.json > package.json
git show 06afeb3:package-lock.json > package-lock.json

# 2. コンテナ内で実行
docker run --rm -v "${PWD}:/w" -w /w node:20.20.2 sh -c '
  npm ci --ignore-scripts
  npm audit --audit-level=high --omit=dev
  node node_modules/@wdio/cli/bin/wdio.js --version
'
```

#### 期待される成功出力
```text
found 0 vulnerabilities
9.29.1 (exit 0)
```
バージョン 9.29.1 が正常に出力され、ESM 解決不整合が本質的に解消されていることが証明されます。検証後、以下のクリーンアップを実行してください：
```powershell
cd ../..
Remove-Item -Recurse -Force .scratch/broken-sandbox
```

---

## 5. 本再現手順で「再現できない」範囲の限界明示 (AC1)

「Docker でローカル再現できる」という記述が誇大にならないよう、以下の限界を明示します。

- **Tauri バックエンドのテスト / ビルド:**
  `webkit2gtk`、`xvfb`、システム側のコード署名（SignPath）、およびネイティブ Windows/macOS API に依存するビルドとテストは、共通の Linux Node コンテナ内では再現できません。これらは専用の Windows 開発機または CI クラウドランナー上でのみ担保されます。
- **UIテスト/E2Eテストの実画面起動:**
  ヘッドレスブラウザを超えるネイティブ OS レイヤーの GUI テストは、Docker Node コンテナ上では検証対象外となります。

---

## 6. 一覧と実体の同期維持方針 (AC4)

本書に記述された検証ゲートや Node/npm バージョンなどの SSOT 定義が古くなり、実体と乖離して「黙って腐る」状態を防ぐため、以下の自動検知ポリシーを定義します。

1. **ワークフロー定義ファイル (.github/workflows/\*.yml) のフック検査:**
   GitHub Actions の各ジョブ構成ファイルが追加・修正された際、本書の定義一覧の変更も要求する簡易な CI ゲート、またはコミットルールを定義します。
2. **バージョン更新時の検出:**
   `package.json` の `"engines"` や `.github/workflows/ci.yml` の `node-version` などのバージョン値が書き換わった場合、本書（`ci-gate-local-reproduction.md`）の記載と不一致がないか、セキュリティスキャンまたは lint スイープにて定期的に検知します。
