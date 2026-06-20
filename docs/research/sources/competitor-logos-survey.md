# 競合・参照アプリ ロゴ実物調査 — 一次情報メモ

取得日: 2026-06-20
対象: QuickScribe(ローカル完結ボイスジャーナル)ロゴ刷新のための、直接競合＋隣接優良デザイン10社のアプリロゴ実物観察。
方針(README/ADR-0007準拠): 逐語転載せず、書誌・正典URL・鍵となる短い逐語引用のみ保存。
検証法: 各ロゴは公式サイト/App Store(iTunes Lookup API経由の公式 mzstatic 配信アセット)/公式ブランドキット/公式SVGソースの**実画像を取得し視覚確認**。WebFetchのmarkdown変換はアイコン画像を落とすため、バイナリ画像を取得し目視＋ピクセルサンプリングした。
**hex精度の注意**: ブランドキット由来の確定hexは Obsidian `#6c31e3`(公式SVGソースの逐語値)と Rosebud `#ED0F64`(公式サイトCSSの逐語値)のみ。他は公式App Store/公式画像からの**ピクセルサンプリング**(JPEG圧縮で±1-2/ch のずれ得る)。憶測は含めず、確認できたもののみ記述。

---

## 1) Day One(ジャーナリング, Automattic)

- 一次ソース: App Store配信アイコン(mzstatic CDN) / 公式リスティング https://apps.apple.com/ca/app/day-one-daily-journal-diary/id1044867788 / プレス https://dayoneapp.com/press/
  - アイコンファイル: `https://is1-ssl.mzstatic.com/image/thumb/Purple221/v4/91/4d/3c/914d3c0e-9038-c4a8-d20a-d1b040bb9d94/AppIcon-0-0-1x_U007epad-0-1-0-sRGB-85-220.png`
- 形/メタファ: 白い**ブックマーク/しおりリボン**(矩形上部＋下端V字ノッチ)。「保存したページ/日記の栞」の単純化フラット記号。書籍を literal には描かない。
- 色: 地は**フラットな空色 `#44C0FF`**(ピクセルサンプル, グラデなし=四隅と中央同値), グリフは純白 `#FFFFFF`。彩度高めだが軽く爽やか。
- 構図: iOS角丸タイル, 地は不透明青で全面塗り。白しおりは中央配置・余白たっぷり(中央幅約40%)。
- craft: 非常にクリーンなフラット。直線エッジ, 対称V字ノッチ, 影・テクスチャ・グラデなし。2色実装。
- 情緒: 静謐・軽やか・開放的・親しみ。空色＋白で「信頼できるが堅くない」。
- 小サイズ耐性: 優秀(2色高コントラスト・単一シルエット, favicon級でも可読)。
- 確認不能: ブランドキット逐語hexは未取得(プレスはzip配布のみ)。`#44C0FF`はアイコン実物サンプル値。

## 2) Obsidian(ローカル思考ノート)

- 一次ソース: 公式ブランド https://obsidian.md/brand / 新アイコン由来 https://obsidian.md/blog/new-obsidian-icon/ / **公式ロゴSVGソース**(色stop逐語読取) https://obsidian.md/images/obsidian-logo-gradient.svg / app tile https://obsidian.md/apple-touch-icon.png
- 形/メタファ: **黒曜石(火山岩)の多面カットされた結晶/宝石**。複数の平面が鋭利なエッジで交わる。公式由来: 火山岩で「矢じり・ナイフ等の道具を作った」 knapping(石を打ち欠いて鋭い刃を作る)＝「思考を研ぎ精緻化する」メタファ。
- 色: 紫。SVG base fill は**逐語 `#6c31e3`**。面取りは白のradial-gradientオーバーレイ(opacity .1–.8)を重ねて表現=**グラデ/多階調**(フラットでない)。サンプル: 明面 `#C9B3F5`, 暗面 `#5C30B4`。
- 構図: app tile版は**ほぼ黒の角丸タイル `#151515`** 上に宝石中央配置。標準ロゴSVG自体は**背景透過**。
- craft: 高い。SVGに8つの名前付きグラデ(四隅＋エッジハイライト)を定義し、面に光が当たる立体陰影を作る。鋭くクリーンなベクターエッジ。
- 情緒: プレミアム・鋭利・集中・"tool for thought"。紫＋暗タイルで知的・やや神秘的(遊び心ではない)。
- 小サイズ耐性: 良(中)。公式に「単色適用に縮小しても可読であるよう設計」と明記。グラデは大サイズでの richness 用、芯のシルエットは単色でも保持。

