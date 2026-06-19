# タスクバー録音コントロールのユーザビリティ/アクセシビリティ — 一次情報メモ

取得日: 2026-06-19
対象: QuickScribe (Tauri v2 + Rust, Windows) のタスクバー上に重畳する小型録音コントロール
（左=録音●/停止■トグル、右=ウィンドウを開く）
目的: 準拠すべき規格(A)・モダンである根拠(B)・正直なアクセシビリティ評価(C)・改善設計(D) の一次情報固定。
併読: `windows-taskbar-control.md`（実装パスと UIPI/サブクラス検証）、`windows-thumbnail-toolbar.md`。

注: README 方針に従い全文逐語転載はしない。書誌・正典URL・構造・鍵となる短い逐語引用のみ保存する。

---

## A. ユーザビリティ/アクセシビリティ規格（条項＋逐語引用）

### WCAG 2.2（W3C Recommendation, 2023-10-05; 正典 https://www.w3.org/TR/WCAG22/）
各 SC の規範文は w3.org の Understanding ページ（https://www.w3.org/WAI/WCAG22/Understanding/）で確認。

- **1.1.1 Non-text Content (Level A)** — 逐語:
  「All non-text content that is presented to the user has a text alternative that serves the equivalent purpose, except for the situations listed below.」
  Controls/Input カテゴリの規定（逐語）:
  「If non-text content is a control or accepts user input, then it has a name that describes its purpose.」
  Understanding 注（逐語）: 「A label is presented to all users whereas the name may be hidden and only exposed by assistive technology.」
  → アイコンのみボタンは「purpose を記述する name」が必須。ツールチップは視覚ヒントになるが、プログラム的な name（UIA/IAccessible 上の名前）が別途必要。

- **1.4.11 Non-text Contrast (Level AA)** — 逐語:
  「The visual presentation of the following have a contrast ratio of at least 3:1 against adjacent color(s): User Interface Components [:] Visual information required to identify user interface components and states, except for inactive components or where the appearance of the component is determined by the user agent and not modified by the author; Graphical Objects [:] Parts of graphics required to understand the content...」
  → 録音/停止/ウィンドウ アイコングリフと、状態（録音中/停止中）を識別するために必要な視覚情報は隣接色に対し 3:1 以上。明/暗テーマ両方で満たす必要。

- **2.5.3 Label in Name (Level A)** — 逐語:
  「For user interface components with labels that include text or images of text, the name contains the text that is presented visually.」
  Understanding 補足（逐語）: 「where a visible text label does not exist for a component, this success criterion does not apply to that component.」
  「Speech-input users ... can navigate by speaking the visible text labels of components.」
  → アイコンのみ（可視テキストラベル無し）の場合は厳密には本 SC の適用外。ただし可視ラベルを足すなら、その文言を accessible name に含めること（音声入力対応）。

- **4.1.2 Name, Role, Value (Level A)** — 逐語:
  「All user interface components have a name and role that can be programmatically determined; states, properties, and values that can be set by the user can be programmatically set; and notification of changes to these items is available to user agents, including assistive technologies.」
  → 本UIの最大の論点。自前描画オーバーレイは name/role をプログラム的に公開しないと非準拠（後述C）。

- **3.2.4 Consistent Identification (Level AA)** — 逐語:
  「Components that have the same functionality within a set of web pages are identified consistently.」
  → タスクバー側「録音」とアプリ内「録音」は同一機能。ラベル/アイコン/名称を一貫させる。

- **2.5.8 Target Size (Minimum) (Level AA)** — 逐語:
  「The size of the target for pointer inputs is at least 24 by 24 CSS pixels, except when:」
  例外（逐語）:
  - Spacing: 「Undersized targets ... are positioned so that if a 24 CSS pixel diameter circle is centered on the bounding box of each, the circles do not intersect another target or the circle for another undersized target」
  - Equivalent: 「The function can be achieved through a different control on the same page that meets this criterion」
  - Inline: 「The target is in a sentence or its size is otherwise constrained by the line-height of non-target text」
  - User agent control: 「The size of the target is determined by the user agent and is not modified by the author」
  - Essential: 「A particular presentation of the target is essential or is legally required...」
  → 24×24 CSS px 未満なら「24px 径の円が重ならない」間隔を確保。WCAG は CSS px 基準（≒96dpi 基準。Windows ネイティブは DPI スケーリングを加味して物理ピクセル換算が必要 — これは設計時の留意点であり SC 文言自体は CSS px）。
  「Equivalent」例外: アプリ内ボタン＋グローバルホットキーという別経路で同機能を満たせるなら、オーバーレイ側の小ささは緩和されうる（ただし帰属・発見性は別問題）。

