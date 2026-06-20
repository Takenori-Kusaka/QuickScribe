#!/usr/bin/env python3
"""QuickScribe ロゴ候補を Gemini 画像生成で作り、透過処理＋小サイズ目視まで行う(デザイン時ツール)。

ADR-0007 / docs/research/logo-redesign.md の設計原則に基づくプロンプトで候補を生成し、
scripts/process-brand-icon.py の決定論的処理(彩度マスクで背景除去)を各候補に適用、
16/24/32/48px の明暗プレビューを出力する。「場当たり」を避け、生成→処理→検証を1スクリプトに集約。

鍵はコードに埋めず環境変数 GEMINI_API_KEY から読む(コミット安全)。
    GEMINI_API_KEY=xxx python scripts/generate-logo-candidates.py
出力: tmp/logo-gen/cand_<id>.png(原画) / cand_<id>_clean.png(透過) / cand_<id>_strip.png(小サイズ目視)
"""
from __future__ import annotations

import base64
import importlib.util
import json
import os
import sys
import urllib.request
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "tmp" / "logo-gen"
MODEL = os.environ.get("GEMINI_IMAGE_MODEL", "gemini-2.5-flash-image")

# 設計方針(2026-06-20, docs/research/logo-redesign.md §6 に基づく再設計):
# 却下の真因 = 明るい飽和ベタ地＋太字グリフ＋浅い単語列挙プロンプト＋craft欠如。
# 新方針: 暗 deep-indigo の squircle 地 ＋ 淡く発光する monoline マーク(Apple Journal/Obsidian の
# ジャーナル文法)。Google公式どおり「叙述段落」で書く。情緒=静謐・内省・信頼できる道具性。
# パイプライン互換: 白背景は除去・暗indigoタイル(min<170)を抽出・内部の淡グリフは穴埋めで保持。
# よって外側グロウ/ハロー/影は厳禁(品質ゲートが弾く)。発光は「タイル内部のマークのみ」。
NEG = ("The white background must stay pure solid white with absolutely no tint, no outer glow, no halo, "
       "no bloom, no drop shadow, no vignette anywhere around the tile. No 3D, no bevel, no emboss, no "
       "photographic texture, no realistic material, no reflection, no gloss, no specular highlight. "
       "No microphone, no realistic mic, no headphones, no equalizer-bars cliche, no speech bubble. "
       "No text, no letters, no words, no numbers, no watermark, no noise. Keep the indigo tile fill "
       "perfectly flat and uniform with no color gradient; only the mark itself may be softly luminous. "
       "The mark is a thin elegant monoline, never a thick bold filled glyph.")
BASE = ("A premium, minimalist desktop app icon shown on a pure flat solid white background (#FFFFFF) with "
        "nothing else around it. Centered on the canvas is a single rounded-square tile shaped as a smooth "
        "Apple-style squircle (continuous, optically rounded curvature — not a plain rounded rectangle), "
        "with generous empty margin around it. The tile is filled with ONE calm, deep, slightly desaturated "
        "indigo color (a quiet dark blue-violet around hue 262, like #2c2a5e), completely flat and even with "
        "no gradient and no shading. Inside the tile, the mark is rendered as a single soft, luminous "
        "pale-lavender, almost off-white monoline — one continuous thin stroke with gently rounded ends that "
        "looks as if it is quietly lit from within, calm and refined, in the spirit of the Apple Journal and "
        "Obsidian app icons. The overall mood is quiet, intelligent, trustworthy, private and premium: a "
        "thoughtful tool for organizing one's own thinking, never loud, never techy, never playful. ")

