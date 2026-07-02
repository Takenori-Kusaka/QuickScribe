# SignPath Foundation 署名 — CI組込ドラフト（承認後に差し込む）

> Status: **DRAFT / 未検証**（2026-06-28）。SignPath Foundation の申請を送信済み・**審査結果待ち**。
> 承認メール（`@signpath.org`）が届き、Organization/Project/Signing Policy が用意されたら、本書の手順で `release.yml` に署名ステップを差し込む。
> 役割: 証明書・HSMは SignPath 側（費用ゼロ・鍵管理不要）。CIワークフローと本手順は自動化側で用意。

## 0. 前提・スコープ
- 署名対象は **Windows の NSIS インストーラ（`*_x64-setup.exe` / `*_arm64-setup.exe`）のみ**。
- Linux（AppImage/deb）と macOS（未ビルド）は対象外。
- 既存の **Tauri updater 署名（自前Ed25519鍵）= 自動更新の完全性担保**は別物で、これは継続する。SignPath が足すのは **Authenticode（OSの発行元検証＝SmartScreen警告解消）**。

## 1. ★最重要の設計論点：署名の順序（Authenticode と updater署名の競合）
Authenticode 署名は `.exe` のバイト列を書き換える。一方 Tauri updater の `.sig` は `.exe` のバイト列に対する署名。
**順序を誤ると updater が「署名不一致」で自動更新に失敗する。**

正しい順序は必ず:
1. NSIS `.exe` を生成（未署名）
2. **SignPath で Authenticode 署名**（`.exe` が書き換わる）
3. **署名済み `.exe` に対して Tauri updater 署名を生成**（`tauri signer sign` で `.sig` を作る）
4. 署名済み `.exe` ＋ 新しい `.sig` ＋ `latest.json` を Release に添付

→ つまり現状の `tauri-action`（ビルド＋updater署名＋Release公開を一括実行）を**そのままでは使えない**。
updater署名を Authenticode の**後**に回す必要があるため、下記いずれかの再構成が要る。

## 2. 採用方針（プライオリティ順に検討）
### 案A（推奨・prior artに最も近い）: ビルドと公開を分離
1. `tauri-action` を **`includeUpdaterJson: false` かつ Release公開させない**設定でビルドのみ実行（または `tauri build`）。updater署名も**この段では行わない**（`TAURI_SIGNING_*` を渡さない）。
2. 生成された NSIS `.exe` を `actions/upload-artifact` でアップロード。
3. `signpath/github-action-submit-signing-request@v1` で署名要求 → 署名済み `.exe` を取得。
4. 署名済み `.exe` に対し `tauri signer sign`（`TAURI_SIGNING_PRIVATE_KEY` 使用）で `.sig` を生成。
5. `latest.json` を組み立て（既存7プラットフォームキーの形を踏襲）。
6. `softprops/action-gh-release` 等で Release に全アセットを添付。

### 案B（簡易・updater完全性を一部妥協）: updater署名は未署名exe基準のまま、Release添付だけ署名exeに差し替え
- 自動更新が署名不一致で壊れるため**不可**。採用しない（記録のみ）。

→ **案Aを採用**。`max-parallel:1` で latest.json を統合する現行構造を、`needs:` で「全ビルド→署名→公開」の3段に組み替えるのが堅い。

## 3. 必要なもの（承認後に確定する値）
| 項目 | 入手元 | 置き場所 |
|---|---|---|
| `SIGNPATH_API_TOKEN` | SignPath プラットフォーム（User settings → API tokens） | GitHub Secrets |
| Organization ID | SignPath Organization | workflow（直書き可・機密でない） |
| Project slug | SignPath Project（GitHubリポと紐付け） | workflow |
| Signing Policy slug | `release-signing` 等（test/release） | workflow |
| Artifact configuration slug | 署名対象アーティファクト定義 | workflow |

## 4. 差し込む YAML スケルトン（DRAFT・未検証 / 承認後に実値で確定）
```yaml
# release.yml に追加する署名ジョブの骨子（案A）。
# 注意: 実際の input 名・バージョンは承認後に SignPath ドキュメントで最終確認すること。
  sign-windows:
    needs: build            # tauri ビルド（未署名・updater署名なし）が成果物をupload済みの前提
    runs-on: ubuntu-latest
    steps:
      - uses: signpath/github-action-submit-signing-request@v1
        with:
          api-token: ${{ secrets.SIGNPATH_API_TOKEN }}
          organization-id: ${{ vars.SIGNPATH_ORG_ID }}
          project-slug: quickscribe
          signing-policy-slug: release-signing
          artifact-configuration-slug: windows-installers
          github-artifact-id: ${{ needs.build.outputs.windows_artifact_id }}
          wait-for-completion: true
          output-artifact-directory: signed/
      # 署名済み exe に対して updater 署名を再生成
      - name: Re-sign updater (.sig) over Authenticode-signed exe
        env:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        run: npx @tauri-apps/cli signer sign signed/QuickScribe_*_x64-setup.exe
      # latest.json 再構築 + Release 添付（softprops/action-gh-release 等）
```

## 5. 検証（承認後・差し込み直後にやること）
- [ ] テストタグ（例 `v0.0.0-signtest`、prerelease）で署名ジョブを通す。
- [ ] 署名済み `.exe` のプロパティ「デジタル署名」タブに SignPath の署名が出るか。
- [ ] **自動更新が壊れていないか**: 旧版→新版の updater 更新を実機 or 手動で確認（署名不一致が出ないこと）。
- [ ] `latest.json` の7プラットフォームキーが従来通り揃うか。
- [ ] SmartScreen 警告が（評価蓄積後に）解消する見込みかを記録。

## 6. 残課題 / 注意
- SignPath Action の正確な input 名・最新版は**承認後に公式ドキュメントで再確認**（本書は申請段階の想定値）。
- ローカルに cargo 無し → ビルド検証はCI依存。署名ジョブも **テストタグでCI上で確認**する。
- 署名導入後は `release.yml` の `releaseBody` の「未署名」文言を更新する。
- macOS署名（Apple Developer $99/年 + notarization）は本件と別系統・対象外。

参照: [`distribution-and-signing.md`](distribution-and-signing.md)（無料署名=SignPath Foundationの全体方針）。
