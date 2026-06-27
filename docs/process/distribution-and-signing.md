# 配布・署名の手順とトレードオフ（E8 / #50 #51 #54）

> Status: Reference (2026-06-27)。要オーナー対応の項目（証明書/アカウント/方針）の手順と、無い場合のデメリットをまとめる。
> 役割分担: 「**私(自動化側)**で用意できるもの」＝CIワークフロー/manifest雛形/手順。「**あなた(オーナー)**が用意するもの」＝証明書・各種アカウント・方針承認。

## 結論（先に）
- **#50 署名 / #51 winget・Flathub が無いデメリット = 「インストール時の警告（信頼感低下）と、配布チャネルでの発見性低下」だけ**。機能・プライバシー・安全性・自動更新（Tauri updater署名で完全性は担保）には影響しない。
- → MVP/早期公開は「**未署名 + GitHub Releases + アプリ内updater**」で十分回る。署名・winget・Flathubは**ユーザーが増えてから投資**でよい。

## ★維持コストはゼロにできる（無料提供プロダクト前提）
本プロダクトは無料提供＝**金銭コストをかけない方針**。以下はすべて**無料**で成立する:

| 項目 | 無料か | 備考 |
|---|---|---|
| GitHub Releases 配布 | ✅ 無料 | 既に運用中 |
| GitHub Actions CI/ビルド | ✅ 無料 | **publicリポジトリは無料** |
| Tauri updater 署名（自動更新の完全性） | ✅ 無料 | 自前鍵。導入済み |
| winget 公開 | ✅ 無料 | winget-pkgs へPRするだけ |
| Flathub 公開 | ✅ 無料 | submission のみ |
| ブランチ保護 | ✅ 無料 | GitHub設定 |
| **Windows コード署名** | ⚠️ 通常は有料 | **OSSなら無料の道あり↓** |

### Windowsコード署名を無料で：SignPath Foundation（OSS向け）
- **[SignPath.org（Foundation）] は OSS プロジェクトに無料でコード署名を提供**（"For OSS projects, our services are free of charge."）。HSM保管の証明書・**個人の身元確認不要**（リポジトリ→バイナリの紐付けを Foundation が保証）・**GitHub Actions連携**。
- 必要なのは**金銭でなく「申請と審査」**（メンテナが apply する。OSS実体・CIビルドであること等が条件）。
- QuickScribe は MIT・公開リポジトリ・CIビルドのため**該当見込み**。
- 出典: https://signpath.org/ （Apply ページから申請）

→ **有料証明書（Trusted Signing $10/月・SSL.com 年$200-400 等）は不要**。無料路線で署名まで到達できる。

### コストゼロの推奨ロードマップ
1. **当面**: 未署名のまま GitHub Releases ＋ アプリ内updater（無料・現状）。READMEに「SmartScreen警告時は『詳細情報→実行』」を明記して摩擦を下げる。
2. **署名（無料）**: SignPath Foundation に申請 → 承認後、私がCIに署名ステップを組み込む（鍵はSignPath側HSM・費用ゼロ）。
3. **winget/Flathub（無料）**: 公開チャネルへ。発見性UP。
4. **ブランチ保護（無料）**: 品質ゲート。

---

## #50 コード署名（Code Signing）

### 無い場合のデメリット
- **Windows**: SmartScreen「Windowsによって PC が保護されました／発行元不明」警告。初回は「詳細情報→実行」を踏む必要＝インストール障壁・信頼低下（DL数に効く）。
- 一部の配布チャネルが署名を要求する。
- ※ **改ざん検知自体は Tauri updater 署名で担保済み**。欠けているのは「OSの発行元検証」。

### 手順（Windows）
1. **証明書を入手**（OVで可。EVは不要）:
   - 2023年以降、秘密鍵は HSM/トークン保管必須（FIPS）。**クラウド署名サービス**ならCIから署名できる:
     - **Microsoft Trusted Signing（旧Azure Code Signing）**: 安価（$9.99/月〜）・CIフレンドリー（推奨）。組織/個人の本人確認が必要。
     - SSL.com eSigner / DigiCert KeyLocker 等: 年額〜$200-400。
2. **CIに署名を組み込む**（私が用意）: release.yml に署名ステップ（Trusted Signing Action / SSL.com CodeSignTool）を追加。tauri-action の Windows 署名フック（signCommand/証明書サムプリント）または生成後 .exe を signtool で署名。
3. **Secrets登録**（あなた）: アカウント資格情報を GitHub Secrets に。
4. macOS は別系統（Apple Developer $99/年 + notarization）。Win/Linux中心の現状は後回し可。

- 私: release.yml 署名ステップ・Secrets設計・手順書。あなた: 証明書/アカウント購入・Secrets値。
- 補足: OV証明書でも当初はSmartScreen評価が貯まるまで警告が出ることがある（DL実績で改善）。

---

## #51 配布チャネル（winget / Flathub）

### 無い場合のデメリット
- **winget無し**: `winget install` で入れられず、GitHub Releases から手動DLのみ（更新はアプリ内updaterで回る）。発見性・導入容易性が下がる。
- **Flathub無し**: Flatpak で入れられない。AppImage/debはあるので致命的でないが、Linux配布の主流に乗らず発見性減。
- 総じて **致命的でない／普及（リーチ）が制限されるだけ**。

### 手順（winget）
1. 安定したインストーラURL（＋推奨：署名）を用意。
2. `wingetcreate` で manifest 生成 → `microsoft/winget-pkgs` へ PR。
3. CI自動化: リリース時に `wingetcreate update` で manifest 更新→自動PR（GitHub Action あり）。
4. Microsoft自動検証→マージで `winget install Publisher.QuickScribe` 可能に。
- 私: manifest生成・CI自動PRワークフロー。あなた: 識別子確定（`TakenoriKusaka.QuickScribe`等）・winget-pkgsへの初回PR/PAT。

### 手順（Flathub）
1. Flatpak manifest(yaml) 作成（Tauri/WebKitGTK向けSDK・最小権限）。
2. `flathub/flathub` に submission PR → レビュー（サンドボックス権限・ネット権限の正当化を指摘されがち）→ マージで公開。
- 私: manifest雛形・権限設計。あなた: Flathubアカウント・submission。

---

## #54 リポジトリ堅牢化（ブランチ保護）

### 無い場合のデメリット
- 赤いCIのまま main にマージ/直pushできてしまう＝品質事故リスク。現状は手動運用で守っているが、1.0では機械的強制が望ましい。

### 手順（私が gh api で設定可能・要承認）
- main に保護ルール: **PR必須＋必須ステータスチェック（build / e2e / transcribe / CodeQL）＋会話解決必須**。
- 影響: 以後 main への直pushが不可（docs含め全てPR経由）。→ ドキュメントの即時反映が一手間増える。
- 承認をもらえれば即設定可能。

---

## 推奨ロードマップ（コスト最小で普及を上げる順）
1. **Microsoft Trusted Signing** で Windows コード署名（最も体感に効く・安価）。→ SmartScreen警告解消。
2. **winget** 自動PR（署名後だと審査が楽）。→ Windows導入容易化。
3. **ブランチ保護**（無料・即時）。
4. Flathub（Linuxユーザーが増えたら）。
