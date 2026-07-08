# マルチジョブ・キューUI/UX 調査（問い設計＋deep research 結果）

> 対象: 複数のバックグラウンド長時間ジョブ（音声文字起こし）を「取りこぼさず」管理するジョブキューの UI/UX・画面設計。
> 方法: ADR-0007 の[問い設計メソッド](question-framing-method.md)で問いを設計 → deep research（fan-out検索→一次ソース取得→3票敵対的検証→統合）。
> 実行日: 2026-07-08。検証: 25 claims 中 24 confirmed / 1 refuted。一次ソース中心（Apple HIG / NN/g / MDN・WAI-ARIA / Electron・Tauri 公式 / MacWhisper・Buzz 公式）。

## 1. ジョブ接続（コア価値への錨）

When 思考整理のため録音を小分けに複数回行うとき、I want to 各文字起こしを並行してバックグラウンド処理し1件も失わず結果を受け取りたい、so I can 待たずに次の思考を吐き出し続けられる（＝簡便さ・コア価値）。
QuickScribe は「リッチすぎると簡便でなくなる」バランスが設計の核心（CLAUDE.md）。よって狙いは **「控えめだが取りこぼさない」** UI の落としどころ。

## 2. 意思決定（この調査で変える決定）と結論

| 決定 | 結論（推奨） | 主要根拠 | 反証条件（考えが変わる条件） |
|---|---|---|---|
| **D1 UIメタファ** | アプリ内の**行/テーブル型ジョブ一覧**を主入口に。トレイ/OS進捗は補助。 | NN/g「システム状態の可視性」/ Buzz・MacWhisper 実装 | 実利用でジョブ数がほぼ常に1件なら、専用一覧は不要で「現在1件＋直近履歴」の軽量表示で足りる |
| **D2 ページネーション** | **無限スクロール不採用**。「最近N件＋履歴（折りたたみ/別ビュー）」。件数が多い履歴のみ仮想スクロール。 | NN/g: 無限スクロールは task-oriented(探す/比較/上位確認)に不適 | 履歴が数百件規模に育つ実データが出たらページャ or 仮想スクロールを履歴ビューに追加 |
| **D3 ジョブ情報・状態** | 状態モデル= **queued / running / done / error / canceled**。行に「ラベル(録音時刻/長さ)・状態・進捗・操作(開く/再試行/キャンセル)」。 | Apple HIG(進捗・キャンセル)/ Buzz(行＝ジョブ＋状態列) | — |
| **D4 進捗の同時多発** | 測れる限り **determinate(%)＋ETA**。前処理のみ indeterminate。複数running時は各行に細い進捗、詳細は1件展開。 | Apple HIG / NN/g(10秒超はpercent-done) | 「動的アニメの方が満足度が高い」説は**棄却済み**(下記refuted)。アニメの優位は根拠にしない |
| **D5 完了/失敗通知** | 既定は **アプリ内(一覧＋polite live region)**。OS通知/タスクバー進捗は opt-in 補助。**トレイ左クリックを唯一の入口にしない**。 | MDN(aria-live polite)/ Electron setProgressBar / Tauri: トレイ左クリックはLinux非対応 | — |

## 3. 確定した知見（confirmed findings）

1. **10秒超はバックグラウンド前提**: NN/g は10秒を注意持続の限界とし、超過時は「他タスクに移りたがる」。長い処理はバックグラウンド化し作業を続けられる設計に。分単位の文字起こしは常にこの帯。
   - https://www.nngroup.com/articles/response-times-3-important-limits/ / https://www.nngroup.com/articles/designing-for-waits-and-interruptions/
2. **determinate優先＋ペースを均す**: Apple HIG「可能な限り determinate を。非線形ペース(90%→残10%が長い)は動いていないと疑わせ欺瞞的」。NN/g「10秒超は percent-done が最も情報量が多くコントロール感を与える」。
   - https://developer.apple.com/design/human-interface-guidelines/progress-indicators / https://www.nngroup.com/articles/progress-indicators/
3. **待ち時間帯の指標割当**: `<1s` 明示不要 / `2-10s` indeterminate / `10s+` percent-done＋明示キャンセル。文字起こしは常に最後。（厳密閾値は NN/g 準拠が安全。3-10s帯にdeterminateとするブログ配分は境界差ありcaveat）
4. **一覧は無限スクロール不適**: NN/g「無限スクロールは目的なく均質項目を眺める用途に限定。特定を探す/比較/上位数件確認には非推奨」。ジョブ管理は後者。
   - https://www.nngroup.com/articles/infinite-scrolling-tips/ / https://www.nngroup.com/articles/infinite-scrolling/