## 3) Notion

- 一次ソース: 公式iOSアイコン(自社CDN, 直接視覚＋alpha sampling) https://www.notion.com/front-static/logo-ios.png
- 形/メタファ: **黒い3D書籍/立方体(透視図の開いたノート)** 前面白地に**セリフ大文字"N"**。ページ/ノートのメタファ。表象的だが幾何的。
- 色: 厳密な**モノクロ黒白**。本体 `#000000`, 前面・天面ページ `#FFFFFF`。グラデ・色なし。
- 構図: iOSタイルは**白地ベタ**(262,144px中**透明0px**=完全不透明白背景を確認)。標準ベクターロゴは透過上の黒N-book。中央配置・余白快適。
- craft: クリーンで自信のある作り。均一な太い黒ストロークで立方体エッジ。セリフ"N"の終端はやや手彫り風(無機質でない letterpress 的味)。天面ページが透視で持ち上がり微3D。影・テクスチャなし(奥行きは線/透視のみ)。
- 情緒: 中立・プロフェッショナル・editorial/文学的・タイムレス。黒白＋セリフNで文書中心・信頼・控えめ。
- 小サイズ耐性: 優秀(純黒白・単一形状・高コントラスト)。
- 確認不能: ブランド逐語hexは未取得(Notion /brand は認証要 401)。黒白なので影響軽微。

## 4) Bear(ノート, Shiny Frog)

- 一次ソース: App Store(id 1016366447) `https://is1-ssl.mzstatic.com/image/thumb/Purple221/v4/5e/b7/6c/5eb76c4b-3486-b65a-4104-5dcf822bd06c/AppIcon-26-0-0-1x_U007epad-0-0-0-1-0-0-sRGB-85-220.png` / FAQ https://bear.app/faq/bear-app-icons/
- 形/メタファ: **左向きの熊の頭・肩のシルエット**(単一ベタ, 内部ディテールなし=目/毛なし, 輪郭・鼻先・耳・肩線のみ)。半抽象。
- 色: 赤角丸タイルに**縦グラデ**(上の明るい暖赤 `#E15556` → 下の深赤 `#C1181D`)。熊は白〜淡桃 `#FDF4F5`。暖色高彩度の赤。
- 構図: 角丸squircleタイル(透過/白でない)。熊は大きく下端で見切れ(肩/胸が下縁を割る), 下の余白タイト・上左は広め。中央左寄り右向き。
- craft: 高い。鼻先・背の連続滑曲, 白シルエットに微かな内側グロウ/光沢で立体感, 滑らかな背景グラデ。光学バランス良。
- 情緒: 暖かい・親しみつつ自信・プレミアム/作り込み。赤は静謐でなくエネルギー・集中。
- 小サイズ耐性: 強(単一高コントラストシルエット/ベタ地, 小サイズで即読)。

## 5) Rosebud(AIボイスジャーナル=最直接競合, rosebud.app)

- 一次ソース: App Store(id 6451135127) `https://is1-ssl.mzstatic.com/image/thumb/Purple221/v4/8b/ed/1a/8bed1a67-e7de-53b3-79d5-69db68aa963c/AppIcon-0-0-1x_U007epad-0-1-85-220.png` / 公式サイト(CSS逐語hex) https://www.rosebud.app/
- 形/メタファ: **抽象的な薔薇のつぼみ/花弁マーク**(2枚の丸い花弁/しずく形が重なり閉じたつぼみを示唆)。高度に抽象, ほぼ葉/つぼみのモノグラム(literal な薔薇でない)。
- 色: マゼンタ/ラズベリーピンクのベタ地, サンプル `#E31665`(公式サイトCSSのブランドピンク**逐語 `#ED0F64`** に近い)。前花弁は純白, 後ろの重なる花弁は半透明淡桃 `#F7B9D2`。実質2トーンフラット＋透過効果(滑らかなグラデではない)。鮮やか・暖・高彩度。
- 構図: 角丸マゼンタベタタイル(白/透過でない)。つぼみは中央・全方向均等余白。フラット。
- craft: クリーン・フラット・幾何的。滑曲。craft的妙味は2花弁の半透明重なりで第3の色味が出る点のみ。影・テクスチャ・グラデなし。
- 情緒: 暖かい・優しい・楽観・親密(やや feminine 寄り)。丸い花弁＋暖ピンクで「ケア/成長」。ジャーナル/wellnessに適合。
- 小サイズ耐性: 強(白onマゼンタの大胆シルエット高コントラスト)。微妙な花弁重なりだけ極小で潰れ得る。

