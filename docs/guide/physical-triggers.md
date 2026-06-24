# 物理ボタンで録音する（Stream Deck・マウス・フットスイッチ）

QuickScribe の録音は**設定可能なグローバルホットキー**で開始/停止できます。物理デバイスは、そのデバイスの公式ソフトで**このホットキーを送る**よう割り当てるだけで連携します（QuickScribe 側の追加設定は不要）。

## 0. まず QuickScribe 側のホットキーを決める

設定 →「録音開始/停止のホットキー」で、物理ボタンに割り当てる専用キーを登録します。
**おすすめは `F13`〜`F24`**。これらは通常どのアプリも使わないため、他の操作と衝突しません（Elgato も推奨）。物理デバイス側で F13〜F24 を送れない場合は `Ctrl+Shift+R` などでも構いません。

> QuickScribe はシステム全体のグローバルホットキーを登録するため、ウィンドウが非アクティブ（最小化・背面）でも反応します。

## 1. Elgato Stream Deck

専用プラグインは不要です。組み込みの **Hotkey** アクションを使います。

1. Stream Deck アプリで、System カテゴリの **Hotkey** アクションをボタンにドラッグ。
2. Hotkey 欄をクリックし、QuickScribe に登録したキー（例: `F13`）を押す。
3. 完了。そのボタンで録音開始/停止がトグルされます。

## 2. 多ボタンマウス

### Logitech（Logi Options+）
1. Options+ で対象ボタンを選択 → **キーストロークの割り当て**。
2. QuickScribe のホットキー（例: `F13`）を入力して保存。
   - 代わりに **Smart Actions → アプリケーション** で `quickscribe.exe --toggle-record` を起動する方法でも可。

### Razer（Synapse）
1. Synapse でボタンに **Keyboard Function** を割り当て、QuickScribe のホットキーを入力。
   - または **LAUNCH PROGRAM** で `quickscribe.exe --toggle-record` を指定。
2. Synapse は**バックグラウンド常駐**が必要です。

## 3. USB フットスイッチ（足踏みスイッチ）

**プログラマブルな USB フットスイッチ**（標準HIDキーボードとして認識される製品）を推奨します。
1. 付属ユーティリティで、ペダルに QuickScribe のホットキー（例: `F13`）を割り当て、**デバイス本体に保存**。
2. 設定はデバイスに保存されるため、**常駐ソフト不要・Linux でもそのまま動作**します（最も堅牢）。

> 固定機能の安価なペダルはキーを変更できないことがあります。**割り当て変更が可能なプログラマブル版**を選んでください。

## コマンドラインから（上級者・自動化）

常駐中の QuickScribe は CLI でも操作できます。

```
quickscribe --toggle-record
```

ショートカット・タスクスケジューラ・他社マクロソフトの「プログラム起動」から呼べます。

## Linux での注意

- フットスイッチ経路が最も確実です（ベンダーソフト不要）。
- Logicool/Razer のボタン再割り当ては Options+/Synapse が Linux 非対応のため、`solaar` / `libratbag`(Piper) / `OpenRazer` 等を利用してください。

---

> モーメンタリ録音（**押している間だけ録音**＝ hold-to-record）への対応は計画中です（[ADR-0014](../adr/0014-physical-trigger-integration.md)）。
