# エージェント並行セッションの排他 — requirements

> Status: Draft (2026-07-27) / 決定: [ADR-0035](../../adr/0035-agent-session-concurrency-control.md) / 運用 SSOT: [docs/process/agent-session-concurrency.md](../../process/agent-session-concurrency.md)
> 参照実装: [ganbari-quest#4009](https://github.com/Takenori-Kusaka/ganbari-quest/pull/4009)（merge 済み `c66022db`）。**移植であって新規設計ではない。**
> 記法: 軽量 BDD ＋ EARS。実装は QS-Dev。

## ユビキタス言語

- **セッション**: Buzz がチャンネルごとに作る ACP セッション。同じエージェントでも複数並走する
- **重い検証**: 数分以上マシンを占有し、並走すると**結果そのものが信用できなくなる**実行（`npm test` / `cargo build` 等）
- **lock**: `~/.buzz/.locks/<key>.lock`。**checkout の外**にあり、worktree が何個あってもマシンで 1 つを見る
- **stale**: 保持者プロセスが死んでいる、または TTL 超過。奪ってよい状態
- **fail closed**: 排他が成立しているか**判定できない**ときに block する側へ倒すこと

## 受入基準（EARS）

### 排他の成立

- **R1（event）**: When a session runs a heavy command while another **live** session holds the `heavy` lock, the hook shall block it with **exit 2** and print 保持者（pid / 経過時間 / cwd / target）と「待たずに別作業へ移る」対処。
- **R2（event）**: When the holder process is dead **or** the TTL has expired, the requesting session shall acquire the lock（stale 奪取）。
- **R3（state）**: While the same session re-enters a heavy command, acquisition shall succeed（再入可能）。
- **R4（unwanted）**: If the lock directory is unreadable **or** the lock file is malformed, then the hook shall **block（exit 2）**, not treat it as stale。想定外の例外も exit 2 に倒すこと（exit 1 は素通しになるため使わない）。
- **R5（event）**: When the heavy command finishes, the `PostToolUse` hook shall release the lock. 解放側は**決して block しない**。

### 判定対象（QuickScribe 固有 — ADR-0035 D1）

- **R6（ubiquitous）**: 重い検証の判定対象は **フロント**（`npm test` / `vitest` / `npm run coverage` / `npm run e2e` / `npm run screenshots` / `npm run check` / `npm run build`）と **Rust**（`cargo test` / `cargo build` / `cargo clippy` / `npm run tauri build`）の両方を含むこと。**Rust 側を落とさない。**
- **R7（unwanted）**: If a command is a read-only inspection whose *arguments* contain a heavy command name（`grep -rn vitest package.json` / `Select-String cargo` 等）, then it shall **not** be blocked。
- **R8（unwanted）**: If a harmless prefix is prepended（`echo start && npm test` / `cd src-tauri; cargo test`）, then it shall **still** be blocked。判定は `&&` / `||` / `;` / `|` の**セグメント単位**で行うこと（全体の先頭トークンだけを見ると前置き 1 つで回避できる）。

### 二重着手の検出（QuickScribe 固有 — ADR-0035 D2 / D3）

- **R9（event）**: When a session runs `git push`, the hook shall acquire a task lock keyed by the current branch。key は **Issue 番号があれば `qs-task-<番号>`、無ければ `qs-branch-<正規化ブランチ名>`**。
- **R10（unwanted）**: If the branch name contains no Issue number（QuickScribe の実際のブランチは `chore/deps-vuln-sweep` / `fix/idle-cpu-measure-window` 等で**番号を含まない**）, then the key shall **not** be `null`。番号前提の実装をそのまま移植すると task lock が一度も効かないまま「導入済み」に見えるため、**この経路にテストを必ず置くこと。**
- **R11（ubiquitous）**: task lock の key は **`qs-` を前置**すること。`~/.buzz/.locks/` は ganbari-quest と共有されており、`task-669` を共有すると無関係な作業を互いにブロックする。
- **R12（ubiquitous）**: `heavy` key は**リポジトリを跨いで共有**すること（前置しない）。負荷はマシン単位で発生するため、分けると防げない。

### 強制点（QuickScribe 固有 — ADR-0035 D4）

- **R13（ubiquitous）**: hook の matcher は **`Bash|PowerShell`** とすること。QuickScribe のエージェントは Windows 上で PowerShell tool も使うため、`Bash` だけでは素通りする。
- **R14（ubiquitous）**: lock 置き場は `AGENT_LOCK_DIR` で差し替え可能とし、**テストが並走中の実 lock を壊さない**こと。
- **R15（ubiquitous）**: lock ファイルに**コマンド文字列そのものを書かない**（引数に混じりうる値を残さないため）。key / pid / branch / cwd / 開始時刻 / TTL に留める。

## BDD 例

```gherkin
Scenario: 他セッションが重い検証中なら止まる (R1)
  Given 生きている別セッションが heavy lock を保持している
  When 自分のセッションが `npm test` を実行しようとする
  Then hook は exit 2 で block し、保持者の pid・経過時間・対処（待たずに別作業へ）を表示する

Scenario: 保持者が死んでいれば奪える (R2)
  Given heavy lock の保持者 pid のプロセスが存在しない
  When 自分のセッションが `cargo test` を実行しようとする
  Then lock を奪取して実行が続行する

Scenario: 壊れた lock は stale 扱いしない (R4)
  Given lock ファイルが JSON として壊れている
  When 重い検証を実行しようとする
  Then hook は exit 2 で block し、「lock が壊れています」と表示する

Scenario: 前置きで回避できない (R8)
  Given heavy lock が別セッションに保持されている
  When `echo start && npm test` を実行しようとする
  Then block される

Scenario: 読み取り専用は誤爆しない (R7)
  Given heavy lock が別セッションに保持されている
  When `grep -rn vitest package.json` を実行する
  Then block されない

Scenario: 番号を含まないブランチでも task lock が効く (R9,R10)
  Given 現在のブランチが `chore/deps-vuln-sweep` である
  When `git push` を実行しようとする
  Then key `qs-branch-chore-deps-vuln-sweep` で lock を取得する（key は null にならない）

Scenario: 同じブランチの二重 push が止まる (R9)
  Given 別セッションが同じブランチの task lock を保持している
  When `git push` を実行しようとする
  Then block され、「二重作業。チャンネルで担当を確認する」旨が表示される

Scenario: PowerShell 経由でも効く (R13)
  Given heavy lock が別セッションに保持されている
  When PowerShell tool から `npm test` を実行しようとする
  Then block される
```

## 段階実装（ADR-0006 — 削らずに分ける）

| 段 | 内容 | 完了条件 |
|---|---|---|
| **段 1** | 判定 pure function ＋ lock 実体 ＋ 単体テスト（R6〜R12 を全部固定。**hook はまだ配線しない**） | vitest で全 Scenario 相当が pass |
| **段 2** | `PreToolUse` / `PostToolUse` hook を配線（R1〜R5・R13） | hook に stdin JSON を流す probe で block / 奪取 / fail closed / 解放を実測 |
| **段 3** | `docs/process/agent-session-concurrency.md` §7 に検証手順を記載し、CLAUDE.md から参照を張る | 参照が張られていること |

**検出範囲を狭めることによるリスク回避は採らない**（ADR-0006）。誤爆が出たら対象を削るのではなく判定を精緻化する。

## 検証（QS-Dev が実測して報告する）

- 単体: `npx vitest run <テストパス>` — **並走を確認してから回す。** 他セッションが重い検証中なら回さずに報告する（この仕様が防ごうとしている状態そのものである）
- hook: stdin JSON を流す probe で exit code を実測。**パイプで exit code を殺さない**（`cmd | tail` は起動失敗でも exit 0 になる）
- `.claude/settings.json` を追加するため、**既存の hook 設定（`~/.buzz/.claude/settings.json` にオーナーが置いているもの）と二重に走らないか**を確認して報告すること

## 範囲外

- PR 単位の排他（`gh pr merge` / `gh pr edit`）— 別 key が要る。実測された事故が無いので本増分に含めない
- 待機キュー / 優先度 — 「待たない」が運用方針なので不要
- 人間の直接実行の排他 — hook の届く範囲外（[運用 SSOT §6](../../process/agent-session-concurrency.md)）
