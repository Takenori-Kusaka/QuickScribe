# 競合ランドスケープ分析

> 取得日: 2026-06-28。原典主義（[ADR-0007](../adr/0007-research-question-framing-method.md)）に基づき各社公式サイト/公式ドキュメントを一次情報として優先。公式で確証できない点は「未確認」と明記する。
> 関連: [vision.md](../vision.md)（差別化4軸）／ [ADR-0004](../adr/0004-product-positioning-voice-journal.md)（ポジショニング）。価格は変動が激しいため取得日のスナップショット。

## 1. 比較表

| 製品 / OSS | 主な用途 | 文字起こし方式（既定） | プライバシー既定 | 整形の知性 | 価格 / ライセンス | プラットフォーム |
|---|---|---|---|---|---|---|
| **OpenWhispr** | 汎用音声入力＋AIメモ | ローカル(Whisper/Parakeet)とクラウド(BYOK)選択制・**既定ローカル** | 既定で端末外に出さない・テレメトリ既定オフ・学習なし明言 | 清書・後処理寄り（思考整理は未確認） | **OSS/MIT**（無料・商用可） | macOS/Windows/Linux |
| **Otter.ai** | 会議文字起こし・議事録 | **クラウドのみ**（Zoom/Teams/Meet連携） | クラウド送信前提・米国ホスティング | 会議要約・アクション抽出（ジャーナリング用途でない） | Free〜Pro $16.99/月〜Business $30/月 | Web/iOS/Android |
| **Granola** | AI会議ノート | 端末上リアルタイム転写→**音声は即削除**・整形LLMはクラウド | 音声即削除・第三者AIの学習を契約で禁止（自社学習の既定は**未確認**） | 会議ノートのEnhance（会議文脈・ジャーナリングでない） | Free〜$14〜$35/月 | macOS/Windows/iOS |
| **Windows 音声入力 / Voice Access / Copilot** | OS統合の汎用音声入力/操作 | ハイブリッド（Win+H既定オンライン認識／Copilot+ PCはNPUローカルSLM） | 機能で異なる・許可なく録音保存しないと明言 | Fluid Dictation=フィラー除去/句読点/文法（リアルタイム清書） | OS同梱（一部Copilot+ PC要件） | Windows 11/12 |
| **superwhisper** | 汎用音声入力（Mac中心） | **ローカルWhisper既定**・クラウドLLMモード別 | 既定ローカルだが音声ディスク保存既定・クラウドモードの送信を指摘（**第三者情報・要確認**） | Custom Modesでアプリ別整形プロンプト | Free〜Pro $8.49/月〜Lifetime（諸説） | macOS/iOS |
| **MacWhisper** | ファイル/会議文字起こし（Mac） | **ローカルWhisper既定**・クラウドAIはBYOK | 既定ローカル完結「never phones home」 | 転写との対話・要約（BYOK） | Free〜買い切り約$69〜$149 | macOS |
| **Day One** | **ジャーナリング**（音声・AI整形） | 音声添付のAI転写・一部オンデバイス(Apple Intelligence)・他クラウド | 暗号化保管・AI利用時に一時復号しHTTPS送信・学習なし明言（機能でクラウド） | エントリ要約・Daily Chat・Go Deeper（思考整理に最も近い既存例） | サブスク＋Gold(AI) $74.99/年 | iOS/macOS/Android |
| **Obsidian + Whisperプラグイン** | 汎用ノート（自作でジャーナルも） | プラグイン依存・クラウドAPI既定だがローカル(whisper.cpp/Ollama)も可 | 構成依存（完全ローカル可だが手間はユーザー次第） | 後処理LLMで整形・アクション抽出（自作前提） | Obsidian個人無料・プラグインOSS中心 | Win/Mac/Linux/モバイル |

## 2. QuickScribe の差別化4軸 × 競合の空白

### ① 用途特化（ジャーナリング）
会議特化（Otter・Granola）と汎用音声入力（OpenWhispr・superwhisper・MacWhisper・Windows音声入力）が市場の大半。**「ジャーナリング専用」を正面に据えた製品はほぼ空白**。隣接する唯一の例が **Day One**（ジャーナル×AI）だが、既存のテキスト中心ジャーナルにAIを後付けした構造で、音声→思考整理を一次体験に置いていない。汎用ツールは「カーソルに文字を挿入する」が目的で、書いた内容を内省素材として育てる設計思想がない。**→ 用途特化は明確な空白。最大の隣接競合は Day One。**

