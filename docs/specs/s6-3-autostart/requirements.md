# S6.3（一部）OSログイン時の自動起動 — requirements

> Status: Draft (2026-06-24) / 対象 Issue: #41（Epic E6 #13）の自動起動部分。
> 記法: 軽量BDD ＋ EARS。実装は公式 `tauri-plugin-autostart`（独自実装を避ける）。

## ユビキタス言語
- **自動起動（autostart）**: OSへのログイン時にQuickScribeが自動で起動し、トレイに常駐する設定。実体はOSの仕組み（Win=レジストリ Run / mac=LaunchAgent / Linux=.desktop）。

## 受入基準（EARS）
- **R1（event）**: When the user enables "ログイン時に自動起動", the system shall register QuickScribe to launch at OS login.
- **R2（event）**: When the user disables it, the system shall unregister the autostart entry.
- **R3（state）**: While the settings screen is open, the toggle shall reflect the **actual OS registration state**（localStorageでなくOSから取得）。
- **R4（unwanted）**: If enabling/disabling fails, then the system shall surface an error and re-sync the toggle to the real state（虚偽表示を避ける）。
- **R5（ubiquitous）**: Autostart-launched instances shall start with `--minimized` and **not show the window**（トレイ常駐から開始）。

## BDD 例
```gherkin
Scenario: 自動起動を有効化 (R1,R3)
  Given 設定画面を開いている
  When 「PCのログイン時に自動起動する」をONにする
  Then OSのログイン項目にQuickScribeが登録され、トグルは実状態ONを反映する

Scenario: 自動起動で立ち上がる (R5)
  Given 自動起動が有効
  When OSにログインする
  Then QuickScribe は --minimized で起動し、ウィンドウを出さずトレイに常駐する
```

## 範囲外（S6.3の残部）
最小化挙動の細部・終了確認・多重起動制御は single-instance 等で別途。本増分は「自動起動の登録/解除＋最小化起動」に限定。
