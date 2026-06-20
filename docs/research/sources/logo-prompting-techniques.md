# 一次資料スナップショット — 画像生成AIによるプロ品質ロゴ/アプリアイコンのプロンプティング技法

> Status: Research source snapshot (2026-06-20)。ADR-0007 準拠（要約で薄めず原文要点を出典URL付きで保存）。
> 調査対象: QuickScribe ロゴ刷新における「画像生成AIでチープなAIアイコン臭を避けて実用品質を出す」プロンプティング技法。
> 失敗の文脈: Gemini (gemini-2.5-flash-image) に浅いキーワード列挙（"flat 2D vector app icon, indigo tile, white mark, no gradient..."）で一発生成 → 量産AIアイコンのクリシェ（角丸タイル+太字グリフ）に陥り却下。

---

## 1. Google公式: Gemini 2.5 Flash Image のプロンプト技法（最重要・一次）

出典: Google Developers Blog "How to prompt Gemini 2.5 Flash Image generation for the best results"
URL: https://developers.googleblog.com/en/how-to-prompt-gemini-2-5-flash-image-generation-for-the-best-results/

核心原則（原文要点）:
- **"Describe the scene, don't just list keywords."** モデルは「narrative, descriptive paragraph（叙述的な段落）」で最も性能を発揮し、「simple list of disconnected words（バラバラな単語の羅列）」は劣る。
  → 我々の失敗（キーワード列挙）はGoogle公式が明示的に劣ると言う書き方そのものだった。
- **"Be hyper-specific: The more detail you provide, the more control you have."** 例として "fantasy armor" ではなく "ornate elven plate armor, etched with silver leaf patterns, with a high collar and pauldrons shaped like falcon wings" のように記述せよ。

ロゴ向け公式テンプレート（原文）:
> "Create a modern, minimalist logo for a coffee shop called 'The Daily Grind'. The text should be in a clean, bold, sans-serif font. The design should feature a simple, stylized icon of a coffee bean seamlessly integrated with the text. The color scheme is black and white."

スタイル化イラスト/ステッカー テンプレート（原文）:
> "A [style] sticker of a [subject], featuring [characteristics] and [color palette]. Design should have [line style] and [shading style]. Background must be white."
- 具体例: "A kawaii-style sticker of a happy red panda ... The design features bold, clean outlines, simple cel-shading, and a vibrant color palette."

ミニマル構成 例（原文・余白/質感の表現に有用）:
> "A minimalist composition featuring a single, delicate red maple leaf positioned in the bottom-right of the frame. The background is a vast, empty off-white canvas..."
- ネガティブスペース/オフホワイト地/単一フォーカル要素 を**文章で**指定する手本。

参照画像（マルチモーダル）テンプレート（原文）:
- 編集: "Using the provided image of [subject], please [add/remove/modify] [element]... Ensure the change is [integration description]."
- インペイント: "Using the provided image, change only the [specific element] to [new element]. **Keep everything else exactly the same, preserving original style, lighting, and composition.**"
- スタイル転送: "Transform the provided photograph of [subject] into the artistic style of [artist/style]. **Preserve original composition** but render with [stylistic elements]."
- マルチ画像合成: "Create a new image by combining elements from provided images. Take the [element 1] and place it with/on the [element 2]..."

アスペクト比の挙動（原文要点・重要な落とし穴）:
- 編集時、モデルは「generally preserves the input image's aspect ratio」。
- 複数画像をアスペクト比違いで渡すと「the model will adopt the aspect ratio of the **last** image provided」。
- 比率が言葉で効かない時は「**provide a reference image with the correct dimensions**」のが公式ベストプラクティス。

質感/光/素材の精密記述（原文要点）:
- 写真用語が精密な制御を与える: "wide-angle shot, macro shot, low-angle perspective, 85mm portrait lens, Dutch angle"。
- 質感例: "soft, golden hour light streaming through a window, highlighting the **fine texture of the clay**"。
- 製品例: "three-point softbox setup" の照明, "polished concrete surface", "matte black" 仕上げ。

→ 含意（QuickScribe）: アイコンでも「フラットタイル」と単語で言うのでなく、**地の質感（matte paper / soft off-white）・光（subtle top-down soft light）・縁の硬さ・余白の広さ・単一フォーカル**を段落で叙述すべき。参照画像（Day One / Notion 風の地）を渡して style/composition を寄せるのが公式に裏打ちされた正攻法。

---

## 2. プロのMidjourneyロゴ・プロンプト構造（一次解説）

出典: SologoAI "Midjourney for Logos in 2026: The Ultimate Guide"
URL: https://www.sologo.ai/blog/midjourney-for-logos/

プロのプロンプト構文（原文）:
> "[Subject/Brand Name] + [Logo Type] + [Style/Mood] + [Visual Elements] + [Art Direction] + [Technical Parameters]"

