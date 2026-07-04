#!/usr/bin/env python3
# 日本語 CER(文字誤り率) 計算ユーティリティ（#26 日本語精度ベンチ / #403）。
# 正規化: フロントマター/タイムスタンプ除去 → NFKC → 空白・記号除去 → 文字単位。
# ルビ(furigana)が原文に混じる作品は絶対CERが悲観側に出るため、回帰/相対比較の指標として使う。
#   usage: python cer_ja.py REFERENCE_FILE HYPOTHESIS_FILE
import re
import sys
import unicodedata


def normalize(s: str) -> str:
    # YAML フロントマター(先頭の --- ... ---)を除去。
    s = re.sub(r"^\s*---.*?---\s*", "", s, count=1, flags=re.DOTALL)
    # 行頭タイムスタンプ [HH:MM:SS] を除去。
    s = re.sub(r"\[\d{2}:\d{2}:\d{2}\]", "", s)
    # 全角/半角を統一(NFKC)。
    s = unicodedata.normalize("NFKC", s)
    # 記号・約物・空白をすべて除去し、文字の並びだけで比較する。
    s = re.sub(r"[\s　]+", "", s)
    s = re.sub(r"[、。「」『』・､｡（）()!！?？…—―─\-\.,]", "", s)
    return s


def cer(ref: str, hyp: str) -> float:
    # 文字レベル Levenshtein 距離 / 参照長。
    r, h = list(ref), list(hyp)
    if not r:
        return 0.0 if not h else 1.0
    dp = list(range(len(h) + 1))
    for i in range(1, len(r) + 1):
        prev, dp[0] = dp[0], i
        for j in range(1, len(h) + 1):
            cur = dp[j]
            dp[j] = min(dp[j] + 1, dp[j - 1] + 1, prev + (r[i - 1] != h[j - 1]))
            prev = cur
    return dp[len(h)] / len(r)


def main() -> int:
    ref = normalize(open(sys.argv[1], encoding="utf-8").read())
    hyp = normalize(open(sys.argv[2], encoding="utf-8").read())
    rate = cer(ref, hyp)
    # 機械可読(1行)＋パーセント。呼び出し側でパースしやすく。
    print(f"{rate:.4f}\t{rate * 100:.1f}%\tref_len={len(ref)}\thyp_len={len(hyp)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
