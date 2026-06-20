#!/usr/bin/env python3
"""nested_crescents(ユーザー選定アンカー)を「保ったまま磨く」refinementツール(デザイン時)。

新しい概念は作らない。気に入った1枚(tmp/logo-gen/cand_nested_crescents.png)を起点に、
2つの制御された方法で磨いた候補を生成し、明暗・小サイズの比較シートを出す:

  A) 参照画像conditioning: 上記画像を Gemini に渡し「構図(暗indigo squircle＋二重の発光クレッセント)
     を保持したまま線/縁/squircle/グロウだけ微調整」させる(Google公式の正攻法・ドリフト最小)。
  B) 決定論レンダリング: 二重の弧は幾何学的なので、squircle地＋同心クレッセント＋発光を
     numpy/PIL で完全制御してピクセル完璧に描く(再現可能・ドリフト皆無)。

鍵は環境変数 GEMINI_API_KEY から読む(コミット安全)。A を飛ばす場合は鍵未設定で B のみ実行。
出力: tmp/logo-gen/refine/ 配下 + コンタクトシート tmp/logo-sheet.png
"""
from __future__ import annotations

import base64
import importlib.util
import json
import os
import sys
import urllib.request
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
GEN = ROOT / "tmp" / "logo-gen"
OUT = GEN / "refine"
MODEL = os.environ.get("GEMINI_IMAGE_MODEL", "gemini-2.5-flash-image")

INDIGO = (44, 42, 94)       # #2c2a5e 暗deep-indigo タイル
LUM = (224, 226, 255)       # #e0e2ff 淡発光のクレッセント

# --- A) 参照画像conditioning ------------------------------------------------
REF = GEN / "cand_nested_crescents.png"
REFINE_PROMPTS = {
    "even": (
        "Here is an app icon. Recreate it while preserving the EXACT composition: the dark deep-indigo "
        "Apple-style squircle tile and the two nested luminous pale crescent arcs opening to the right. "
        "Refine only the craft: make both crescent strokes one clean perfectly even uniform width with "
        "smooth rounded ends, keep the tile a true smooth squircle, and make the inner glow soft and "
        "tasteful, not blown out. Flat solid white background, no outer glow around the tile, no text."
    ),
    "thin": (
        "Here is an app icon. Keep the EXACT composition (dark indigo squircle tile + two nested luminous "
        "crescent arcs opening right), but refine it to be more elegant and minimal: thinner, even, "
        "refined crescent strokes with rounded ends, a true squircle tile, gentle tasteful inner glow. "
        "Flat solid white background, no outer glow around the tile, no text."
    ),
    "balanced": (
        "Here is an app icon. Preserve the EXACT composition (dark indigo squircle + two nested luminous "
        "crescent arcs opening right). Refine the craft so the inner arc reads as a clear, balanced echo "
        "of the outer one (even spacing between them), uniform stroke width, crisp rounded ends, true "
        "squircle tile, soft premium glow. Flat solid white background, no outer glow, no text."
    ),
    "calm": (
        "Here is an app icon. Keep the EXACT composition (dark indigo squircle + two nested luminous "
        "crescent arcs opening right). Make it calmer and more premium: even refined strokes, rounded "
        "ends, slightly reduced glow so it looks quiet and introspective, true squircle tile. Flat solid "
        "white background, no outer glow around the tile, no text."
    ),
}


