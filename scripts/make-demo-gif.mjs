#!/usr/bin/env node
// デモGIFを生成する（#55 S9.1）。e2e/demo.spec.ts が撮ったフレームを ffmpeg で結合し、
// docs/assets/demo.gif を出力する。ffmpeg が必要（CIは apt で導入、ローカルも要インストール）。
import { execFileSync } from "node:child_process";
import { existsSync, rmSync } from "node:fs";

const DIR = "docs/assets/demo";
const OUT = "docs/assets/demo.gif";
const FRAMES = [0, 1, 2, 3].map((i) => `${DIR}/frame-${i}.png`);

for (const f of FRAMES) {
  if (!existsSync(f)) {
    console.error(`フレームがありません: ${f}。先に 'npm run screenshots' を実行してください。`);
    process.exit(1);
  }
}

// 各フレームを 1.6 秒表示（最後の整形結果は少し長く）。パレット2パスで高品質・軽量化。
// concat demuxer 用のリストを一時生成せず、filter_complex で duration を制御する。
const durations = [1.4, 1.2, 1.8, 2.4];
const inputs = FRAMES.flatMap((f, i) => ["-loop", "1", "-t", String(durations[i]), "-i", f]);
const n = FRAMES.length;
const scale = "scale=1000:-2:flags=lanczos";
const filter =
  FRAMES.map((_, i) => `[${i}:v]${scale},setsar=1[v${i}]`).join(";") +
  ";" +
  FRAMES.map((_, i) => `[v${i}]`).join("") +
  `concat=n=${n}:v=1:a=0[vc];[vc]split[s0][s1];[s0]palettegen=stats_mode=diff[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3[out]`;

execFileSync(
  "ffmpeg",
  ["-y", ...inputs, "-filter_complex", filter, "-map", "[out]", "-loop", "0", OUT],
  { stdio: "inherit" },
);

// フレームは中間生成物なので削除（GIFのみコミット）。
rmSync(DIR, { recursive: true, force: true });
console.log(`\n生成: ${OUT}`);
