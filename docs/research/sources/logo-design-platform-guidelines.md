# アプリアイコン設計 — プラットフォーム公式ガイドライン一次情報メモ

取得日: 2026-06-20
対象: QuickScribe アプリアイコン刷新。小サイズ可読性・単一フォーカル・シルエット認識の規範。
方針(README/ADR-0007準拠): 全文逐語転載はしない。書誌・正典URL・構造・鍵となる短い逐語引用のみ保存。

---

## 1. Apple — Human Interface Guidelines: App icons
正典: https://developer.apple.com/design/human-interface-guidelines/app-icons

鍵となる逐語引用:
- "An effective icon is a graphic asset that expresses a single concept in ways people instantly understand."
- "Embrace simplicity in your icon design. Simple icons tend to be easiest for people to understand and recognize."
- "An icon with fine visual features might look busy when rendered with system-provided shadows and highlights, and details may be hard to discern at smaller sizes."
- "Make sure to avoid extremely thin line weights and sharp corners, because they tend to lose detail and crispness in smaller icon sizes at lower resolutions."
- テキスト: "Text in icons doesn't support accessibility or localization, is often too small to read easily, and can make an icon appear cluttered."
- 深度/効果(Liquid Glass): "Let the system handle blurring and other visual effects... there's no need to include specular highlights, drop shadows between layers, beveled edges, blurs, glows, and other effects."
- 中央寄せ/セーフエリア: "Keep primary content centered to avoid truncation when the system adjusts corners or applies masking."

サイズ指針: 単一マスター + システム縮小。マスター 1024×1024px (iOS/iPadOS/macOS/visionOS)。ページ上に16/20/24/32 の個別再描画リストは無い。可読性は「smaller sizes」という定性表現。

注: Apple ページは JS レンダリングのため、上記は Apple の page-data JSON から取得。原典は要再確認。

---

## 2. Material Design — Product icons
正典(M2, 製品アイコンの正規ガイド): https://m2.material.io/design/iconography/product-icons.html
M3 併読: https://m3.material.io/styles/icons/designing-icons

鍵となる逐語引用:
- "Icons communicate the core idea and intent of a product in a simple, bold, and friendly way."
- "Each icon is cut, folded, and lit as paper would be, but represented by simple graphic elements."
- "The icon grid establishes clear rules for the consistent, but flexible, positioning of graphic elements. Keyline shapes are based on the grid."
- ライブエリア: "Icon content should remain inside of the live area... No parts of the icon should extend outside of the trim area."
- (M3) "Sharp Symbols use corners and straight edges for a crisp, rectangular style that remains legible even at smaller scales."

サイズ指針: 製品アイコンは 48×48 dp グリッドで設計、編集は 400%(192×192dp)。システムアイコンは 24×24dp 目標、20/40/48dp サポート。タッチターゲット 48×48dp 以上。

注: M2 製品アイコン引用は検索インデックス経由(JSレンダリングのため)。Google 公開文言に一致するが原典で要確認。

---

## 3. Microsoft — Design guidelines for Windows app icons (Fluent)
正典: https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-design
(最終更新 2021-10-29 / 2026-03-07 リフレッシュ)

鍵となる逐語引用:
- "Your icon should illustrate the concept of your app in a singular element using simple forms."
- "To enhance communication clarity, use no more than two metaphors in a single icon. If a single metaphor can be used, that's even better."
- "A good test for an effective icon is when users can tell what it represents without a label."
- "Use the grid to design a silhouette that's distinctive, yet legible at small sizes. Use as few shapes with as few corners as possible..."
- "When adding detail, care should be taken to maintain legibility at small sizes."
- "Microsoft aligns its icons to a 48x48 grid initially..."
- "Icons should not include typography as part of the design. Letters and words on your icon should be avoided and only used when it's essential."
- フラット性: "Icons are composed of flat objects sitting on top of the layers below it." / "Layers should always be flat and perpendicular to the viewing angle." / "Icons should be drawn with a straight-on perspective"; "perspective is not recommended."
- 色スケーリング: "To avoid complexity when scaling an icon across a range of sizes, treatments to colors should be minimized."

サイズ/フォースドカラー指針:
- マスターグリッド 48×48。角丸 2px(外)/1px(内) @48px。
- High contrast: "Windows 11 no longer requires high contrast assets for app icons." 必要時は "High contrast icons are black and white... Avoid gradients in high contrast icons."
- アクセシビリティ: "at least half of your icon passes a 3.0:1 contrast ratio on light and dark theme."
- テーマ別: 任意で light/dark 別資産を Taskbar/Start 用に用意可。

注: Windows のアプリアイコンは 16/24/32/48/256px で出荷されるが、その個別サイズ一覧は資産パッケージング側のドキュメントにあり、本デザインページは 48×48 マスターモデルで表現。

---

## クロスプラットフォーム合意点(3社一致)
1. 単一コンセプト/単一フォーカル要素 (Apple/Material/MS, MS は ≤2 metaphor)
2. シンプル/最小形状 (3社)
3. 小サイズ可読性を明示的制約に (3社)
4. 細部/極細線の回避 (Apple/MS, Material はグリッド+ライブエリアで制約)
5. ラベル無しで認識できる独自シルエット (3社)
6. テキスト回避 (Apple/MS)
7. 写実/UI複製の回避 (3社)
8. セーフエリア/グリッド/キーライン (3社)
9. フラット構成・自前の深度を最小化 (Apple=システムに委任 / MS=厳格フラット / Material=固定光のペーパー影)
10. 色/グラデの抑制でスケール・テーマに耐える (3社)

深度の差異: 3社とも「デザイナーが重い静的3D影を塗ること」は禁止。システムが上に深度を載せるかで差。