## 6) Otter.ai(文字起こし)

- 一次ソース: App Store(id 1276437113) `https://is1-ssl.mzstatic.com/image/thumb/Purple211/v4/df/95/29/df95297d-4a96-2119-3846-0ba2da6111b1/otter-logo-0-1x_U007epad-0-1-85-220-0.png` / メディアキット(変種名のみ, hex非公開) https://otter.ai/media-kit
- 形/メタファ: ワードマーク由来のロゴマーク。「O-tt-er」を**音声波形として抽象化**(中空リング"O"＋高さの異なる縦丸棒列＋ドット)。文字"Otter"と sound/voice waveform のダブルリード(文字起こしに適切)。
- 色: 明地(ほぼ白)上に青グリフ。地 `#FFFFFF`〜淡灰 `#F5F5F5`, 青 `#3E90FE`(明azure)。青要素に微グラデ/光沢(上が明・下がやや深)。media kitは"Blue/White"変種名のみで**数値hex非公開**(`#3E90FE`はサンプル値)。
- 構図: 角丸タイルにほぼ白の地(透過でない)。青波形グリフ水平中央・余白広め・垂直中央。
- craft: 磨かれてモダン。丸いバーキャップ, gel状グラデと微内側ハイライト, 明タイルに対する微影。リング/バー/ドットの光学スペーシング滑らか。
- 情緒: プロフェッショナル・清潔・techy・信頼/calm。明地＋クール青で「生産性/ビジネスSW」(遊び心は薄い)。
- 小サイズ耐性: 中。中空リング＋複数細バー＋小ドットは単一シルエットより細かく, 極小でドット/バー間隙がぼやける。明灰地は色ベタタイルよりポップ弱い。

## 7) Granola(AI議事録, granola.ai)

- 一次ソース: App Store(id 6739429409)AppIconアセット / favicon `https://www.granola.ai/favicon/favicon-96x96.png` / リブランドブログ https://www.granola.ai/blog/a-new-look-for-granola
- 形/メタファ: **手描きの渦巻き/スワール**(1本の連続ブラシストロークが内へ巻き中心が塗り)。グラノーラの渦＋"loose, human"な落書き。App Storeタイルと現行サイトfaviconの両方に同マーク→現行と確認。
- 色: オリーブ/ライム緑タイル `#B2C248`, マークは near-black charcoal。グラデなし(フラット緑＋フラット濃ストローク)。公式blog: 「緑は維持しつつ proper system を与えた」「hand-drawn and deliberately imperfect」。
- 構図: App Store版=角丸タイル, フラット緑地, 渦ほぼ中央・余白広め。現行favicon=同渦を透過/緑tint地に(重いタイルなし)。
- craft: **意図的にラフ**。ブラシ/マーカーのテクスチャ, 不均一なストローク幅, 微かに揺れる線。幾何最適化していない(blog: 「committeeが作らない見た目」)。
- 情緒: 親しみ・有機的・暖・やや遊び心/quirky。blog: 「approachable and optimistic, a bit rough around the edges, but sharp enough to feel like a serious tool」。
- 小サイズ耐性: 良(単一高コントラスト濃渦/フラット薄緑地, 96pxでも明瞭)。
- 注: rebrand postに literal な形語/hexの記載なし。「渦」「`#B2C248`」は画像直接観察。Quadrant+Melange書体はblogに記載。

## 8) Apple Journal(純正ジャーナル)