def load_processor():
    spec = importlib.util.spec_from_file_location("pbi", ROOT / "scripts" / "process-brand-icon.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def gen_with_ref(key: str, prompt: str, ref_bytes: bytes) -> bytes | None:
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{MODEL}:generateContent?key={key}"
    parts = [
        {"inline_data": {"mime_type": "image/png", "data": base64.b64encode(ref_bytes).decode()}},
        {"text": prompt},
    ]
    body = json.dumps({"contents": [{"parts": parts}]}).encode()
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


# --- B) 決定論レンダリング ---------------------------------------------------
def squircle_alpha(size: int, margin_frac: float, n: float = 4.0) -> np.ndarray:
    """超楕円(squircle, n=4)の内側=255 の alpha 配列(supersample前提のハード閾値)。"""
    yy, xx = np.mgrid[0:size, 0:size]
    c = (size - 1) / 2.0
    a = (size / 2.0) * (1.0 - margin_frac)
    d = (np.abs((xx - c) / a)) ** n + (np.abs((yy - c) / a)) ** n
    return np.where(d <= 1.0, 255, 0).astype(np.uint8)


def crescent_layer(size: int, r: float, width: float, start: float, end: float) -> Image.Image:
    """同心クレッセント(丸端)の alpha レイヤを描く。角度は PIL 準拠(3時方向0°・時計回り)。"""
    L = Image.new("L", (size, size), 0)
    d = ImageDraw.Draw(L)
    c = (size - 1) / 2.0
    bbox = [c - r, c - r, c + r, c + r]
    d.arc(bbox, start, end, fill=255, width=int(width))
    # 丸端: 始点・終点に半径 width/2 の円を足す。
    for ang in (start, end):
        rad = np.radians(ang)
        px = c + r * np.cos(rad)
        py = c + r * np.sin(rad)
        rr = width / 2.0
        d.ellipse([px - rr, py - rr, px + rr, py + rr], fill=255)
    return L


def render_deterministic(name: str, *, margin: float, r_out: float, r_in: float,
                         width: float, gap_deg: float, glow: float) -> Image.Image:
    """squircle地＋二重クレッセント＋発光を決定論的に合成。S=2048で描き1024へ縮小(AA)。"""
    S = 2048
    half_gap = gap_deg / 2.0
    start, end = half_gap, 360.0 - half_gap  # 右(East,0°)を開口にした"C"
    out_a = crescent_layer(S, r_out * S, width * S, start, end)
    in_a = crescent_layer(S, r_in * S, width * S, start + 8, end - 8)
    arcs = Image.fromarray(np.maximum(np.asarray(out_a), np.asarray(in_a)), "L")

    tile_a = Image.fromarray(squircle_alpha(S, margin), "L")
    tile_np = np.asarray(tile_a)

    canvas = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    # 1) タイル
    tile_rgba = np.zeros((S, S, 4), np.uint8)
    tile_rgba[..., 0], tile_rgba[..., 1], tile_rgba[..., 2] = INDIGO
    tile_rgba[..., 3] = tile_np
    canvas = Image.fromarray(tile_rgba, "RGBA")
    # 2) グロウ(弧をぼかし、タイル内にクリップ)
    if glow > 0:
        g = arcs.filter(ImageFilter.GaussianBlur(glow * S))
        g_np = (np.asarray(g).astype(np.float32) * 0.8).clip(0, 255).astype(np.uint8)
        g_np = np.minimum(g_np, tile_np)  # タイル外に漏らさない
        glow_rgba = np.zeros((S, S, 4), np.uint8)
        glow_rgba[..., 0], glow_rgba[..., 1], glow_rgba[..., 2] = LUM
        glow_rgba[..., 3] = g_np
        canvas.alpha_composite(Image.fromarray(glow_rgba, "RGBA"))
    # 3) クリスプな弧(タイル内にクリップ)
    a_np = np.minimum(np.asarray(arcs), tile_np)
    arc_rgba = np.zeros((S, S, 4), np.uint8)
    arc_rgba[..., 0], arc_rgba[..., 1], arc_rgba[..., 2] = LUM
    arc_rgba[..., 3] = a_np
    canvas.alpha_composite(Image.fromarray(arc_rgba, "RGBA"))

    return canvas.resize((1024, 1024), Image.LANCZOS)


DET_VARIANTS = {
    # margin, r_out, r_in, width, gap_deg, glow (すべてタイル一辺=1.0 比)
    "det_even":     dict(margin=0.10, r_out=0.30, r_in=0.20, width=0.045, gap_deg=90, glow=0.012),
    "det_thin":     dict(margin=0.10, r_out=0.31, r_in=0.205, width=0.032, gap_deg=95, glow=0.010),
    "det_bold":     dict(margin=0.10, r_out=0.29, r_in=0.185, width=0.058, gap_deg=85, glow=0.016),
    "det_tight":    dict(margin=0.10, r_out=0.30, r_in=0.225, width=0.040, gap_deg=90, glow=0.011),
}


def strip_label(name: str) -> str:
    return name


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    produced: list[tuple[str, Path]] = []

    # アンカー(比較用先頭)
    anchor = GEN / "cand_nested_crescents_clean.png"
    if anchor.exists():
        produced.append(("anchor", anchor))

    # A) 参照画像conditioning
    key = os.environ.get("GEMINI_API_KEY")
    if key and REF.exists():
        pbi = load_processor()
        ref_bytes = REF.read_bytes()
        for name, prompt in REFINE_PROMPTS.items():
            print(f"[A:{name}] ref-conditioning with {MODEL} ...")
            img = gen_with_ref(key, prompt, ref_bytes)
            if not img:
                print(f"  skip {name}"); continue
            raw = OUT / f"refA_{name}.png"
            raw.write_bytes(img)
            try:
                clean = pbi.process(Image.open(raw))
                pbi.assert_quality(clean)
                cp = OUT / f"refA_{name}_clean.png"
                clean.save(cp)
                produced.append((f"A:{name}", cp))
                print(f"  ok -> {cp.name}")
            except SystemExit as e:
                print(f"  process/quality failed: {e}")
            except Exception as e:
                print(f"  process error: {str(e)[:160]}")
    else:
        print("A) skip: GEMINI_API_KEY 未設定 または 参照画像なし")

    # B) 決定論レンダリング
    for name, p in DET_VARIANTS.items():
        print(f"[B:{name}] deterministic render ...")
        img = render_deterministic(name, **p)
        cp = OUT / f"{name}_clean.png"
        img.save(cp)
        produced.append((f"B:{name}", cp))
        print(f"  ok -> {cp.name}")

    # コンタクトシート
    build_sheet(produced)
    print("done. sheet -> tmp/logo-sheet.png")
    return 0


def build_sheet(items: list[tuple[str, Path]]):
    row_h, label_w, big = 110, 150, 88
    sizes = [16, 24, 32, 48]
    W = label_w + 8 + big + 16 + (8 + sum(sizes) + 8 * 3 + 8) + 16
    H = row_h * len(items) + 8
    sheet = Image.new("RGBA", (W, H), (28, 28, 30, 255))
    d = ImageDraw.Draw(sheet)
    for i, (label, path) in enumerate(items):
        y = i * row_h + 8
        d.text((8, y + row_h // 2 - 4), label, fill=(230, 230, 230, 255))
        c = Image.open(path).convert("RGBA")
        sheet.alpha_composite(c.resize((big, big), Image.LANCZOS), (label_w, y + (row_h - 8 - big) // 2))
        x = label_w + big + 16
        for s in sizes:
            sheet.alpha_composite(c.resize((s, s), Image.LANCZOS),
                                  (x, y + (row_h - 8 - 48) // 2 + (48 - s)))
            x += s + 8
    sheet.convert("RGB").save(ROOT / "tmp" / "logo-sheet.png")


if __name__ == "__main__":
    sys.exit(main())
