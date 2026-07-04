# ブランドカラーパレット

QuickScribe の公式カラーパレット。すべての UI 色はここで定義するトークン（CSS カスタムプロパティ）を経由して使う。

- **単一の真実の源**: トークンの実体は [`src/app.css`](../../src/app.css) の `:root` に定義する。生の hex を UI コード（`.css` / `.svelte` の `<style>`）に直接書くことは **stylelint（`color-no-hex`, CI の lint ジョブ）で禁止**している。色を追加・変更する場合は本書と `src/app.css` を同時に更新する。
- **根拠**: [ロゴ再設計調査](../research/logo-redesign.md) の色の方向（§2）と craft 原則（§6.4）。紫系を「捨てず規律する」方針のもと、**deep indigo / blue-violet（色相約 250–275°）の単色相規律**をブランドの軸とする。

## 1. アクセント（deep indigo — ブランドの軸）

| トークン | hex | 用途 |
|---|---|---|
| `--color-accent` | `#4f46e5` | 主ボタン背景・スピナー・進捗%・フォーカス色。ブランドの基準色 |
| `--color-accent-strong` | `#4338ca` | 主ボタン hover・アクセント文字（エントリ種別・チップ active 等） |
| `--color-accent-bright` | `#6366f1` | 進捗バーのグラデ始点・チップ active 枠 |
| `--color-accent-deep` | `#3730a3` | 更新バナーの文字色（indigo 地の濃色テキスト） |
| `--color-accent-border` | `#c7d2fe` | アクセント系の枠線（副ボタン枠・スピナー地・カード強調枠） |
| `--color-accent-border-strong` | `#a5b4fc` | アクセント系枠線の hover |
| `--color-accent-bg` | `#eef2ff` | アクセント淡背景（バナー・バッジ・コード・選択状態） |
| `--color-accent-bg-hover` | `#f8faff` | エントリ hover 背景 |
| `--color-accent-bg-tint` | `#fbfbff` | 整形済みカードの極淡背景 |
| `--color-link` | `#2563eb` | タグ等のリンク的テキスト（indigo に隣接する青。多用しない） |

## 2. ニュートラル（テキスト・枠線・背景）

| トークン | hex | 用途 |
|---|---|---|
| `--color-text` | `#1f2330` | 基本テキスト |
| `--color-text-secondary` | `#374151` | 二次テキスト（プレビュー・強調 muted） |
| `--color-text-muted` | `#4b5563` | 補助テキスト。**淡グレー背景上でも WCAG AA 4.5:1 を満たす濃さ**（#395） |
| `--color-text-faint` | `#6b7280` | 弱い補助テキスト（白背景上のみ。淡背景上は `--color-text-muted` を使う） |
| `--color-border` | `#e5e7eb` | 標準の枠線（カード・パネル） |
| `--color-border-strong` | `#d1d5db` | 入力欄・チップの枠線 |
| `--color-border-faint` | `#eef0f3` | 極淡の区切り線（設定グループ） |
| `--color-surface` | `#ffffff` | カード・入力欄などの面。アクセント地の上の文字/ドットにも用いる |
| `--color-bg` | `#f8fafc` | アプリ基底背景（`app.css`） |
| `--color-bg-muted` | `#f3f4f6` | 本文画面の背景・アイコンボタン hover |
| `--color-bg-subtle` | `#f9fafb` | 淡い面（エントリ本文表示・空状態 CTA） |

## 3. 状態色（成功・警告・エラー）

| トークン | hex | 用途 |
|---|---|---|
| `--color-success` | `#10b981` | 成功・オンデバイス（プライバシー）ドット |
| `--color-success-strong` | `#059669` | 再起動ボタン背景 |
| `--color-success-hover` | `#047857` | 再起動ボタン hover |
| `--color-success-bg` | `#ecfdf5` | 成功系の淡背景 |
| `--color-success-border` | `#a7f3d0` | 成功系の枠線 |
| `--color-success-text` | `#065f46` | 成功系背景上のテキスト |
| `--color-warning` | `#f59e0b` | 警告・クラウド送信（プライバシー）ドット |
| `--color-warning-bg` | `#fffbeb` | 警告系の淡背景（誤字修正パネル等） |
| `--color-warning-border` | `#fde68a` | 警告系の枠線 |
| `--color-warning-text` | `#92400e` | 警告系背景上のテキスト |
| `--color-danger` | `#b91c1c` | エラーテキスト・削除対象の取り消し線 |
| `--color-danger-bright` | `#dc2626` | 録音中ボタン背景 |
| `--color-danger-bg` | `#fef2f2` | エラーの淡背景 |
| `--color-danger-border` | `#fecaca` | エラーの枠線 |

## 4. ロゴと UI の対応関係

ロゴは[再設計調査 §6.6](../research/logo-redesign.md) の結論どおり「**暗 deep-indigo squircle 地 ＋ 淡く発光する単一 monoline グリフ**（ほどけ目1つを持つ連続線＝整形の知性）」を採用方向とする。UI との対応:

- **色相の一致**: UI のアクセント階調（`--color-accent` #4f46e5 ＝ 色相約 243°、`--color-accent-strong`/`--color-accent-deep` はより深い同色相）は、ロゴの deep indigo（250–275°）と同じ単色相規律に属する。UI・ロゴともに**色相跨ぎの派手なグラデーションを使わない**（進捗バーのグラデも `#6366f1 → #4f46e5` の単色相内）。
- **明暗の役割分担**: ロゴは「暗地＋淡発光グリフ」、UI は「明地（`--color-bg*`）＋ indigo アクセント」。同一色相を明暗反転で使い分けることで、アイコンとアプリ画面が同じブランドとして繋がる。
- **状態色は機能色**: 緑/琥珀/赤はプライバシー状態・警告・エラーの意味伝達専用であり、ブランド表現（ロゴ・マーケ素材）には使わない。

## 5. 運用ルール

1. 新しい色が必要になったら、まず既存トークンで代替できないか検討する（「リッチすぎると簡便でなくなる」原則）。
2. 追加する場合は `src/app.css` の `:root` にトークンを定義し、本書の表に用途とともに追記する。
3. `.css` / `.svelte` の宣言値に生 hex を書かない（`npm run lint:css` / CI が検出）。hex が書けるのは `src/app.css` のトークン定義ブロック（`stylelint-disable color-no-hex` 区間）のみ。
4. 既知の例外: `color-mix()` に渡す CSS 名前色（`crimson` / `orange`、設定エラー表示とストリークバッジ）が2箇所残っている。トークン化は将来の見直し対象。
5. コントラストは WCAG 2.1 AA（テキスト 4.5:1・非テキスト 3:1）を維持する（#395 の実測に基づく注記をトークン用途に記載）。