# double_arc 洗練バッチ(2026-06-20)。生成版が「ローディングのスピナー(割れた輪)」に読める弱点を
# 解消。共通: 暗indigo squircle ＋ 淡発光 monoline。鍵は「閉じた円弧にしない・片側のみに開く非対称・
# 数を絞る・丸い端」でスピナー/wifi 誤読を断つ。声が一点から静かに広がる=発話のリップル。
PROMPTS = {
    "ripple_open": BASE + (
        "The mark is one small solid luminous dot with TWO short concentric arcs that open and radiate only to "
        "the right side of the dot (a calm voice gently emanating from a single point). Strongly asymmetric — "
        "the arcs are clearly a quarter-to-third of a circle, never a closed ring, with soft rounded ends. "
        "It must never read as a wifi symbol or a loading spinner. Minimal, serene, generous negative space. "
    ) + NEG,
    "quote_arcs": BASE + (
        "The mark is two short, calm luminous arcs sitting side by side like an abstract pair of quotation "
        "marks turned into gentle curves — a captured spoken voice. Each arc is a simple open curve with "
        "rounded ends, clearly not a circle. Minimal, centered, lots of negative space. "
    ) + NEG,
    "nested_crescents": BASE + (
        "The mark is two nested luminous crescent arcs of different size, both opening to the left, like a "
        "quiet echo of a single voice. Clearly open crescents (not a full circle, not a spinner), soft rounded "
        "ends, calm and premium. Centered, generous negative space. "
    ) + NEG,
    "rising_arcs_dot": BASE + (
        "The mark is one small solid luminous dot near the bottom with two gentle concentric arcs arcing OVER "
        "the top of it (a calm voice rising). The arcs span only the upper half — open at the bottom, clearly "
        "not a closed ring or spinner. Soft rounded ends, minimal, serene, centered. "
    ) + NEG,
    "double_arc_clean": BASE + (
        "The mark is two clean concentric luminous arcs on the right side only, each spanning about a third of "
        "a circle with clearly cut, softly rounded open ends, suggesting a calm voice emanating. Deliberately "
        "asymmetric and obviously incomplete so it never looks like a loading spinner or wifi. Minimal, "
        "elegant, centered, generous negative space. "
    ) + NEG,
}

# 参考: 以前の3方向9案。必要なら ARCHIVED を PROMPTS にして再生成できる(履歴保持)。
ARCHIVED = {
    # --- 方向1: もつれ→一本の線 (コア価値「整形の知性」直接・競合空白) ---
    "thread_settle": BASE + (
        "The mark is ONE single continuous luminous monoline that begins on the left as a single small, gentle, "
        "loosely curled loop — one soft tangle of thought — and then flows smoothly to the right where it "
        "resolves and settles into one calm, perfectly straight horizontal line. Just one unbroken stroke, "
        "lots of quiet negative space around it, very minimal and elegant. "
    ) + NEG,
    "thread_settle_v": BASE + (
        "The mark is ONE single continuous luminous monoline descending from a small soft knot near the top "
        "that gently untangles as it moves down and resolves into one clean, calm straight line at the bottom — "
        "a single thought being organized. One unbroken stroke, generous negative space, minimal. "
    ) + NEG,
    "thread_min": BASE + (
        "The mark is an extremely minimal single continuous luminous line: one soft open loop on the left that "
        "smoothly untangles into one straight calm line on the right, almost like a single relaxed gesture. "
        "Maximum simplicity, one stroke only, lots of empty space. "
    ) + NEG,
    # --- 方向2: 一筆のインク/筆致 (捕えた声→整った一文) ---
    "ink_stroke": BASE + (
        "The mark is ONE single elegant luminous brush-and-ink stroke: it sweeps gently from a slightly fuller "
        "beginning and tapers as it settles into one calm horizontal written line, with soft tapered ends like "
        "a fine pen. One graceful stroke only, centered, lots of negative space. "
    ) + NEG,
    "nib_line": BASE + (
        "The mark is an abstract, minimal fountain-pen feeling suggested with only a single luminous monoline: "
        "a slim pointed tip at the top from which one calm, straight written line is drawn downward. Very "
        "abstract — no literal pen body, just the implied gesture of writing. One stroke, minimal. "
    ) + NEG,
    "ink_drop_line": BASE + (
        "The mark is ONE single continuous luminous monoline that begins as a small soft rounded droplet (a "
        "captured spoken moment) on one side and draws out smoothly into one calm straight line — a voice "
        "becoming written thought. One unbroken stroke, centered, minimal. "
    ) + NEG,
    # --- 方向3: 静かな単一の弧 (声の一吐き・小サイズ最強・上品) ---
    "calm_arc": BASE + (
        "The mark is ONE single luminous smooth crescent arc with softly rounded ends, centered with generous "
        "space, conveying a single calm utterance. It is clearly a single graceful arc — not a full circle, "
        "not a sound wave, not a smile. Minimal and serene. "
    ) + NEG,
    "arc_dot": BASE + (
        "The mark is ONE single luminous smooth arc with a single small luminous dot resting at its focus — a "
        "spoken point captured and held. Calm, minimal, centered, lots of negative space. One arc and one dot "
        "only. "
    ) + NEG,
    "double_arc": BASE + (
        "The mark is two short concentric luminous arcs of slightly different length on one side, suggesting a "
        "calm voice gently emanating, intentionally asymmetric so it never reads as a wifi symbol. Minimal, "
        "serene, centered, generous negative space. "
    ) + NEG,
}