5. **実アプリは「バッチ/キュー＋行＝ジョブ＋状態列」**: MacWhisper は Batch ウィンドウ(合計時間・ETA・順次処理)、Buzz は状態列付きの行、Completed 行をダブルクリックで開く。
   - https://docs.macwhisper.com/article/19-batch-transcription / https://github.com/chidiwilliams/buzz / https://chidiwilliams.github.io/buzz/docs/usage/file_import
6. **OS統合は補助に**: Electron `setProgressBar(0-1)` は非前面でもタスクバー/Dock進捗を表示(>1でWin=indeterminate、負値で除去)。Tauri v2 はトレイ左クリックが Linux 非対応。よってアプリ内一覧を主、OS進捗/通知を補助に。
   - https://www.electronjs.org/docs/latest/tutorial/progress-bar / https://v2.tauri.app/learn/system-tray/
7. **動的増減リストのa11y**: `aria-live="polite"` 既定(assertiveは緊急のみ)、live region は空で事前設置し別ステップで更新、削除も読むなら `aria-relevant="additions removals"`(既定では削除は読まれない)。
   - https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Guides/Live_regions
8. **横断原則**: Nielsen ヒューリスティック#1「システム状態の可視性」。「控えめだが取りこぼさない」は、状態を隠さず・過剰通知もしないこの原則の実装。
   - https://www.nngroup.com/articles/visibility-system-status/

## 4. caveats / 反証・棄却

- **棄却(refuted, 1-2)**: 「動的(アニメ)進捗の方が静的より満足度が高く長く待てる」→ 敵対的検証で棄却。アニメーション自体の優位性は設計根拠にしない。
- 3-10秒帯にdeterminateを割り当てる配分はブログ由来。厳密閾値は NN/g（`<1s`無し / `2-10s` indeterminate / `10s+` percent-done＋キャンセル）を採る。
- `aria-relevant="additions removals"` は仕様上正しいが実AT対応にばらつき→実機検証必須。
- MacWhisper「順次処理・最大20ファイル」等の運用詳細は二次記事依存でバージョン変動あり。
- Tauri トレイ左クリック Linux 非対応は v2 時点の挙動。

## 5. 未解決の問い（openQuestions → 設計時にスパイク/実測で埋める）

1. QuickScribe 実利用のジョブ数分布（1件で足りるのか、複数常態か）— D1メタファの最終確定に必要。
2. 複数 running 同時の進捗を破綻なく見せる具体レイアウト（1件展開＋他は集約バッジ等）の一次事例が不足。MacWhisper/Buzz は主に順次処理で、真の並行進捗UIの原典が未特定 → **並列度=1（逐次キュー）を既定にすればこの問題は回避できる**（設計含意）。
3. 完了/失敗通知の「簡便さを損なわない既定」と OS 通知権限フローの両立（プライバシー方針との整合）。
4. 部分結果(segment)ストリーミングの暫定テキスト視覚区別＋aria-live頻度制御の具体パターン。

## 6. QuickScribe への設計含意（次アクション＝ADR-0026）

- **処理モデル**: whisper は CPU/GPU 重。**逐次キュー（並列度=1、FIFO）を既定**にすれば「取りこぼさない」を満たしつつ、openQuestion#2（並行進捗の破綻）を構造的に回避できる。将来 opt-in で並列度を上げる余地は残す（スコープ規律：削らず段階実装）。
- **バックエンド**: 停止ごとに `job_id` を発番し、全イベント(progress/segment/status/done/error)に `job_id` を付与。ジョブレジストリで一覧/状態を保持。
- **フロント**: 単一スロット state を廃し `jobs[]`（id/ラベル/状態/進捗/ETA/結果/エラー）へ。行型一覧＋状態列＋determinate進捗、polite live region。新規録音で走行中ジョブをリセットしない。
- **既定は控えめに**: 通常運用（1〜数件）ではヘッダ近くに「処理中N件」の控えめなインジケータ＋展開でリスト。OS通知・タスクバー進捗は opt-in。