効くキーワード（原文要点）:
- ミニマル: "geometric," "monoline," "single line art," "negative space"。例: "abstract geometric mountain peak symbol, hexagon frame, monoline"。
- ワードマーク: タイポを明示し、文字自動生成を `--no text, letters, words` で抑制。
- マスコット: "simple shapes, bold outlines, flat colors" で過剰レンダリングを回避。

パラメータ（原文要点）:
- `--style raw`（芸術的装飾を排しクリーンなロゴに）
- `--ar 1:1`（ロゴは正方形が最安全）
- `--stylize 20-30`（ミニマル向け。値が低いほど指示に忠実=クリシェ回避に寄与）
- `--sref [image URL]`（**スタイル参照**で line weight / color vibe / minimalism level を世代間で固定）

補足出典: CyberLink / Superside（複数二次だが一致する技法）
URL: https://www.cyberlink.com/blog/ai-prompts/5141/midjourney-ai-logo-prompts
URL: https://www.superside.com/blog/ai-prompts-logo-design
- "photorealistic" や "cinematic" はロゴから遠ざけるので避ける。
- `--s 50 / --s 100` や `--c 0 / --c 5` でシンプル&再現性を上げる。
- 重み付け "minimalist logo::3 flower::1" でコア概念を優先。
- **Remix ループ**: 気に入った1枚をupscale → Remixで1-2語だけ変えて再生成、を反復して磨く（一発生成しない）。

→ 含意: 我々の失敗は「一発生成・単語列挙・参照なし」の三重苦。プロは(構文化された段落) + (style raw相当=装飾抑制) + (sref=参照固定) + (反復Remix) で詰める。

---

## 3. ベクター生成 vs ラスター→トレース（一次・実務フロー）

出典: MindStudio "What Is Recraft V4 Vector? How to Generate Native SVG Logos and Icons With AI"
URL: https://www.mindstudio.ai/blog/what-is-recraft-v4-vector-generate-svg-logos-icons-ai

ネイティブSVG vs トレースの差（原文要点）:
- Recraft V4 Vector は「doesn't generate a raster image and then 'trace' it into vector paths」→「generates **vector-native output from the start**」。
- 結果は「**cleaner path structures**」。対してオートトレースは「typically produces over-complicated paths with **many unnecessary anchor points**」。
- 出力SVGは「Clean `<path>` elements」「Explicit fill and stroke colors」「Viewbox and dimensions set correctly」で「**lacks embedded raster data**」。

効くプロンプト記述（原文）:
> "Use descriptors like `flat design`, `geometric`, `minimalist`, `limited color palette`, `icon style`, `bold outlines`."
- 「specific about subject and style, **not scene**」「**name your colors or palette**」。

限界（原文要点・重要）:
- 「Complex typographic logotypes with custom letterforms still **benefit from human refinement**.」
- 「The more complex the scene, the **messier the vector output**.」
- 「AI generation **won't match** [exact brand standards] automatically.」

実務フロー（原文）:
> "Think of the AI output as a **strong starting point, not a finished asset**. A designer can typically take a solid Recraft output and finalize it in **15–30 minutes**—compared to hours building from scratch."
- 手順: generate → download → inspect SVG code → open in Illustrator/Figma/Inkscape → refine（colors, paths, proportions）。

ツール役割分担（一次比較）:
出典: Rangy "Ideogram vs Recraft for Logos (2026)"
URL: https://rangy.ai/blog/ideogram-vs-recraft-for-logos/
- Recraft V3/V4 は「the only major AI model that outputs **true editable SVG paths**」。Ideogram 3 はラスター（PNG/JPEG）のみ。
- Ideogram はテキスト描画 90-95% 精度（他は30-50%）でワードマーク向き。
- プロのワークフロー: 「**wordmark in Ideogram, vector finalize in Recraft**」。トレースしたベクターは「not the same as a natively-generated SVG path」。

→ 含意: Geminiはラスター出力。ロゴ実用にはネイティブSVG（Recraft）か、Gemini/Midjourneyのラスターを**人手でSVGに引き直す**前提が必要。one-shotラスターをそのまま製品アイコンにするのが「チープ化」の温床。AI出力は「強い出発点」であり、craft（パス整理・光学調整）は人手 or ベクターネイティブツールで担保する。

---

## 4. アイコンcraft: グリッド/キーライン/光学調整（一次）

出典: Helena Zhang "Icon Grids & Keylines Demystified"
URL: https://minoraxis.medium.com/icon-grids-keylines-demystified-5a228fe08cfd

