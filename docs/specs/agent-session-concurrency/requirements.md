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
- **R2（event）**: When the TTL has expired **or** the holder's `ownerPid` resolves to a process that is no longer alive, the requesting session shall acquire the lock（stale 奪取）。**`ownerPid` は R19 を満たす長寿命 pid であることが前提**で、これを満たせないうちは TTL のみで判定すること（短命 pid を生存判定に使うと自分の lock を自分で奪う）。
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

### 登録点（QuickScribe 固有 — ADR-0035 D5）

- **R16（ubiquitous）**: hook の登録は**セッションの cwd に依存しない形**であること。Buzz エージェントのセッションの cwd は `C:\Users\kokor\.buzz` であり、リポジトリ root ではない。repo 側 `.claude/settings.json` の相対パス登録は**一度も発火しない**。
- **R17（ubiquitous）**: 登録の正は**オーナー領域 `~/.buzz/.claude/settings.json`** とし、コマンドは**絶対パス**で書くこと。repo 側 `.claude/settings.json` も併せて置く（cwd が repo root になる起動経路で効くため）が、**それを唯一の登録点にしない**。
- **R18（unwanted）**: If both registration points fire for the same session and command, then acquisition shall still succeed（R3 の再入可能性で吸収）。解放も**二重解放で壊れない**こと。
- **R19（ubiquitous）**: lock に書く `ownerPid` は、**重い検証が走っている間ずっと生存しているプロセス**の pid であること。hook は呼び出しごとに終了する短命プロセスなので、その `process.ppid` をそのまま書くと、**pid 生存で stale を判定する ganbari-quest 側から即座に奪われる**（R20）。長寿命 pid を解決できない場合は、**解決できなかったことを lock に明示**し（`ownerPid: null`）、TTL のみで運用すること。**「取れたつもりで取れていない」状態を黙って作らない。**

### 共有 lock の相互運用（QuickScribe 固有 — ADR-0035 D6）

- **R20（ubiquitous）**: `~/.buzz/.locks/heavy.lock` は **ganbari-quest の独立実装と同じファイルを共有**する。QuickScribe が書いた lock を ganbari-quest 側の失効判定が **stale と判定しない**こと、および ganbari-quest が書いた lock を QuickScribe 側が正しく尊重し、**保持者が死んでいれば TTL を待たずに奪える**こと。**両方向を実測すること**（片方向だけの確認では「効かない」ほうが残る）。
- **R21（unwanted）**: If the QuickScribe hook encounters an internal error, then it shall not silently degrade — fail closed（exit 2）は維持する。ただし**オーナー領域への登録はマシン上の全 Buzz セッション（ganbari-quest 系を含む）で発火する**ため、段 2 で **ganbari-quest 形状のコマンドで誤爆・誤 block しないこと**を実測すること。

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

Scenario: cwd がリポジトリ root でなくても発火する (R16,R17)
  Given セッションの cwd が `C:\Users\kokor\.buzz` である
  And オーナー領域の settings.json に絶対パスで hook が登録されている
  When 重い検証を実行しようとする
  Then hook が起動し、lock の取得が試みられる

Scenario: repo 側登録だけでは発火しない (R16)
  Given repo 側 `.claude/settings.json` にだけ相対パスで hook が登録されている
  And セッションの cwd が `C:\Users\kokor\.buzz` である
  When heavy lock が別セッションに保持された状態で `npm test` を実行する
  Then block されない（＝この構成は不十分であることを実測で示す）

Scenario: 二重登録でも通る (R18)
  Given 同じ hook が repo 側とオーナー領域の両方から登録されている
  When 同一セッションが 1 つの重い検証を実行する
  Then 2 回目の取得も成功し（再入可能）、block されない

Scenario: ganbari-quest 側から奪われない (R19,R20)
  Given QuickScribe のセッションが heavy lock を取得した
  When ganbari-quest 側の失効判定にその lock を渡す
  Then stale と判定されない

Scenario: ganbari-quest の残骸を待たされない (R20)
  Given ganbari-quest が書いた heavy lock の ownerPid のプロセスが死んでいる
  And TTL は超過していない
  When QuickScribe のセッションが重い検証を実行しようとする
  Then lock を奪取して実行が続行する（TTL 満了まで待たされない）
```

## 段階実装（ADR-0006 — 削らずに分ける）

| 段 | 内容 | 完了条件 |
|---|---|---|
| **段 1** | 判定 pure function ＋ lock 実体 ＋ 単体テスト（R6〜R12・R19・R20 を固定。**hook はまだ配線しない**） | vitest で該当 Scenario が pass。R20 は **ganbari-quest が実際に書いた lock JSON を fixture** にして両方向を固定する |
| **段 2** | `PreToolUse` / `PostToolUse` hook を配線し、repo 側 `.claude/settings.json` を置く（R1〜R5・R13・R18） | hook に stdin JSON を流す probe で block / 奪取 / fail closed / 解放を実測 |
| **段 3** | **オーナー領域 `~/.buzz/.claude/settings.json` への登録**（R16・R17・R21）。**オーナーの作業を含むため、手順を用意して依頼する** | 登録前後で「別セッション保持中の `cargo build` が block されるか」が **否 → 是**に変わることを実測。ganbari-quest 形状のコマンドで誤爆しないことも実測 |
| **段 4** | `docs/process/agent-session-concurrency.md` §7 に検証手順を記載し、CLAUDE.md から参照を張る | 参照が張られていること |

**検出範囲を狭めることによるリスク回避は採らない**（ADR-0006）。誤爆が出たら対象を削るのではなく判定を精緻化する。

**段 3 を「オーナー作業だから」と後回しにしない。** 段 2 までで止めると、テストが全部 green でリポジトリに一式揃っているのに **hook が一度も発火しない**状態が完成する。これは何も入っていない状態より悪い（入っているつもりになる）。段 2 の PR には「**まだ効いていない**」ことを本文に明記すること。

## 検証（QS-Dev が実測して報告する）

- 単体: `npx vitest run <テストパス>` — **並走を確認してから回す。** 他セッションが重い検証中なら回さずに報告する（この仕様が防ごうとしている状態そのものである）
- hook: stdin JSON を流す probe で exit code を実測。**パイプで exit code を殺さない**（`cmd | tail` は起動失敗でも exit 0 になる）
- **登録が効いているかは「hook が動くこと」で確認する。** ファイルを置いたことや設定 JSON を読めたことは根拠にならない。**別セッションが lock を保持している状態で、重い検証が実際に exit 2 で止まること**を見る（2026-07-27 に QS-Dev が実施した実験がこの形）
- **R20 は片方向だけ確認して終わりにしない。** 「QuickScribe の lock が ganbari-quest から奪われない」と「ganbari-quest の死んだ lock を QuickScribe が待たずに奪える」は別の失敗であり、片方が通っても他方は壊れている

## 範囲外

- PR 単位の排他（`gh pr merge` / `gh pr edit`）— 別 key が要る。実測された事故が無いので本増分に含めない
- 待機キュー / 優先度 — 「待たない」が運用方針なので不要
- 人間の直接実行の排他 — hook の届く範囲外（[運用 SSOT §6](../../process/agent-session-concurrency.md)）
