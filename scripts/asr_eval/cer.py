#!/usr/bin/env python3
# 文字誤り率(CER)とブートストラップ信頼区間（ADR-0024 / #578）。
# - CER = 文字単位 Levenshtein 距離 / 参照長。
# - コーパスCERは**マイクロ平均**（総編集距離 / 総参照長 = 発話長で重みづけ）を主指標にする。
# - 信頼区間は**発話単位のブートストラップ**（seed 固定で決定的）。CIが重なる差は「差なし」。
import random
from typing import List, Sequence, Tuple


def edit_distance(ref: Sequence, hyp: Sequence) -> int:
    """文字（要素）単位の Levenshtein 距離。"""
    r, h = list(ref), list(hyp)
    if not r:
        return len(h)
    dp = list(range(len(h) + 1))
    for i in range(1, len(r) + 1):
        prev, dp[0] = dp[0], i
        for j in range(1, len(h) + 1):
            cur = dp[j]
            dp[j] = min(dp[j] + 1, dp[j - 1] + 1, prev + (r[i - 1] != h[j - 1]))
            prev = cur
    return dp[len(h)]


def cer(ref: str, hyp: str) -> float:
    """1発話の CER。参照が空なら仮説も空で 0.0、非空なら 1.0。"""
    if not ref:
        return 0.0 if not hyp else 1.0
    return edit_distance(ref, hyp) / len(ref)


def utterance_stats(pairs: Sequence[Tuple[str, str]]) -> List[Tuple[int, int]]:
    """各発話の (編集距離, 参照長) を返す。マイクロ平均・ブートストラップの素。"""
    return [(edit_distance(r, h), len(r)) for r, h in pairs]


def micro_cer(stats: Sequence[Tuple[int, int]]) -> float:
    """マイクロ平均 CER = 総編集距離 / 総参照長。参照長ゼロは 0.0。"""
    total_len = sum(n for _, n in stats)
    if total_len == 0:
        return 0.0
    return sum(d for d, _ in stats) / total_len


def bootstrap_ci(
    stats: Sequence[Tuple[int, int]],
    iters: int = 1000,
    seed: int = 1234,
    alpha: float = 0.05,
) -> Tuple[float, float, float]:
    """発話単位リサンプリングでマイクロ平均CERの (点推定, 下限, 上限) を返す。

    seed 固定で決定的（CIログの再現性のため）。alpha=0.05 で 95%CI。
    """
    n = len(stats)
    point = micro_cer(stats)
    if n == 0:
        return (point, 0.0, 0.0)
    rng = random.Random(seed)
    means = []
    for _ in range(iters):
        resample = [stats[rng.randrange(n)] for _ in range(n)]
        means.append(micro_cer(resample))
    means.sort()
    lo = means[int((alpha / 2) * iters)]
    hi = means[min(iters - 1, int((1 - alpha / 2) * iters))]
    return (point, lo, hi)
