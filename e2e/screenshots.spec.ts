import { test, expect } from "@playwright/test";

// README/サイト用スクリーンショットを自動生成する。
// Tauri API はモック（e2e/mocks）。すべてダミーデータで描画する。
// 出力: docs/assets/screenshot-main.png / screenshot-vault.png

test.beforeEach(async ({ page }) => {
  // 整形プロバイダをローカル(Ollama=鍵不要)にして refine を通す。
  await page.addInitScript(() => {
    localStorage.setItem("provider", "ollama");
    localStorage.setItem("locale", "ja"); // 日本語UIで決定的に検証（OS言語に依存させない）。
    localStorage.setItem("onboarded", "1"); // 既存ショットは通常フロー（初回オンボーディングは別テストで撮影）。
  });
});

test("メイン画面（文字起こし→整形結果）", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "QuickScribe" })).toBeVisible();

  // 文字起こし完了イベントをモックから発火 → 文字起こしカードを表示。
  await page.evaluate(() => {
    const w = window as unknown as { __mockEmit?: (n: string, p: unknown) => void };
    w.__mockEmit?.(
      "transcribe-done",
      "えーと、最近AIDLCっていう、AIを使った開発のライフサイクルっていう考え方が気になっていて、生成AIを使った開発を1年くらい続けてきたんですけど、既存の開発スタイルにうまく落とし込むにはいろいろ理解を深めないといけないなと思っていて、検証用の専用リポジトリを作りたいなと考えています。",
    );
  });
  await expect(page.getByRole("heading", { name: "文字起こし" })).toBeVisible();

  // 整形を実行（refine_text はモックが整形済みテキストを返す）。
  await page.getByRole("button", { name: "✨ 整形する" }).click();
  await expect(page.getByRole("heading", { name: "整形（思考整理）" })).toBeVisible();

  await page.screenshot({ path: "docs/assets/screenshot-main.png", fullPage: true });
});

test("ジャーナル（過去エントリ一覧・横断発見）", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "ジャーナル" }).click();

  const panel = page.locator(".vault-panel");
  await expect(panel.getByRole("heading", { name: "ジャーナル" })).toBeVisible();
  // 一覧が描画されるまで待つ。
  await expect(panel.locator(".entry-item").first()).toBeVisible();

  await panel.screenshot({ path: "docs/assets/screenshot-vault.png" });
});

test("メイン画面（英語ロケール / OS言語デフォルト検証）", async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem("locale", "en"));
  await page.goto("/");
  // en カタログのキーが反映されること。
  await expect(page.getByRole("button", { name: "Start recording" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Journal" })).toBeVisible();
  await page.screenshot({ path: "docs/assets/screenshot-main-en.png", fullPage: true });
});

test("初回オンボーディング（空状態のコア体験案内）", async ({ page }) => {
  // beforeEach の onboarded を消し、初回起動の状態を再現する。
  await page.addInitScript(() => localStorage.removeItem("onboarded"));
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "QuickScribe へようこそ" })).toBeVisible();
  await page.screenshot({ path: "docs/assets/screenshot-onboarding.png", fullPage: true });
});

test("設定パネル（カテゴリ/アコーディオン）", async ({ page }) => {
  await page.goto("/");
  await page.getByTestId("settings-btn").click();

  const panel = page.getByRole("dialog", { name: "設定" });
  await expect(panel).toBeVisible();
  await panel.screenshot({ path: "docs/assets/screenshot-settings.png" });
});
