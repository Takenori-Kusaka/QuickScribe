# ADR-0035: エージェント並行セッションの排他（worktree 分離 ＋ マシン単位 lock）

- Status: Proposed
- Date: 2026-07-27
- Deciders: kokoro-dev（オーナー）
- 関連: [ADR-0006](0006-scope-completeness-policy.md)（スコープ完全性）/ [docs/process/agent-session-concurrency.md](../process/agent-session-concurrency.md)（運用 SSOT）/ [docs/specs/agent-session-concurrency/requirements.md](../specs/agent-session-concurrency/requirements.md)（受入基準）
- 参照実装: [Takenori-Kusaka/ganbari-quest#4009](https://github.com/Takenori-Kusaka/ganbari-quest/pull/4009)（merge 済み・commit `c66022db`）

## Context

### 前提: 1 エージェント = 1 セッションではない

QuickScribe の開発は Buzz 上の複数エージェント（QS-PO / QS-Dev）が担う。Buzz は**チャンネルごとに ACP セッションを作る**ため、同じエージェントでもチャンネル数だけセッションが並走しうる。セッションは互いの存在を知らず、共有しているのは**同じマシンと同じ checkout 群**だけである。

QuickScribe の checkout は 2026-07-27 時点で本体 1 つ（`E:/Github/QuickScribe`）と worktree 5 つ（`qs-wt-666` / `qs-wt-667` / `qs-wt-adr0033` / `qs-wt-cpuwin` / `qs-wt-deps`）である（`git worktree list` 実測）。**checkout が分かれていてもマシンは 1 台**なので、ファイル衝突を防いでも CPU / メモリ / ディスク I/O の奪い合いは残る。

### 何が壊れるか

| 事象 | 出所 | 状態 |
|---|---|---|
| 同一 worktree を 2 セッションが相互上書きし作業が消失 | 2026-07-26・QuickScribe 実測（エージェントの運用ルールに記録済み） | **QuickScribe で発生済み** |
| 重い検証の並走で単独 17 分の全ユニットが 29 分に伸び、`Test timed out` 5 件・assertion failure 0 件 | ganbari-quest 実測（[#4009 の docs/sessions/agent-concurrency.md §2](https://github.com/Takenori-Kusaka/ganbari-quest/pull/4009)） | **QuickScribe では未実測**。同一マシン・同一の並走構造なので同種の汚染が起こりうる、という推論である |
| 同じ branch / 同じ Issue に 2 セッションが着手する二重作業 | ganbari-quest 実測 | QuickScribe では未実測 |

並走の害は「遅くなる」ではない。**結果そのものが根拠として使えなくなる**ことである。落ちても通っても、実装のせいか負荷のせいか切り分けられない。汚染された結果を PR や ADR の根拠として引用すると、誤診が下流へ伝播する。

### なぜ自制では足りないか

自分の残存プロセスは片付けられるが、**他セッションのプロセスは kill できない**（相手が引用しようとしている証跡を壊す破壊的操作にあたる）。「重い検証の前に他セッションの有無を確認する」という運用ルールは既にエージェント側に入っているが、確認の抜けを検出する手段が無く、抜けたことに誰も気付けない。**機械強制が要る。**

## Decision

**2 層で防ぐ。両方が要る。**

| 層 | 防ぐもの | 手段 |
|---|---|---|
| **worktree 分離** | 同一ファイルの相互上書き | チャンネル専用 worktree（`.claude/worktrees/ch-<チャンネル UUID 先頭 8 桁>`）で作業する |
| **マシン単位 lock** | マシン資源の奪い合い（＝汚染された検証結果）と、同一 branch の二重着手 | `~/.buzz/.locks/<key>.lock` を Claude Code の `PreToolUse` hook で取得し、`PostToolUse` で解放する |

lock の実装は ganbari-quest#4009 を**移植**する（判定 pure function / lock 実体 / hook I/O の 3 層分離、保持者プロセスの生存判定 ＋ TTL、判定不能時は fail closed）。設計の是非は同 PR で決着済みなので、本 ADR では QuickScribe 固有の差分だけを決める。

### 移植時に変える 4 点（そのまま持ってくると効かない箇所）

**D1. 重い検証コマンドの集合が違う。Rust 側を必ず含める。**

QuickScribe の `package.json` scripts 実測（`npm test` = `vitest run` / `coverage` / `e2e` = wdio / `screenshots` = playwright / `check` = svelte-check / `build` = vite build）に加え、**Rust 側（`cargo test` / `cargo build` / `cargo clippy` / `npm run tauri build`）を対象に含める。** whisper-rs のビルドは QuickScribe で最も重い処理であり、ここを外すと排他の意味が半減する。

**D2. task lock の key をブランチ名から導く。Issue 番号に依存させない。**

ganbari-quest の `taskKeyFromBranch` は `feat/3963-...` のように**ブランチ名に Issue 番号が入る**命名を前提にしている。QuickScribe の実際のブランチは `chore/deps-vuln-sweep` / `fix/idle-cpu-measure-window` / `feat/staged-entry-list` / `docs/adr-0033-webview2-residency` で、**番号を含まない**（`git worktree list` 実測）。そのまま移植すると key が常に `null` になり、**task lock が一度も効かないまま「導入済み」に見える** — 最も避けたい失敗モードである。

→ **番号があれば `task-<番号>`、無ければブランチ名を正規化した `branch-<slug>` を key にする。** 二重着手の検出という目的には、ブランチ名の一致で十分である。

**D3. lock の key 名前空間にリポジトリを前置する。**

`~/.buzz/.locks/` は**マシン全体で共有**される（2026-07-27 時点で ganbari-quest 由来の `heavy.lock` / `task-4004.lock` が実在することを確認済み）。`heavy` を共有するのは**意図どおり**である（実測された害は同一マシンの負荷由来なので、リポジトリごとに分けると効かない）。

一方 **task key は分けなければならない。** QuickScribe の Issue #669 と ganbari-quest の Issue #669 は別物であり、`task-669` を共有すると無関係な作業を互いにブロックする。→ QuickScribe 側の task key は `qs-task-<番号>` / `qs-branch-<slug>` とする。

**D4. hook の matcher に PowerShell を含める。**

ganbari-quest の `.claude/settings.json` は `"matcher": "Bash"` のみである（実測）。QuickScribe のエージェントは Windows 上で動いており **PowerShell tool も使う**。`Bash` だけを見張ると、PowerShell 経由の `npm test` / `cargo build` が素通りする。→ matcher は `Bash|PowerShell` とする。

### スコープ規律（ADR-0006）

検出対象を「軽いものだけ」「フロントだけ」に絞るのは**スコープ縮小にあたり、採らない。** 重い順に段階実装（フェーズ分割）で全部入れる。誤爆が出たら検出範囲を削るのではなく、判定（読み取り専用コマンドの除外・セグメント単位判定）を精緻化して解く。

## Consequences

**得るもの**: 検証結果が「実装の状態」をそのまま表すようになる。二重作業に費やされるマシン時間と往復が減る。止められた側は待つのではなく、マシンを占有しない作業（PR 本文整備・Issue 起票・レビュー対応）へ移る。

**失うもの・受け入れるコスト**:

- これまで通っていた「他セッションが重い検証中の `npm test` / `cargo test`」が exit 2 で止まる。**意図した挙動変更**である。想定外の停止が出たら `.claude/settings.json` から該当 hook を外すだけで元に戻せる
- `heavy` key はマシン全体で 1 本なので、**QuickScribe と ganbari-quest の軽い検証同士も塞ぐ。** 粒度をリポジトリ単位に割る案は採らない（実測された害はマシン負荷由来で、リポジトリで分けると防げない）
- 全 Bash / PowerShell 呼び出しで hook が走るため、通常経路のオーバーヘッドが乗る

**検証できていないこと**:

- QuickScribe 上で並走が偽の red を作ることは**未実測**である。本 ADR はその実測を待たずに導入する判断であり、根拠は (a) 同一マシン・同一の並走構造であること (b) 相互上書きは QuickScribe で既に発生していること (c) 導入コストが移植で済むこと、の 3 点である

## 却下した代替案

| 案 | 却下理由 |
|---|---|
| 運用ルール（自制）のみで回す | 既に運用ルールとして存在するが、抜けを検出する手段が無い。2026-07-26 の相互上書きは運用ルールがある状態で起きた |
| lock をリポジトリ内（`.git/` や `tmp/`）に置く | worktree が複数あると別ファイルになり、同一マシンの負荷を防げない。checkout の外に置く必要がある |
| `heavy` key をリポジトリ単位に分ける | 負荷はマシン単位で発生する。分けると「両リポジトリで 1 本ずつ = 2 本並走」を許してしまう |
| CI に寄せてローカル検証を禁止する | CI に無い gate（Windows 実機の手順など）が回せなくなる。スコープ縮小にあたる |
