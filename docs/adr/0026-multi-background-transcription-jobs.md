# ADR-0026: 複数バックグラウンド文字起こしジョブの逐次キュー化とジョブ一覧UI

- Status: Accepted（Deciders 承認 2026-07-08。処理モデル=逐次キュー/通知=アプリ内既定を確認、Phase1 実装着手を承認）
- Date: 2026-07-08
- Deciders: Takenori Kusaka
- 関連: #600（長尺末尾欠落）/ ADR-0013（ループバック録音）/ ADR-0014（物理トリガー）/ [ADR-0006 スコープ完全性](0006-scope-completeness-policy.md)
- 一次情報: [マルチジョブUI/UX 調査](../research/multi-job-queue-ui-question-design.md)（deep research・24 confirmed/1 refuted・Apple HIG / NN/g / MDN・WAI-ARIA / Electron・Tauri・MacWhisper・Buzz 公式）

## 背景・課題

ユーザー要件: **「文字起こしには時間がかかる。思考を小分けにするため録音を複数回に分けるが、文字起こし中に次の録音→停止をすると前のジョブが終わってしまう（結果が失われる）。マルチバックグラウンドジョブで取りこぼさないようにしたい。」**

現状のコードを調査した結論:

- **バックエンド（`src-tauri/src/lib.rs` `stop_recording`）は既に非同期・独立実行**。停止ごとに `tauri::async_runtime::spawn` で独立タスクを起こすため、前ジョブのタスク自体は殺されない。
- しかし **全イベント（`progress` / `segment` / `status` / `transcribe-done` / `transcribe-error`）がジョブID無しのグローバル発火**（`lib.rs:343,346,526,529`）。複数ジョブが同時進行すると識別不能。
- **フロント（`App.svelte`）は単一スロット state**（`transcribing` bool / `transcript` / `refined` / `segments` / `progress` / `eta` / `status`）。新規録音開始時（`toggle()` `App.svelte:769-774`）に表示をリセットし、`transcribe-done`（`App.svelte:872`）は単一 `transcript` に上書きする。

→ **実害の真因はバックエンドがジョブを殺すことではなく、UIに"ジョブが1個しか無い"前提の state しか無いこと。** 次の録音を始めた瞬間に走行中ジョブの表示と結果がクロバーされ「消えた」ように見える。

コア価値との関係: QuickScribe のコア価値は「簡便さ」と「思考を止めない」。待たずに次を吐き出せることは価値の中核。ただし「リッチすぎると簡便でなくなる」（CLAUDE.md）ため、**"控えめだが取りこぼさない"** 落としどころを厳守する。

## 決定

### D1. 処理モデル: 逐次キュー（並列度=1・FIFO）を既定 ★Decider確認済み
- whisper は CPU/GPU 重。真の並列デコードはリソース競合で全ジョブが遅くなり、かつ「複数 running の並行進捗を破綻なく見せる」原典事例が調査で見つからなかった（openQuestion）。
- **並列度=1 の逐次キューを既定にすれば、"取りこぼさない" を満たしつつ並行進捗UIの破綻を構造的に回避できる。**
- 将来 opt-in で並列度 N に拡張する余地は設定として残す（ADR-0006 スコープ規律＝削らず段階実装。最終ゴール機能集合は不変）。

### D2. ジョブ一覧UI: 行型リスト＋「最近N件＋履歴」。無限スクロール不採用
- NN/g: 無限スクロールは task-oriented（特定を探す/比較/上位確認）に不適。ジョブ管理はまさに task-oriented。
- 通常運用（1〜数件）はヘッダの控えめなバッジ（「処理中N件」）＋展開で一覧。完了は一覧内に静かに積み上げ。
- 履歴が育った場合のみ履歴ビューに仮想スクロール/ページャを追加（段階実装）。