- 一次ソース: iOS17公式アプリアイコンPNG(3375×3375) https://en.wikipedia.org/wiki/Special:FilePath/Apple_Journal_-_App_Icon_-_iOS17.png / リスティング https://apps.apple.com/us/app/journal/id6447391597 / Newsroom https://www.apple.com/newsroom/2023/12/apple-launches-journal-app-a-new-app-for-reflecting-on-everyday-moments/
- 形/メタファ: 4枚の曲面"花弁"/ページ状フォームによる**抽象的な蝶/開いた本**(上2翼＋下2翼が中央縦シームで会う)。蝶の翼と開いた日記の見開きページのダブルリード。
- 色: 深いネイビー/チャコール地 `#222338` 上に**4枚のグラデパネル**。サンプル: 左上ミュート紫 `#8871A7`, 右上暖peach `#FFB2A8`, 左下periwinkle青 `#8B9CF7`, 右下coral赤 `#FA625D`。各パネルは滑らかグラデ(内シームが明・外縁が深)。寒色紫青〜暖ピンクcoralに渡る。
- 構図: 標準iOS角丸タイル, 暗ネイビー地, マルチカラーマーク中央・均等余白・中央縦対称軸。
- craft: 高い。滑らかグラデ, 翼が暗地に重なる箇所の微内側陰影で層/3Dの紙めくれ感, 各花弁のクリーン曲率。
- 情緒: 静謐・内省・プレミアム・穏やかな楽観。暗地＋柔らかい発光パステルで「内省的・warm but serious」。Appleは「日常の瞬間を振り返る」と positioning。
- 小サイズ耐性: 強(大胆な対称シルエット, 暗タイルに対し高コントラスト)。

## 9) Reflect(AIノート, reflect.app)

- 一次ソース: App Store(id 1575595407)AppIconアセット / https://reflect.app/
- 形/メタファ: **発光する3Dネットワーク/グラフ球**(球面格子の曲線が交差, 交点に発光ノード=ドット)。connected knowledge graph / "second brain" を描く。
- 色: 角丸タイル内に深いviolet→blue地, ネットワーク線/ノードが明るい白紫に発光。サンプル: ノード発光 `#4E27AA`(深violet)＋ `#ECE5FF` 系ハイライト。線にneon/luminousグロウ, 滑らかvioletグラデ。(周囲のnear-white `#F8F8F8` はApp Store marketing枠でタイルでない)
- 構図: 角丸タイル, 暗violet/indigo地, 発光球が中央・中程度余白。
- craft: 高い(各strand/nodeに柔らかい outer glow/bloom, 球全体に滑らかな深度シェーディングで3D crystalline-network)。
- 情緒: 知的・プレミアム・やや magical/"AI"。暗地のneon violet glowで洗練・集中・ややethereal。
- 小サイズ耐性: 中。発光オーブのシルエットは縮小に耐えるが, 内部の細い線/個々のノードは極小でblur/merge。

## 10) Logseq(ローカル知識グラフ, logseq.com)

- 一次ソース: 公式リポジトリアセット(1024×1024) https://raw.githubusercontent.com/logseq/logseq/master/resources/icon.png / 意味確認 公式フォーラム https://discuss.logseq.com/t/what-is-the-logo/384
- 形/メタファ: 暗地に**3つの丸いティール blob/ドット**(上に小2つ, 下に大1つ, ゆるいクラスタ)。創業者公式説明: 「三つの slip-box と connection」, 双方向リンク記号 `[[ ]]`・block・log/断面 から着想。
- 色: 非常に暗いティール/ネイビー地 `#002B36`(Solarized dark base), 3マークは柔らかいミュートティール `#85C8C8`。フラット, グラデなし。
- 構図: 正方形アイコン, 暗ティールネイビー地が全面, 3ティール形を中央にグループ化・周囲に negative space たっぷり。
- craft: シンプルでクリーン(柔エッジのフラット楕円, テクスチャ/影/グラデなし, 意図的にミニマル)。創業者: 旧アイコンは「複雑で醜い」ので簡素化。
- 情緒: 静謐・控えめ・やや有機/抽象。ミュートティールon暗で quiet・technical・low-key(エネルギッシュでない)。
- 小サイズ耐性: 強(3つの高コントラストベタblob＋広いスペーシング, 小サイズでも判別可)。
- 注: 公式ブランドガイドのhexは非公開。サンプル値, `#002B36`は周知のSolarized dark base。

