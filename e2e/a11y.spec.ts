import { test, expect } from "@playwright/test";

// アクセシビリティの実ブラウザ検証（#395）。Tauri API はモック（e2e/mocks）。
// モーダルが dialog ロールを持ち、Escape で閉じることを確認する。

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem("provider", "ollama");
  });
});

test("ジャーナルモーダルが dialog ロールを持ち Escape で閉じる", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "ジャーナル" }).click();

  const dialog = page.getByRole("dialog", { name: "ジャーナル" });
  await expect(dialog).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
});

test("設定モーダルが dialog ロールを持ち Escape で閉じる", async ({ page }) => {
  await page.goto("/");
  await page.getByTestId("settings-btn").click();

  const dialog = page.getByRole("dialog", { name: "設定" });
  await expect(dialog).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
});