def load_processor():
    spec = importlib.util.spec_from_file_location("pbi", ROOT / "scripts" / "process-brand-icon.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def gen(key: str, prompt: str) -> bytes | None:
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{MODEL}:generateContent?key={key}"
    body = json.dumps({"contents": [{"parts": [{"text": prompt}]}]}).encode()
    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=120) as r:
            d = json.loads(r.read())
    except Exception as e:
        print("  request error:", str(e)[:160])
        return None
    for cand in d.get("candidates", []):
        for part in cand.get("content", {}).get("parts", []):
            inl = part.get("inlineData") or part.get("inline_data")
            if inl and inl.get("data"):
                return base64.b64decode(inl["data"])
    print("  no image in response:", json.dumps(d)[:200])
    return None


def strip(clean: Image.Image, path: Path):
    sizes = [16, 24, 32, 48]
    pad, gap = 8, 8
    w = pad * 2 + sum(sizes) + gap * (len(sizes) - 1)
    h = pad * 2 + max(sizes)
    for bg in [("dark", (32, 32, 32, 255)), ("light", (240, 240, 240, 255))]:
        canvas = Image.new("RGBA", (w, h), bg[1])
        x = pad
        for s in sizes:
            ic = clean.resize((s, s), Image.LANCZOS)
            canvas.alpha_composite(ic, (x, pad + (max(sizes) - s)))
            x += s + gap
        canvas.convert("RGB").save(str(path).replace(".png", f"_{bg[0]}.png"))


def main() -> int:
    key = os.environ.get("GEMINI_API_KEY")
    if not key:
        print("error: set GEMINI_API_KEY"); return 1
    OUT.mkdir(parents=True, exist_ok=True)
    pbi = load_processor()
    for name, prompt in PROMPTS.items():
        print(f"[{name}] generating with {MODEL} ...")
        img = gen(key, prompt)
        if not img:
            print(f"  skip {name}"); continue
        raw_path = OUT / f"cand_{name}.png"
        raw_path.write_bytes(img)
        try:
            src = Image.open(raw_path)
            clean = pbi.process(src)
            pbi.assert_quality(clean)
            clean_path = OUT / f"cand_{name}_clean.png"
            clean.save(clean_path)
            strip(clean, OUT / f"cand_{name}_strip.png")
            frac = (Image.open(clean_path).split()[3])
            print(f"  ok -> {clean_path.name} (+ strips)")
        except SystemExit as e:
            print(f"  process/quality failed for {name}: {e}")
        except Exception as e:
            print(f"  process error for {name}: {str(e)[:160]}")
    print("done. previews in tmp/logo-gen/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