---

## 取得・確認した主要URL一覧
- Day One: https://apps.apple.com/ca/app/day-one-daily-journal-diary/id1044867788 / https://dayoneapp.com/press/
- Obsidian: https://obsidian.md/brand / https://obsidian.md/blog/new-obsidian-icon/ / https://obsidian.md/images/obsidian-logo-gradient.svg / https://obsidian.md/apple-touch-icon.png
- Notion: https://www.notion.com/front-static/logo-ios.png
- Bear: https://apps.apple.com/us/app/bear-markdown-notes/id1016366447 / https://bear.app/faq/bear-app-icons/
- Rosebud: https://apps.apple.com/us/app/rosebud-ai-journal-diary/id6451135127 / https://www.rosebud.app/
- Otter: https://apps.apple.com/us/app/otter-transcribe-voice-notes/id1276437113 / https://otter.ai/media-kit
- Granola: https://apps.apple.com/us/app/granola-ai-meeting-notes/id6739429409 / https://www.granola.ai/blog/a-new-look-for-granola
- Apple Journal: https://en.wikipedia.org/wiki/File:Apple_Journal_-_App_Icon_-_iOS17.png / https://apps.apple.com/us/app/journal/id6447391597 / https://www.apple.com/newsroom/2023/12/apple-launches-journal-app-a-new-app-for-reflecting-on-everyday-moments/
- Reflect: https://apps.apple.com/us/app/reflect-notes/id1575595407 / https://reflect.app/
- Logseq: https://raw.githubusercontent.com/logseq/logseq/master/resources/icon.png / https://discuss.logseq.com/t/what-is-the-logo/384

## オープンな疑問 / 限界
- ブランドキット逐語hexは Obsidian `#6c31e3` と Rosebud `#ED0F64` のみ確定。他はApp Store/公式画像のピクセルサンプル(±1-2/ch)。確定値が必要なら: Day Oneプレスzip, Otter `press@otter.ai`, Granola/Otter の Brandfetch(403で未取得)を手動確認。
- Mem / Stoic / Reflectly はフォールバック未使用(Reflect・Logseqの一次ソースで充足)。

---

## 横断分析: 比較表

| # | アプリ | 形/メタファ(抽象度) | 色(地→グリフ, グラデ) | 構図(地) | craft水準 | 情緒 | 小サイズ耐性 |
|---|---|---|---|---|---|---|---|
| 1 | Day One | しおり/リボン(中) | 空色`#44C0FF`→白, グラデ無 | 角丸タイル・ベタ青・余白大 | フラット2色・高クリーン | 静謐・軽・信頼 | 優 |
| 2 | Obsidian | 黒曜石カット結晶(中) | 紫`#6c31e3`+白グラデ面取 | 角丸ほぼ黒`#151515`(標準は透過) | 高(8グラデで立体) | 知的・鋭利・premium | 良(単色も可読設計) |
| 3 | Notion | 透視3D本+セリフN(中低) | 黒`#000`/白, グラデ無 | 白ベタタイル(完全不透明) | 高(手彫り風セリフ味) | 中立・文学的・timeless | 優 |
| 4 | Bear | 熊の頭シルエット(中) | 赤グラデ`#E15556`→`#C1181D`/白熊 | 角丸squircle・下見切れ | 高(微グロウ+滑曲) | 暖・親しみ・自信 | 強 |
| 5 | Rosebud | 薔薇つぼみ抽象花弁(高) | マゼンタ`#ED0F64`→白+半透明桃 | 角丸ベタ・均等余白 | クリーンフラット+透過妙味 | 暖・優・親密・成長 | 強 |
| 6 | Otter | 波形=文字"Otter"(高/ダブルリード) | 明白地→青`#3E90FE`・微光沢 | 角丸ほぼ白地・余白大 | 磨/gel光沢・微影 | プロ・清潔・techy | 中(細部多) |
| 7 | Granola | 手描き渦(中) | オリーブ緑`#B2C248`→charcoal, 無 | 角丸ベタ(現行faviconは透過) | 意図的ラフ・ブラシ質感 | 親しみ・有機・遊び心 | 良 |
| 8 | Apple Journal | 蝶/開いた本(高) | 暗ネイビー`#222338`+4色グラデ翼 | 角丸暗タイル・中央対称 | 高(滑グラデ+紙めくれ陰影) | 静謐・内省・premium | 強 |
| 9 | Reflect | 発光ネットワーク球(中) | 暗violet地+neon白紫グロウ`#4E27AA` | 角丸暗タイル | 高(bloom+3D球) | 知的・magical・AI | 中(内部線潰れ) |
| 10 | Logseq | 3つのティールblob(高) | 暗ティール`#002B36`→ミュート`#85C8C8`, 無 | 正方暗地・negative space大 | シンプル・ミニマル | 静謐・控えめ・technical | 強 |

