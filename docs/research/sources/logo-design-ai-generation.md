# 生成AIでアプリアイコンを作る実務 — 一次情報メモ

取得日: 2026-06-20
対象: QuickScribe アイコンを画像生成モデル(Gemini "nano banana"/Imagen/SDXL)で作り、
彩度マスク背景除去スクリプト(scripts/process-brand-icon.py)と相性良く出力するための prompt 指針。
方針(README/ADR-0007準拠): 逐語転載せず、書誌・正典URL・鍵となる短い逐語引用・出力形式を保存。

## パイプライン前提(本リポの制約)
- `min(R,G,B) < 170` を本体、淡色(高 min)を背景/グロウとして除去。
- 内部の穴を埋め、最大連結成分のみ残し、縁を微フェザー、タイトクロップ後 ~4% マージン、1024px RGBA 出力。
- 品質ゲート: 角は透過(alpha<=8)。不透明割合 0.55-0.90(高すぎ=グロウ残存ハロー、低すぎ=本体欠け)。
- **最大の敵 = 外側グロウ/グラデブルーム/影**(中彩度ハローがマスクを通過 or 本体縁を侵食しゲート失敗)。

## VERIFIED — Google 公式
- Gemini 2.5 Flash Image ("nano banana") プロンプトガイド:
  https://developers.googleblog.com/how-to-prompt-gemini-2-5-flash-image-generation-for-the-best-results/
  - 逐語/近逐語: "The background must be white." (切り抜き用途の明示指示)
  - 原則: "Describe the scene, don't just list keywords." (Gemini は narrative 文が有効。SDXLのタグ羅列と異なる)
  - ステッカー型テンプレ: "A [style] sticker of a [subject], featuring [key characteristics]. The background must be white."
  - 余白テンプレ: "The background is a vast, empty [color] canvas, creating significant negative space."
  - 既定で 3D/tactile レンダリングに寄る傾向(公式の icon 例が "colorful and tactile 3D style") → 3D/影/グロウを明示否定する必要。
- Gemini API Image generation (Nano Banana):
  https://ai.google.dev/gemini-api/docs/image-generation
  - 透過/alpha の指針は無し。icon 例は白背景 + "No text"。透過は非サポート前提で自前マスカーに委ねる。
- Imagen / Vertex プロンプトガイド:
  https://docs.cloud.google.com/vertex-ai/generative-ai/docs/image/img-gen-prompt-guide
  - ロゴ型テンプレ: "A {logo_style} logo for a {company_area} company on a solid color background"
  - negative-prompt フィールドあり: "Plainly describe what you don't want to see."
- Imagen は非推奨化(2026-08-17 shutdown 記載)、生成は nano-banana へ移行: https://ai.google.dev/gemini-api/docs/imagen

## コミュニティ実務(一部のみ裏取り)
- フラットデザイン定義 = 2D形状/ソリッド色/クリーンライン/写実影・グラデ・テクスチャ無し。media.io, iconikai.com
- アイコンは 3-5 特徴に絞る(過剰ディテールは小サイズで崩壊)。stockimg.ai, media.io
- 単一フォーカル/中央配置の単一シンボル、"no text"。stockimg.ai, aisuperhub.io
- SDXL は短く的を絞った negative が有効(品質ジャンクの羅列より)。https://help.layer.ai/en/articles/8120630
- グロウ/ブルーム/ビネット/ベベル等の具体 negative トークンは**一般実務(未検証)**: glow, outer glow, bloom, vignette, bevel, emboss, reflection, specular highlight, depth of field, gradient background, soft light, ambient occlusion, drop shadow。

## 再利用可能 prompt ルール(マスク互換)
A. 背景: 純白フラット背景を明示("plain pure-white background #FFFFFF, flat, uniform")。外側グロウ/ブルーム/ビネット/背景グラデを本文+negative で禁止。
B. 本体: 単一中央フォーカル + 単一の角丸スクエア本体(最大連結成分維持)。深く彩度の高い本体色("rich saturated")で min(RGB)<170 を確保。クリスプな硬い縁。内部グリフは白/淡色OK(穴埋めされる)。
C. スタイル: フラット2Dベクター。3D/ベベル/エンボス/写実陰影/スペキュラ/反射を明示否定(Gemini は既定で 3D に寄るため必須)。ドロップ/キャスト影禁止(灰色=中 min がマスクを汚す)。
D. 構図: 中央 + たっぷり余白。テキスト/文字無し。
E. モデル方言: Gemini=1つの descriptive 文に統合。SDXL=positive タグ + 専用 negative。Imagen=solid背景テンプレ + negative フィールド。
F. 頑健性: 太く単純な形状(極細線は淡色化し削られ opaque<0.55)。白を**本体色に使わない**(背景として削除される。白は本体上の内部グリフに限定)。地面/接地面/環境を描かない。

## 残課題
- グロウ/ブルーム negative トークンはベンダー未検証 → 本パイプラインの opaque-fraction ゲート通過率で A/B 較正推奨。
- マスク閾値170 と淡いブランド色の整合(淡色は本体が削られる)。最小彩度を prompt で強制 or 閾値調整。
- Gemini の 3D 既定傾向が強い場合、否定だけに頼らず生成→ゲート判定→再生成ループを検討。
