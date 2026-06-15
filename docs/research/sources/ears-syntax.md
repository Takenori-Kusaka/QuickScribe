# 原典スナップショット: EARS（Easy Approach to Requirements Syntax）

- 著者/出自: Alistair Mavin ら（Rolls-Royce）。ジェットエンジン制御の耐空性規制分析から考案。初出 RE'09（2009）。
- 正典URL: https://alistairmavin.com/ears/
- 定義（短い逐語）: "Easy Approach to Requirements Syntax (EARS) is a mechanism to gently constrain textual requirements."（テキスト要件を穏やかに制約する仕組み）

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