## 優れたロゴに共通する設計原則 Top6

1. **自己完結したベタ地タイル＋反転グリフ**: 10社中ほぼ全てが角丸タイルに自前の地を持ち, グリフ対地の3:1コントラストを固定(QuickScribeの既存ADR=「透過上に細グリフ」を避ける根拠と一致)。
2. **単一の支配的シルエット**: 小サイズで即読されるのは「1つの大胆な形」(Day One/Bear/Notion/Logseq)。複数細部(Otter/Reflect)は耐性が一段落ちる。
3. **色相を1つに絞る規律**: 強い記憶はモノ色相(青=Day One, 赤=Bear, マゼンタ=Rosebud, 緑=Granola)。例外のApple Journalは「暗地が統一フレーム」として多色を束ねる。
4. **グラデは控えめ・単色相内 or 暗地に発光**: 効くグラデは Bear(単赤の濃淡)/Obsidian(紫面取り)/Journal(暗地に発光)。色相を跨ぐ派手グラデは小サイズ・高コントラストモードで崩れる。
5. **メタファは抽象度を1段引き上げる**: literal な物体でなく記号化(しおり, 波形=文字, つぼみ, 蝶=本)。説明的すぎないことで品位と記憶性。
6. **情緒の意図的選択**: ジャーナル系は「静謐/内省/暖」(Day One/Journal/Rosebud), ツール系は「鋭利/プロ」(Obsidian/Otter/Notion)。情緒の軸がプロダクト価値と一致。

## QuickScribeが参照すべき要素 / 避けるべき要素

**参照すべき(盗む)**:
- **暗地＋発光/淡グリフのジャーナル文法**(Apple Journal/Obsidian): 内省・premium・ローカルの落ち着きを最も強く出す。QuickScribeの deep indigo タイル方針と整合。
- **単一・大胆・抽象シルエット**(Day Oneしおり/Bearの熊): 物理ボタン統合の「押す/録る」体験を1記号で象徴できれば最強。
- **音声のダブルリード記号**(Otterの波形=文字): 「声を整形する知性」を波形+ペン/インク等のメタファ合成で示す好例(ただしOtterより単純化して小サイズ耐性を確保)。
- **単色相規律**(Granola緑/Rosebudマゼンタ): 1色で記憶を作る。QuickScribeは deep indigo を一貫させる。
- **意図的な"少しの手仕事感"**(Granola/Notionのセリフ): 機械的すぎない「人間の思考整理」温度を, ノイズでなくニュアンスとして1点だけ入れる。

**避けるべき**:
- **クラウドAIクリシェの紫ネオングラデ**(Reflectの発光ネットワーク球): 「ローカル/反クラウドAI」positioningと逆行。発光AI球は最も模倣すべきでない。色相も他研究メモ(`logo-design-color-and-metaphor.md`)で「マゼンタ紫の過飽和」として警告済。
- **明灰地＋多細部**(Otter): 色ベタタイルよりポップ弱く, 細部多で小サイズ耐性が落ちる。
- **遊び心過多/quirky**(Granolaのラフ渦): 親しみは良いが「思考整理の知性」という真剣さと相反し得る。温度は1点に留める。
- **literal すぎるメタファ**: マイク/吹き出し/ペンをそのまま描くと安っぽく説明的。10社は全て1段抽象化している。
- **色相を跨ぐ派手グラデ**: Apple Journalの多色は暗地フレームと光学調整の作り込みで初めて成立。同レベルの craft なしに真似ると小サイズ・HCで崩壊。