- キーシェイプ（原文）: "Four basics are common: a circle, square, portrait rectangle, and landscape rectangle." 同一面積でなく**形状ごとに別キーライン**で光学的に揃える。
- 光学バランス優先（原文）: "**Always check for optical balance... It's often necessary to deviate from the grid for visual balance. Follow what looks optically right versus strict metric values.**"
  - "G" アイコン例: グリッド通りだと視覚的に大きく見え、**縮小**してファミリー内で調和させた。
- セーフ/トリム領域（原文）: "The safe area or live area shows where the important content of the icon should live, while the inverse—the trim area—shows the area to avoid."
- 具体値（Phosphor for Android）: 48×48px canvas / 1.5px centered stroke / 6px trim area / keyshape 28×28 circle, 25×25 square, 22×28・28×22 rect。

→ 含意: 「同じ正方形タイルに同じ太字グリフ」は光学調整ゼロ＝AIクリシェの正体。円形要素は方形より一回り大きく、視覚的に飛び出す角・線は縮める等の**光学補正**を後処理で入れるとプロ品質に近づく。

### コーナー曲率: squircle / superellipse（一次・数式）

出典: Squircle.js "The Squircle Formula: Superellipse Math Explained"
URL: https://squircle.js.org/blog/math-behind-squircles
出典: Squircle.js "How Apple Uses Squircles in iOS Design"
URL: https://squircle.js.org/blog/squircles-in-apple-design

- superellipse 式: `|x/a|^n + |y/b|^n = 1`。squircle は `x^4 + y^4 = r^4`（n=4）。「raising the exponent from 2 to 4 flattens the sides and squares up the form while keeping the curve perfectly smooth」。
- Appleの実装（原文）: 「a corner radius of about **22.37% of the icon width with 60% corner smoothing** reproduces the iOS icon shape, which is exactly what `cornerSmoothing: 0.6` renders」。
- 普通の角丸との差（原文）: 角丸長方形は「curvature jumps **discontinuously**」（辺で曲率0、角で1/r）。squircleは「curvature changes **continuously** ... There is **no jump**」（**G2連続/曲率連続**）。

→ 含意: 単純な `border-radius`（普通の角丸）タイルは曲率不連続で「やや硬く・既製感」。Apple/iOS品質は superellipse（n≈4-5, smoothing≈60%, r≈22.4%）。タイル形状を squircle にするだけで「チープな角丸タイル」感が下がる。

---

## 5. 「単色飽和タイル+白グリフ」を脱する方向（一次・批評）

出典: 複数のデザイナー批評（AIロゴがチープに見える理由の合意点）
URL: https://medium.com/@alomaria/your-ai-logo-sucks-76c60f35c151
URL: https://thelogocompany.net/why-ai-generated-logos-often-fail-real-brands/

チープ化の合意要因（原文要点）:
- AIは「templates, pre-existing styles, trend-based algorithms」に依存し「generic designs that lack originality ... built from the same visual datasets」。→ 「角丸タイル+太字グリフ」は学習分布の最頻パターン。
- AIは「whether your brand needs to feel premium, warm ... technical」を理解しない。**戦略/感情の翻訳が欠落**。
- 「AI typically outputs **raster** ... real logos need vector ... fail when resized, simplified ... small details disappearing」。
- 縮小耐性: 「AI-generated logos often fail when resized or simplified」。

→ 脱クリシェの実装的含意（QuickScribe）:
1. **地を単色飽和ベタにしない**: off-white / 微細マット紙テクスチャ / ごく薄い陰影で「素材感のある地」を叙述（Gemini公式の "fine texture of the clay" 手法を地に応用）。Day One/Notion系の繊細さはここ。
2. **グリフを太字ベタ塗りにしない**: monoline / single continuous line / negative space を使い「描かれた知性」を表現（コア価値=もつれ→直線の変換に合致）。
3. **タイル自体を squircle** 化し、内側に光学調整した単一フォーカル要素を置く。
4. 一発生成でなく **参照画像で寄せ → 反復Remix → 最終はSVGで人手整形**。
5. ラスター生成物は必ずベクター化（Recraftネイティブ or Illustrator Image Trace で引き直し、アンカー削減）。

---

## 反証条件（各結論が覆る条件）

- (Q1/2) Gemini参照画像入力が style/composition をほぼ無視する（実機で検証）なら、Gemini単体生成を諦め Recraft/Midjourney(sref) に主軸を移す。
- (Q3) ネイティブSVG出力（Recraft）の品質が我々のグリフ複雑度では破綻するなら、ラスター生成→人手SVGトレースに統一。
- (Q4) superellipse 化しても 16-24px で角丸との差が視認できないなら、コーナー曲率投資の優先度を下げる。
- (Q5) off-white地+微細質感が暗テーマ/タスクバーで図地コントラスト3:1（WCAG 1.4.11）を割るなら、彩度を上げた地に戻す（本リポ品質ゲートが優先）。
