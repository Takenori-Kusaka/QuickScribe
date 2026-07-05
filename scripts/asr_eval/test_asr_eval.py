#!/usr/bin/env python3
# scripts/asr_eval の評価コア（正規化・CER・ブートストラップCI）の単体テスト（ADR-0024 / #578）。
# 依存を増やさないため plain assert で書く。実行: python scripts/asr_eval/test_asr_eval.py
from cer import bootstrap_ci, cer, edit_distance, micro_cer, utterance_stats
from normalize_ja import normalize


def test_normalize_strips_timestamp_punct_space():
    # タイムスタンプ・約物・空白（半角/全角）を落として文字だけにする。
    assert normalize("[00:00:01] こんにちは、世界！", use_neologdn=False) == "こんにちは世界"
    assert normalize("東京　大阪　名古屋", use_neologdn=False) == "東京大阪名古屋"


def test_normalize_nfkc_and_lowercase():
    # 全角英数→半角（NFKC）＋ラテン小文字化。
    assert normalize("ＡＢＣ 123", use_neologdn=False) == "abc123"


def test_normalize_removes_frontmatter_and_ruby_brackets():
    assert normalize("---\nschema: 1\n---\n\nテスト", use_neologdn=False) == "テスト"
    # 括弧内注記（ルビ等）は除去。
    assert normalize("漢字（かんじ）を書く", use_neologdn=False) == "漢字を書く"


def test_normalize_ref_hyp_equivalence():
    # 句読点/空白だけ違う参照と仮説は、正規化後に一致する（=CER 0 になるべき）。
    a = normalize("東京、大阪。", use_neologdn=False)
    b = normalize("東京 大阪", use_neologdn=False)
    assert a == b == "東京大阪"


def test_edit_distance_and_cer():
    assert edit_distance("あいう", "あいう") == 0
    assert edit_distance("あいう", "あいえ") == 1  # 1置換
    assert cer("あいう", "あいう") == 0.0
    assert abs(cer("あいう", "あいえ") - (1 / 3)) < 1e-9
    # 参照が空: 仮説も空なら0、非空なら1。
    assert cer("", "") == 0.0
    assert cer("", "x") == 1.0


def test_micro_cer_is_length_weighted():
    # 長い発話の誤りが平均に強く効く（マイクロ平均）。
    # 発話1: ref長10で誤り1 → 局所0.1 / 発話2: ref長2で誤り1 → 局所0.5
    stats = [(1, 10), (1, 2)]
    assert abs(micro_cer(stats) - (2 / 12)) < 1e-9  # 総編集2/総長12


def test_bootstrap_ci_deterministic_and_brackets_point():
    pairs = [("あいうえお", "あいうえお"), ("かきくけこ", "かきくけと"), ("さしすせそ", "さしすせそ")]
    stats = utterance_stats(pairs)
    point, lo, hi = bootstrap_ci(stats, iters=500, seed=1234)
    # 点推定はマイクロ平均に一致。
    assert abs(point - micro_cer(stats)) < 1e-12
    # 区間は点推定を含む。
    assert lo <= point <= hi
    # seed 固定で決定的（同じ入力・seed で同じ結果）。
    again = bootstrap_ci(stats, iters=500, seed=1234)
    assert (point, lo, hi) == again


def test_bootstrap_ci_perfect_is_zero():
    stats = [(0, 5), (0, 3)]
    assert bootstrap_ci(stats, iters=100, seed=1) == (0.0, 0.0, 0.0)


def _run():
    fns = [v for k, v in sorted(globals().items()) if k.startswith("test_")]
    for fn in fns:
        fn()
        print(f"ok  {fn.__name__}")
    print(f"\n{len(fns)} passed")


if __name__ == "__main__":
    _run()
