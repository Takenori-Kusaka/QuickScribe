# 色・形のトーン と 競合アイコン/視覚メタファ — 一次情報メモ

取得日: 2026-06-20
対象: QuickScribe(ローカル完結ボイスジャーナル)アイコンの色相/彩度/コントラストの根拠、
および競合アイコン傾向と「埋もれない」視覚メタファの記号論的根拠。
方針(README/ADR-0007準拠): 逐語転載せず、書誌・正典URL・鍵となる短い逐語引用のみ保存。

---

## 1. 色の意味論(エビデンスの強弱を明示)
- **最強の一次(大規模異文化研究)**: Jonauskaite, Mohr et al. (2020) "Universal Patterns in Color-Emotion Associations..." *Psychological Science*. n=4,598 / 30か国 / 22言語。
  正典: https://journals.sagepub.com/doi/10.1177/0956797620948810 / OA: https://eprints.whiterose.ac.uk/169496/
  - 色-感情連想は概ね普遍(平均類似 r≈.88) "apart from emotion associations with PURPLE." 紫(と茶)は連想数も少ない。
  - 含意: **紫は意味が最も異文化的に不安定な色**。「紫=内省/創造」は最も根拠が弱い物語。普遍意味論で紫を正当化できない。
- 文脈依存(一次): "The good, the bad, and the red" (2023) https://pmc.ncbi.nlm.nih.gov/articles/PMC10017663/ — 色の意味は関係的・文脈依存・文化依存。紫は論じられず。
- 大衆色彩心理(=慣習であって科学でない): WebMD https://www.webmd.com/mental-health/what-is-color-psychology 。violet→想像/創造/洗練、blue→沈静/信頼、teal→均衡/明晰。西洋デザイン文脈の共有慣習として設計上は有用。

## 2. コントラスト/可読性(明暗両タスクバー) — WCAG 1.4.11
正典: https://www.w3.org/WAI/WCAG21/Understanding/non-text-contrast.html / 技法 G207: https://www.w3.org/WAI/WCAG21/Techniques/general/G207
- 逐語: UI Components / Graphical Objects は隣接色に対し 3:1 以上。値は丸めない(2.999:1 は不合格)。
- MS が本ケースに具体化: https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-design
  - "App icons are primarily displayed on either light and dark backgrounds..."
  - "Make sure at least half of your icon passes a 3.0:1 contrast ratio on light and dark theme."
- 含意(load-bearing): アイコンは**自己完結した塗りの角丸スクエア本体**(中にグリフを反転配置)であるべき。透過上に細グリフが浮く構成は、明/暗/壁紙のいずれかで 3:1 を割る。塗りタイルは自前の地を持つため意味あるコントラスト(グリフ対タイル)が固定される。

## 3. フォースドカラー / High Contrast
- MDN forced-colors: https://developer.mozilla.org/en-US/docs/Web/CSS/@media/forced-colors
  - 強制色モードは著者色を上書きし、background-image を none、box-shadow/text-shadow を none に強制 → **グラデと影は破棄される**。
- MS app icon High contrast(同ページ): "Avoid gradients in high contrast icons." / "Windows 11 no longer requires high contrast assets for app icons."(が、シルエットで読める設計は維持すべき)
- MS Edge: https://blogs.windows.com/msedgedev/2020/09/17/styling-for-windows-high-contrast-with-new-standards-for-forced-colors/
- 含意: アイコンの identity が**グラデ依存だと HC で消える**。2トーンのシルエットで読めること。

## 4. 紫の評価と推奨
- FOR: 西洋デザイン慣習で violet/indigo は内省/創造/calm-premium を喚起。青多めの競合からの差別化。
- AGAINST: 紫は異文化的に意味が最も不安定(Jonauskaite 2020)。2023-2025 の AI ブランドで紫グラデが過飽和(下記ソフト証拠)。グラデは HC で破棄され小サイズで崩れる。
  - 紫グラデの AI クリシェ(ソフト): https://www.jackpearce.co.uk/notes/purple-gradient-ai-aesthetics/ / https://www.ebaqdesign.com/blog/ai-startups-logos
  - QuickScribe は「ローカル/反クラウドAI」が positioning → クラウドAI的紫グラデは逆効果。