### ② ニュアンス保持の整形（思考整理の知性）
整形はコモディティ化が進行。Windows Fluid Dictation／superwhisper／OpenWhispr／MacWhisper／Obsidianプラグインはいずれも**清書（cleanup）**にとどまる。Otter/Granola は**会議要約・アクション抽出**に強いが、これは「決定事項を圧縮する」方向＝QuickScribeが狙う「ニュアンスを残す思考整理」とは逆ベクトル（**要約は捨てる、QuickScribeは残す**）。思考整理（自己理解の促進）に踏み込むのは Day One Gold の Go Deeper/Daily Chat のみだが、Day Oneはクラウド前提でローカル完結でない。**→ 「ローカルでニュアンス保持の思考整理」は実質空白。**

### ③ ローカルプライバシー（既定で音声を端末外に出さない）
ここは**最も競争が激しい**。OpenWhispr（既定ローカル・テレメトリオフ・学習なし）と MacWhisper（"never phones home"）は QuickScribe と同等。superwhisper はローカル既定だが既定の穴が指摘される。Granola は音声即削除だが整形LLMはクラウド。Otterは完全クラウド。**→ ローカルプライバシー単体は差別化になりにくい（コモディティ化、[vision](../vision.md) の認識と一致）。①②④と束ねて初めて価値化される。**

### ④ 物理ボタン統合体験
調査範囲の全競合で、**専用物理ボタン（ハードウェア）統合を主機能に据えた製品は確認できず**（網羅確認ではない＝「主機能でない」確認にとどまる）。各社の起動手段はグローバルホットキーやウェイクワードにとどまる。**→ 競合空白が最も明確な軸。** ただしハード前提の参入障壁/UX価値の市場検証は別途要検証。

## 3. 総合所見：QuickScribe が立つべきポジション

- **単軸では勝てない（条件付きの勝機）。** ローカル(③)・清書(②の一部)・用途特化(①)はそれぞれ既存プレイヤーが部分的に埋めている。
- **勝機は4軸の「束ね方」にある。** とりわけ **「ローカル完結(③) × ジャーナリング特化(①) × 要約でなくニュアンス保持の思考整理(②)」** の交差点は、調査範囲で**該当製品なし**。Day Oneは①②に踏み込むが③が不完全、OpenWhispr/MacWhisperは③が強いが①②が弱い。
- **最警戒すべき競合は Day One（特にGoldのAI路線）と OpenWhispr。** Day Oneがオンデバイス整形を強めると②③①の交差点に最接近。OpenWhisprは土台が強く、用途特化に振れば①へ侵食しうる。
- **物理ボタン(④)は最も白い空白**だが、コア価値の代替ではなく**体験の参入障壁/想起トリガー**として位置づけるのが妥当。
- **整形＝清書/要約はコモディティ化が進行中**（Windows Fluid Dictation がOS標準で無料提供）。QuickScribeは「整形」を清書/要約と差別化し、**“要約は捨てる、QuickScribeは残して育てる”** 思考整理の方向性を明確にメッセージングする必要がある。

## 4. 未確認 / 要追検証（憶測で埋めない）

- **Granola の自社モデル学習の既定**: 公式は「第三者AIの学習を契約で禁止」と明言するが、第三者レビューは「Basic/Businessは既定でユーザーデータが学習対象・組織一括オプトアウトはEnterpriseのみ」と主張。公式一次情報で確証できず**要追検証**。
- **superwhisper**: 公式の privacy/pricing 原文を未取得。価格・「音声ディスク保存が既定」等は第三者依拠。
- **物理ボタン統合(④)**: 全競合に「ない」ことの網羅確認はしていない。
- **Windows 12 固有の音声機能**: 一次情報で確認できず（現状は Windows 11 + Copilot+ PC の文脈）。
- **OpenWhispr のライセンス**: privacyページは「open source」のみ。MIT はリポジトリ説明依拠（LICENSE直接確認は未実施）。

## 5. 出典（一次情報優先・取得日 2026-06-28）

- OpenWhispr: <https://openwhispr.com/> ・ <https://openwhispr.com/privacy> ・ <https://github.com/OpenWhispr/openwhispr>
- Otter.ai: <https://otter.ai/pricing>
- Granola: <https://www.granola.ai/> ・ <https://www.granola.ai/pricing> ・ <https://www.granola.ai/blog/granola-pricing-privacy-tradeoff>
- Windows 音声・Copilot: <https://support.microsoft.com/en-us/windows/speech-voice-activation-inking-typing-and-privacy-149e0e60-7c93-dedd-a0d8-5731b71a4fef>
- superwhisper（第三者レビュー）: <https://www.getvoibe.com/resources/superwhisper-review/> ・ <https://spokenly.app/comparison/superwhisper>
- MacWhisper: <https://macwhisper.org/> ・ <https://www.todayonmac.com/macwhisper-your-private-transcription-assistant-that-never-phones-home/>
- Day One: <https://dayoneapp.com/guides/labs/ai-features/> ・ <https://dayoneapp.com/features/>
- Obsidian + Whisper: <https://github.com/nikdanilov/whisper-obsidian-plugin> ・ <https://community.obsidian.md/plugins/whisper>