### Microsoft（Fluent / Win32 公式ドキュメント）

- **Tooltips（WinUI/Fluent, MS Learn, ms.date 2025-02-26）**
  正典 https://learn.microsoft.com/en-us/windows/apps/design/controls/tooltips
  鍵となる逐語引用:
  - 「Toolbar controls and command buttons showing only icons need tooltips.」（＝アイコンのみのコマンドボタンはツールチップ必須）
  - 「Does a control have a text label? If not, use a tooltip to provide the label.」
  - 「the text must be supplemental — that is, not essential to the primary tasks. If it is essential, put it directly in the UI so that users don't have to discover or hunt for it.」
  - 「Keyboard accelerators are displayed in tooltips by default. If you add your own tooltip, make sure that it includes information about the keyboard accelerators which are available.」（＝ツールチップにショートカットを含めよ）
  - 「Don't use a tooltip to display text already visible in the UI.」
  → アイコンのみボタンにツールチップを付け、動作＋ショートカットを示すのは Microsoft 公式の指針に合致。ただし「本質情報はツールチップに隠さずUIに直接」という制約あり。

- **Win32 ToolTip control（TOOLTIPS_CLASS, MS Learn, ms.date 2018-05-31, updated 2025-03-11）**
  正典 https://learn.microsoft.com/en-us/windows/win32/controls/tooltip-controls
  要点（逐語/近逐語）:
  - 作成: 「call CreateWindowEx and specify the TOOLTIPS_CLASS window class」。`InitCommonControlsEx` で comctl32 をロード。`WS_POPUP | TTS_NOPREFIX | TTS_ALWAYSTIP`、`HWND_TOPMOST` 指定。
  - ツール登録: `TTM_ADDTOOL` ＋ `TOOLINFO` 構造体。
  - 「A tooltip control supports tools implemented as windows ... and as rectangular areas within a window's client area.」（＝任意の矩形領域にツールチップを付与可能 → 自前オーバーレイの各ボタン矩形に付けられる）
  - 矩形ツールでは `TOOLINFO.hwnd` に親、`rect` にクライアント座標、`uID` に識別子。`TTF_SUBCLASS` でマウスメッセージ自動中継（同一スレッド条件）。別経路では `TTM_RELAYEVENT` で手動中継。
  - 標準ツールチップ本文上限 80 文字。複数行はそれ以上可。
  → 自前 Win32 オーバーレイでもツールチップは実装可能（D で採用）。

- **WM_GETOBJECT（UI Automation/MSAA, MS Learn, ms.date 2025-07-14）**
  正典 https://learn.microsoft.com/en-us/windows/win32/winauto/wm-getobject
  要点（逐語）:
  - 「Sent by both Microsoft Active Accessibility and Microsoft UI Automation to obtain information about an accessible object contained in a server application.」
  - 「If dwObjId is equal to UiaRootObjectId, the request is for a UI Automation provider ... return a provider using the UiaReturnRawElementProvider function.」
  - 「If the window or control does not need to respond to this message, it should pass the message to the DefWindowProc function」。
  → 自前ウィンドウが `WM_GETOBJECT` を `DefWindowProc` に流すだけ（既定）の場合、UIA プロバイダを返さない。`UiaReturnRawElementProvider`＋`IRawElementProviderSimple`（または MSAA `IAccessible`＋`LresultFromObject`）を実装して初めて name/role が露出する。実装しなければスクリーンリーダから「役割・名前を持つコントロール」として見えない（=4.1.2 非準拠）。

### Nielsen Norman Group（NN/g）