- 推奨: 紫系は捨てず**規律する**。マゼンタ紫(290-320°)でなく**deep indigo/blue-violet(約250-275°)**へ。深い値の indigo タイル + 内側に白/淡グリフ(3:1容易)。フラット〜準フラット(グラデは1-2段・単色相内)。HC でも 2トーンシルエットで成立。任意で控えめな teal アクセント(AI紫の波で相対的に未使用)。
- 反証条件: (a) 音声ジャーナル/ローカルAIニッチの競合スキャンで indigo が既に密集 → 差別化にならない / (b) ユーザーテストで deep indigo が「冷たい企業AI」と読まれる → 温かい内省レーン(deep teal-green / 暖clay/amber=私的ノート)へ転換。

---

## 5. 競合アイコン サーベイ(VERIFIED: 実画像目視。itunes lookup API/公式資産で確認)

### 音声・文字起こし系
- Apple Voice Memos: **波形/オシロ**(縦バー群、マイクではない)。黒地/赤白バー。フラット角丸。
- Otter.ai: **サウンドウェーブ/イコライザ**("O"+減衰丸バー)。青グロッシー。
- Notta: **レターマーク "n"**。白n+青→紫/マゼンタ対角グラデ。フラット角丸。
- Granola: **抽象スパイラル**(手描き筆渦)。オリーブ/黄緑地に黒筆。有機フラット。
- Superwhisper: **抽象3D形状**(角丸トーラス/結び目)。モノクロ3Dライティング。
- MacWhisper: **マイク**(クロームstudio mic)。銀+青放射グラデ。スキューモーフィック。
- Whispering(Epicenter): **マイク**(銀メタル studio-mic)。リポジトリ資産。
- Just Press Record: **録音ドット**(赤塗り円+ピンクハロー)。白地。フラット+グロー。

### ジャーナリング・思考系
- Day One: **しおりリボン**(紙日記のブックマーク)。スカイブルー地/白。ミニマル。
- Reflect: **抽象ネットワーク球**(ノード/ナレッジグラフ)。藍〜紫グラデ+発光。
- Mem: **抽象ツイン矢印**。コーラル/赤橙地/白。フラット。
- Rosebud: **薔薇のつぼみ/花弁**(成長・自己理解)。マゼンタ/ラズベリー地。フラット。
- Stoic: **"S." レターマーク**。ほぼ白地に黒グロッシー。
- (参考)Notion: "N"+3Dワイヤーキューブ / Bear: 熊横顔。

### 飽和クリシェ(避けるべき記号)
1. **マイク**(最大クリシェ。MacWhisper/Whispering、特に「青地+銀マイク」)。現行 QuickScribe はここに直撃。
2. **波形/イコライザバー**(Voice Memos/Otter)。
3. **青**(カテゴリ既定色)。
4. **青→紫/ピンク グラデタイル**(Notta。無個性 AI SaaS ルック)。

### 決定的発見
- **ジャーナリング/思考系はマイク・波形を1つも使わない**。音声機能を持つ Rosebud/Day One もマイク不使用。成長/自然・本/しおり・抽象/タイポで「思考の仕事」を表現し、音声は暗黙。
- 音声系で抜きん出るのは録音「装置」を捨てた群(Granola 有機渦 / Superwhisper 黒抽象3D / Just Press Record 赤録音ドット)。
- 含意: マイク+波形は QuickScribe をコモディティ・ディクテーションの山に埋没させ、「文字起こし精度は差別化でない/思考整理ボイスジャーナル」という positioning(ADR-0004)と矛盾。最強参照は **Rosebud(成長)と Day One(レコーダーでなくジャーナル)**。

