# 原典スナップショット: EARS（Easy Approach to Requirements Syntax）

- 原典論文: Mavin, Wilkinson, Harwood, Novak（Rolls-Royce）. "Easy Approach to Requirements Syntax (EARS)." RE'09（2009）, pp.317–322. **DOI 10.1109/RE.2009.9**。航空エンジン制御(FADEC)の耐空性規制(CS-E 50)分析から考案。
- 正典URL: https://alistairmavin.com/ears/ ／ IEEE https://ieeexplore.ieee.org/document/5328509
- 定義（短い逐語, 公式サイト）: "Easy Approach to Requirements Syntax (EARS) is a mechanism to gently constrain textual requirements."

## 動機: 自然言語要件の8つの問題（原典 §2）
Ambiguity / Vagueness / Complexity / Omission / Duplication / Wordiness / **Inappropriate implementation**（"how" を書いてしまう）/ **Untestability**。
→ EARSは「whatを曖昧さなくテスト可能に書く」ための最小の構文制約。ISO 29148 の "what not how"（[iso-29148.md](iso-29148.md)）と同じ動機。

## 基本テンプレート
> While `<optional pre-condition>`, when `<optional trigger>`, the `<system name>` shall `<system response>`

## 5パターン
| 型 | キーワード | テンプレート |
|---|---|---|
| Ubiquitous（常在） | （なし） | The `<system>` shall `<response>` |
| State-driven（状態駆動） | While | While `<precondition>`, the `<system>` shall `<response>` |
| Event-driven（イベント駆動） | When | When `<trigger>`, the `<system>` shall `<response>` |
| Optional feature（任意機能） | Where | Where `<feature included>`, the `<system>` shall `<response>` |
| Unwanted behaviour（不要動作） | If-Then | If `<trigger>`, then the `<system>` shall `<response>` |

- Complex requirements: 複数キーワードの組み合わせ（While + When 等）。

## QuickScribeでの含意
受入基準を EARS で書けば、AIエージェントに対し「前提条件・トリガー・システム応答」を曖昧さなく渡せる。
TDDのテストリスト/受入テストの語彙と一致させやすい（[driven-development.md](../driven-development.md) 参照）。