- **Tooltip Guidelines**（著者 Alita Kendrick, 2019-01-27）
  正典 https://www.nngroup.com/articles/tooltip-guidelines/
  鍵となる逐語引用:
  - 「If you're too stubborn to provide text labels for the icons on your site, the least you can do is provide your users with a descriptive tooltip.」
  - 「Users shouldn't need to find a tooltip in order to complete their task.」（タスク必須情報をツールチップ依存にしない）
  - 「Tooltips that appear only on mouse hover are inaccessible for users that rely on keyboards to navigate.」（ホバーのみは非アクセシブル）
  - 「Tooltips are microcontent — short text fragments intended to be self-sufficient.」

- **Icon Usability**（著者 Aurora Harley, 2014-07-27）
  正典 https://www.nngroup.com/articles/icon-usability/
  鍵となる逐語/近逐語引用:
  - 「a user's understanding of an icon is based on previous experience」。多くのアイコンは標準的意味を欠く。
  - 「icon labels should be visible at all times, without any interaction from the user.」（ラベルは常時可視が理想 — ホバー依存を戒める）
  - ほぼ普遍認知のアイコンは home / print / 検索の虫眼鏡の 3 つ程度。
  - 「a word is worth a thousand pictures」（Bruce Tognazzini 引用）。
  → アイコンのみは誤解を招く。ツールチップは最低限の保険であり、可視ラベル併用が望ましい。ただしタスクバー上の極小領域という制約とは緊張関係（D でバランス案）。

---

## B. このUIがモダンである根拠（実アプリ・原典つき）

- **録音=赤丸●/停止=四角■ は国際的慣習**
  - Media control symbols（Wikipedia, 出典 ISO/IEC 18035 を明記）:
    正典 https://en.wikipedia.org/wiki/Media_control_symbols
    逐語/近逐語: Play=三角(▶, U+23F5)、Stop=四角(⏹, U+23F9)、Pause=二本線(⏸, U+23F8)、Record=円(⏺, U+23FA)。
    「Their application is described in ISO/IEC 18035.」 録音は赤の円、停止は四角で**形状が明確に異なる**。
  - 録音=赤丸の由来（UX SE / 各種）: カセット/テープデッキの赤い録音ボタンに由来。誤って上書きする高コストを避けるため赤で警告的に分離。
    参考: https://ux.stackexchange.com/questions/41434 （StackPrinter 経由）。
  - **含意**: QuickScribe の「録音=赤丸」は普遍慣習に合致しモダン。一方「ウィンドウを開く=塗りつぶし四角」は**停止(■)と衝突**し慣習違反（D で是正）。

- **アイコン＋ツールチップのミニ操作UIの実例**
  - Windows サムネイルツールバー（旧 Win7 以降）: play/pause/next をアイコン＋ツールチップで提供（`THB_TOOLTIP`）。
    併読 `windows-taskbar-control.md` §1–§4（ThumbBarAddButtons の `THB_TOOLTIP`、Electron 実装）。
  - System tray / 通知領域アイコン: 多数アプリがアイコンのみ＋ホバー툴チップで状態/操作を提供（Windows の確立パターン）。
  - （未一次確認）OBS Studio・Discord・foobar2000 のミニ/トレイ操作が「アイコン＋ツールチップ」で成立、という具体UIは本調査では原典で個別確認していない（一般に観測される事実だが、本メモでは未確認として扱う）。
  → 「アイコン＋ツールチップ＋ショートカット明記」はモダンなツールバー/トレイ作法として Microsoft 公式（Tooltips ドキュメント）と Windows 標準パターンに裏打ちされる。

---

## C. アクセシビリティの正直な評価

- 自前描画の Win32 オーバーレイは、`WM_GETOBJECT` を `DefWindowProc` に流すだけだと **UIA/MSAA 上で name/role を持たない** → スクリーンリーダから不可視 → **WCAG 4.1.2 非準拠の可能性が高い**（一次根拠: WM_GETOBJECT ドキュメント）。
- ツールチップ（TOOLTIPS_CLASS）は視覚ヒントを足すが、**ホバー前提のため非アクセシブル**（NN/g）。ツールチップ＝アクセシブルな name ではない。
- 妥当な設計: **機能本体をアクセシブルな代替で担保**する。
  - アプリ内に role/name を持つ標準ボタン（Tauri/WebView2 側は ARIA/ネイティブで 1.1.1/4.1.2 を満たしやすい）。
  - グローバルホットキー（録音/停止）。WCAG 2.5.8「Equivalent」例外（同機能の別コントロールが基準を満たす）に接続。
  - オーバーレイは「補助的ショートカット」と位置づけ、本質機能はアクセシブル経路に置く。
