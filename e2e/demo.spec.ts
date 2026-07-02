import { test, expect } from "@playwright/test";
import { mkdirSync } from "node:fs";

// READMEデモGIF用のフレームを撮る（#55 S9.1 GIFデモ）。
// コアループ(録音→文字起こし→整形)を一定ビューポート(1280x800)で数フレーム撮影し、
// scripts/make-demo-gif.mjs が ffmpeg でGIFへ結合する。Tauri API は e2e/mocks でモック。

const FRAME_DIR = "docs/assets/demo";

test("デモGIF用フレーム（録音→文字起こし→整形）", async ({ page }) => {
  mkdirSync(FRAME_DIR, { recursive: true });
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.addInitScript(() => {
    localStorage.setItem("provider", "ollama");
    localStorage.setItem("locale", "ja");
    localStorage.setItem("onboarded", "1");
  });
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "QuickScribe" })).toBeVisible();

  // frame 0: アイドル。
  await page.screenshot({ path: `${FRAME_DIR}/frame-0.png` });

  // frame 1: 録音中（ボタンを押した状態）。
  await page.getByTestId("record-btn").click();
  await expect(page.getByRole("button", { name: /停止/ })).toBeVisible();
  await page.screenshot({ path: `${FRAME_DIR}/frame-1.png` });
  // 録音を止めて次へ（文字起こしはイベントで供給する）。
  await page.getByTestId("record-btn").click();

  // frame 2: 文字起こし表示。
  await page.evaluate(() => {
    const w = window as unknown as { __mockEmit?: (n: string, p: unknown) => void };
    w.__mockEmit?.(
      "transcribe-done",
      "最近考えていることを声に出して記録して、あとで見返しながら思考を整理していきたい。",
    );
  });
  await expect(page.getByRole("heading", { name: "文字起こし" })).toBeVisible();
  await page.screenshot({ path: `${FRAME_DIR}/frame-2.png` });

  // frame 3: 整形結果。
  await page.getByRole("button", { name: "✨ 整形する" }).click();
  await expect(page.getByRole("heading", { name: "整形（思考整理）" })).toBeVisible();
  await page.screenshot({ path: `${FRAME_DIR}/frame-3.png` });
});
