# エージェント並行実行 — セッション分離と排他（運用 SSOT）

> **位置づけ**: QuickScribe を触る AI エージェントが**同一マシンで複数セッション並走**することを前提に、何が壊れるか・何を機械強制しているか・止められたとき何をするかを定める。決定の「なぜ」は [ADR-0035](../adr/0035-agent-session-concurrency-control.md)。受入基準は [docs/specs/agent-session-concurrency/requirements.md](../specs/agent-session-concurrency/requirements.md)。

## §1 前提 — セッションは 1 本ではない

Buzz は**チャンネルごとに ACP セッションを作る**。同じエージェント（QS-PO / QS-Dev）でも、参加チャンネルの数だけセッションが並走しうる。セッションは互いを知らず、共有しているのは**同じマシンと同じ checkout 群**だけである。

**自制では調整できない。** 自分の残存プロセスは片付けられるが、他セッションのプロセスは kill してはならない（相手が引用しようとしている証跡を壊す破壊的操作にあたる）。

## §2 2 層で防ぐ

| 層 | 防ぐもの | 手段 |
|---|---|---|
| worktree 分離 | ファイルの相互上書き | チャンネル専用 worktree |
| lock | マシン資源の奪い合い（＝汚染された検証結果）・同一 branch の二重着手 | `~/.buzz/.locks/` ＋ Claude Code hook |

**片方では足りない。** worktree を分けてもマシンは 1 台であり、lock を取ってもファイルは同じ checkout を指しうる。

## §3 worktree 分離の規約

1. `[Context]` のチャンネル UUID 先頭 8 桁を `<cid8>` とする（例: `450a18f8`）
2. worktree が無ければ作る。**ブランチ名は GitHub Flow（`feat/*` `fix/*` `docs/*` `chore/*`）に従う**（[CLAUDE.md](../../CLAUDE.md) 開発フロー）

   ```bash
   git -C <repo> worktree list
   git -C <repo> worktree add .claude/worktrees/ch-<cid8> -b <type>/<topic>
   ```

3. `EnterWorktree` ツールが使えるならセッションを移す。使えない場合は以降のファイル操作・git 操作を**すべて worktree 配下の絶対パス**で行う
4. 同じチャンネルで再開したときは**同じ `ch-<cid8>` に戻る**（作業が継続する）

**読み取り専用の調査（ログ確認・コード閲覧・`git log`）は本体クローンのままでよい。**

**worktree は削除しない。** 他セッションが証跡として参照している可能性がある。整理はオーナーが行う。既存の `E:/Github/qs-wt-*` は旧規約の worktree であり、そのまま残す（新規は `.claude/worktrees/ch-<cid8>` に寄せる）。

## §4 機械強制している排他

### §4.1 lock の実体

| | |
|---|---|
| 置き場 | `~/.buzz/.locks/<key>.lock`（**repo の外**。checkout / worktree が複数あっても同じマシンなら同じ lock を見る。ganbari-quest と共有する） |
| 強制点 | `PreToolUse` hook で取得 / `PostToolUse` hook で解放。matcher は **`Bash|PowerShell`**（PowerShell 経由の実行を素通ししないため） |
| 環境変数 | `AGENT_LOCK_DIR` で置き場を差し替え可（テストが実 lock を壊さないため） |

| key | 対象 | 粒度 | TTL |
|---|---|---|---|
| `heavy` | 下記の重い検証（**ganbari-quest と共有**） | マシン全体で 1 本 | 60 分 |
| `qs-task-<Issue番号>` / `qs-branch-<slug>` | `git push`（ブランチ名から導出） | ブランチ単位 | 4 時間 |

**`heavy` を他リポジトリと共有するのは意図どおり**である（負荷はマシン単位で発生する）。**task key に `qs-` を前置するのは必須**である（QuickScribe #669 と ganbari-quest #669 は別物）。

### §4.2 重い検証の対象

フロント: `npm test` / `vitest` / `npm run coverage` / `npm run e2e`（wdio）/ `npm run screenshots`（playwright）/ `npm run check`（svelte-check）/ `npm run build`

Rust: `cargo test` / `cargo build` / `cargo clippy` / `npm run tauri build`

**Rust 側を外さない。** whisper-rs のビルドは QuickScribe で最も重く、ここを抜くと排他の意味が半減する。

### §4.3 保持者の生存判定

lock の持ち主は Claude セッションのプロセス（hook から見た `process.ppid`）。

- 持ち主が死んでいれば lock は stale として**奪える**。セッション断で lock が残り続けることはない
- TTL は「プロセスは生きているが処理が終わらない」場合の保険であり、生存判定の代替ではない
- 同じセッションからの再取得は成功する（再入可能）

### §4.4 判定できないときは通さない（fail closed）

lock ディレクトリが読めない、lock ファイルが壊れている等、**排他が成立しているか判定できない**状態では block する。判定できないまま重い検証を走らせると、汚染された結果を根拠に使ってしまう。

`PreToolUse` は **exit 2 のみが block** で、exit 1 は tool 実行が継続する（＝素通し）。全経路を try/catch で囲み、想定外の例外も exit 2 に倒す。

解放側（`PostToolUse`）は block しない。**正しさの担保は lock 側（生存判定 ＋ TTL）にあり、解放 hook は早期返却の最適化**である。

## §5 止められたときにすること

**待たない。** 待機で turn を潰さず、別の作業に移る。

1. チャンネルに「他セッションが重い検証中のため見送った」と報告する（**どの lock に当たったかを書く**）
2. PR 本文整備 / Issue 起票 / レビュー対応など、マシンを占有しない作業に移る
3. CI で代替できるなら**ローカル実行を諦めて CI を正とする**。ローカル完走が必要なのは「CI に無い gate」を回すときだけ

`qs-task-*` / `qs-branch-*` で止められた場合は**二重作業**である。チャンネルで担当を確認し、**どちらが進めるかを決めてから**再実行する。

## §6 適用範囲と限界

- **hook が効くのは Claude Code 経由の Bash / PowerShell のみ**。人間が直接ターミナルで叩く分には効かない。オーナーが手で重い検証を回すときは、エージェントが動いていないことを確認する
- **`gh pr merge` / `gh pr edit` は task lock の対象外**。PR 番号で他人の PR を操作するコマンドで、自分の branch と対応しない
- **判定は文字列マッチ**。セグメント（`&&` / `;` / `|`）単位で判定するため無害な前置きでは回避できないが、**新しい重量コマンドを足したら判定パターンの更新が要る**
- **lock は「同時に走らせない」だけを保証する。** 内容の衝突（同じファイルを別ブランチで別々に直す）は防げない。それは worktree 分離と担当分けの領分である

## §7 検証

実装後にここへ検証コマンドを記載する（[受入基準](../specs/agent-session-concurrency/requirements.md) §検証）。lock 置き場は `AGENT_LOCK_DIR` で temp へ逃がすため、**テスト実行が並走中の実 lock を壊すことはない**。