### D3. ジョブモデル（ユビキタス言語・DDD）
- 状態: **`queued` / `running` / `done` / `error` / `canceled`**（Buzz/Apple HIG 準拠の最小十分セット）。
- ジョブ属性: `id` / `created_at`（録音時刻ラベル）/ `duration`（音声長）/ `status` / `progress`(0-100) / `eta` / `result_text` / `error_code` / `segments`。
- 行の操作: `done`→[開く（整形へ）] / `error`→[再試行] / `queued|running`→[キャンセル]。

### D4. 進捗表示: determinate(%)＋ETA 優先
- Apple HIG/NN/g: 10秒超は percent-done＋残り時間。前処理（デコード/モデルDL）のみ indeterminate。
- 逐次のため画面上 `running` は常に1件。その1件を determinate 進捗＋ETA、他は状態バッジ（queued/done/error）。
- 棄却済み: 「アニメ進捗の方が満足度が高い」説は根拠にしない。

### D5. 通知: アプリ内のみ既定・OS通知はopt-in ★Decider確認済み
- 既定は **アプリ内一覧＋`aria-live="polite"` の live region** で静かに反映（過剰通知を避ける＝簡便さ維持、プライバシー方針とも整合）。
- **OS通知・タスクバー進捗は設定でopt-in**。
- **トレイ左クリックを唯一の入口にしない**（Tauri v2 で Linux 非対応）。アプリ内一覧を主導線に。

### D6. アクセシビリティ
- 動的増減リストは `aria-live="polite"`、live region は空で事前設置し別ステップ更新、削除も読むなら `aria-relevant="additions removals"`（実AT対応は実機検証）。

## 段階実装計画（縦切り・仕様→TDD→DDD）

- **Phase 0（スパイク・本ADR）**: 現状分析・調査・設計確定。★本ファイル。
- **Phase 1（バックエンド・ジョブ識別＋逐次キュー）**:
  - `job` ドメイン導入。`stop_recording` が `job_id` を発番し即返す。全イベントに `job_id` を付与（`{job_id, payload}`）。
  - 逐次キュー（並列度1）: 走行中は後続を `queued` にし順次処理。ジョブレジストリ（`Mutex<Vec<Job>>` 等）で状態管理。
  - 純粋ロジック（キューの状態遷移・次ジョブ選択）は単体テスト（TDD）。
- **Phase 2（フロント・jobsモデル＋一覧UI）**:
  - `jobs[]` state（Svelte 5 runes）へ移行。単一スロット state を廃し、イベントを `job_id` でルーティング。
  - ヘッダのバッジ＋展開一覧・状態列・determinate進捗・操作ボタン・polite live region。
  - i18n 4言語 parity（catalog.test）。coverage branch 80% ゲート維持。
- **Phase 3（キャンセル/再試行/永続化）**:
  - `queued|running` のキャンセル、`error` の再試行。アプリ再起動でキュー状態をどう扱うか（未処理ジョブの復元 or 破棄）を決めて実装。
- **Phase 4（opt-in 拡張）**: OS通知・タスクバー進捗のopt-in、並列度Nの設定。

## 結果・トレードオフ

- **Pro**: 録音を小分けにしても結果を1件も失わない（コア価値の実害解消）。状態の可視性（Nielsen #1）を満たす。既定は控えめでシンプル。
- **Con/リスク**: フロントの state モデル刷新は影響範囲が広い（`App.svelte` 単一スロット依存箇所・テスト）。段階実装とテストで対処。逐次のため、複数投入時は後続の待ち時間が積み上がる（ETA表示で緩和。将来 opt-in 並列で対応）。
- **却下案**: 真の並列デコードを既定（リソース競合・並行進捗UIの破綻・原典事例欠如）。トレイを主入口（Linux非対応）。無限スクロール一覧（task-orientedに不適）。

## 反証・見直し条件

- 実利用でジョブがほぼ常に1件なら、専用一覧UIは過剰 → 「現在1件＋直近履歴」の軽量表示に縮退してよい。
- 履歴が数百件規模に育ったら履歴ビューへ仮想スクロール/ページャ導入。
- 逐次の待ちが実用で苦痛なら opt-in 並列度を前倒し。