---

## 6. メタファー候補(記号論的根拠)
記号論: icon(類似:波形は音に似る) / index(痕跡) / symbol(慣習)。クリシェの mic/波形は index/icon に過ぎず「思考の整理」という抽象を表せない → index→symbol へ昇格を狙う。

1. **もつれた糸→一本の滑らかな線**(Untangling thread): もつれ=未整理思考、収束=整理。変換そのものを1枚に描き「整形の知性」という動的価値を直接表す。競合で「変換」を描くものは皆無。リスク: 16-32pxでもつれが潰れる(最小サイズ最弱点)。緩和: もつれ側を2-3ノードに極端簡略化し収束線とのコントラストで読ませる。
2. **波形→整った行/テキストへ変容**: 左=不規則波(音声)、右=等間隔水平線(整理テキスト)。Voice Memos を意図的に引用し裏切る。リスク: 波形を残す以上クリシェに半歩。遠目で「ただの波形」に退行。緩和: 波形を全体1/3未満、整列線を支配的に。
3. **同心円リップル**(inner voice radiating): 内なる声が静かに広がる(内省・共鳴)。calm 系。Just Press Record の上位互換。リスク: Wi-Fi/放送/AirDrop クリシェと衝突(曖昧)。緩和: 非対称な波紋+中心の核で内面性強調。
4. **折られた頁/綴じ目+静かな音の気配**: ジャーナル宣言(Day One系)+頁縁の1-2うねりで「声で書く日記」。リスク: ジャーナル系と近接、頁=タイピング連想で音声体験とズレ。緩和: 頁を抽象化、折り目の陰影で「思考の襞」。
5. **芽吹き/種から伸びる芽**: 思考が育ち自己理解へ(成長・内発)。Rosebud が市場実証。音声系では無人地帯。リスク: 録音との接続が弱く健康/瞑想アプリと混同。緩和: 茎を微かな波形カーブに。

補助モチーフ(主役にしない): 鍵/盾(プライバシー暗示。単独はセキュリティアプリに見える) / 抽象"S"(Stoic 占有済み、意味を語れず非推奨)。

推奨優先度: 1(もつれ→直線, コア価値直接・空白地帯) > 5(芽吹き, 自己理解最強・市場実証) > 3(リップル, calm/内省)。2と4は実制作スパイクで小サイズ検証後に採否。

---

## 検証ステータス・残課題
- 競合サーベイ(§5)は全アプリ実画像目視 VERIFIED(Whispering のみリポジトリ資産)。アイコンは改訂されるため最終決定前に再確認(Day One 2021改訂, Bear 2025末 Liquid Glass)。
- メタファー(§6)は記号論的合成でユーザーテスト未実施。候補1/2/3の最小サイズ可読性は実制作スパイクで実証要。
- 色の推奨は西洋デザイン慣習+差別化論拠であり、いかなる色相も「内省的である」科学的証拠は無い(§1)。小規模知覚テスト(deep indigo vs teal vs warm-clay)で慣習→製品証拠に転換推奨。
- 競合実アートワーク URL は itunes.apple.com/lookup?id=... で取得可。

### 主要ソース
Voice Memos https://apps.apple.com/us/app/voice-memos/id1069512134 / Otter https://otter.ai/media-kit / Notta https://www.notta.ai/en/brand-assets / Granola https://apps.apple.com/us/app/granola-ai-meeting-notes/id6739429409 / Superwhisper https://superwhisper.com/ / MacWhisper https://macwhisper.pressdeck.io/ / Whispering https://github.com/EpicenterHQ/epicenter / Just Press Record https://www.openplanetsoftware.com/just-press-record/ / Day One id=1044867788 / Reflect https://reflect.app/ / Mem https://get.mem.ai/blog/introducing-mem-2-0 / Rosebud https://apps.apple.com/us/app/rosebud-ai-journal-diary/id6451135127 / Stoic https://www.getstoic.com/