- 余力があれば: オーバーレイ各ボタンに最低限の UIA（`WM_GETOBJECT`→`UiaReturnRawElementProvider`＋`IRawElementProviderSimple` で Name/ControlType=Button/Toggle 状態）を実装すれば 4.1.2 に近づく。コスト高のため段階実装（スパイク）候補。

---

## D. 改善設計（QuickScribe 向け）

- **ツールチップ文言例**（動作＋現在ショートカット＋アプリ名）:
  - 録音ボタン（停止中）: 「録音開始 ・ Ctrl+Shift+R ・ QuickScribe」
  - 録音ボタン（録音中）: 「録音停止 ・ Ctrl+Shift+R ・ QuickScribe」（状態でトグル）
  - ウィンドウボタン: 「QuickScribe ウィンドウを開く」
  - 根拠: MS Tooltips「アイコンのみボタンはツールチップ必須」「アクセラレータを含めよ」、NN/g「descriptive tooltip」。
- **アイコン**:
  - 録音/停止トグル: 録音=赤丸●、録音中の停止=四角■（ISO/IEC 18035 / Unicode 慣習）。
  - ウィンドウを開く: **塗りつぶし四角をやめ「ウィンドウらしいグリフ」**（外枠＋上部にタイトルバー線、または右上に外部リンク風の角）。停止(■)と形状で峻別（3.2.4/1.4.11 の識別性）。
- **アプリ帰属**: ツールチップに「QuickScribe」を明記。可能なら極小アプリマーク/色アクセントを併置。
- **ターゲットサイズ**: 各ボタン 24×24 CSS px 以上を基本（DPI スケール時は物理 px 換算で確保）。困難なら 24px 径円が重ならない間隔（WCAG 2.5.8 Spacing 例外）。
- **コントラスト**: アイコングリフ/状態識別情報を隣接色に対し 3:1 以上、明/暗テーマ両対応（1.4.11）。録音赤は暗背景/明背景双方でコントラストを検証。
- **一貫性**: タスクバー側とアプリ内で録音アイコン/ラベルを統一（3.2.4）。

---

## 一次情報URL一覧
- WCAG 2.2 本体: https://www.w3.org/TR/WCAG22/
- 1.1.1: https://www.w3.org/WAI/WCAG22/Understanding/non-text-content.html
- 1.4.11: https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html
- 2.5.3: https://www.w3.org/WAI/WCAG22/Understanding/label-in-name.html
- 4.1.2: https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html
- 3.2.4: https://www.w3.org/WAI/WCAG22/Understanding/consistent-identification.html
- 2.5.8: https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
- MS Fluent Tooltips: https://learn.microsoft.com/en-us/windows/apps/design/controls/tooltips
- MS Win32 ToolTip control: https://learn.microsoft.com/en-us/windows/win32/controls/tooltip-controls
- MS WM_GETOBJECT: https://learn.microsoft.com/en-us/windows/win32/winauto/wm-getobject
- NN/g Tooltip Guidelines: https://www.nngroup.com/articles/tooltip-guidelines/
- NN/g Icon Usability: https://www.nngroup.com/articles/icon-usability/
- Media control symbols (ISO/IEC 18035): https://en.wikipedia.org/wiki/Media_control_symbols

## 未確認/留意
- 4.1.2 の充足度はオーバーレイの UIA 実装次第（現状未実装なら非準拠の公算）。
- OBS/Discord/foobar2000 の具体ミニUIは本調査で個別原典未確認（一般観測のみ）。
- WCAG の「CSS px」と Windows ネイティブの物理px/DPI スケールの対応は設計時に換算検証が必要。
- ISO/IEC 18035:2003 の原本（有料規格）は未取得。Wikipedia 経由の二次参照。
