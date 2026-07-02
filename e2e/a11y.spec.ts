import { test, expect, type Page } from "@playwright/test";

// アクセシビリティの実ブラウザ検証（#395 / WCAG2.1 AA）。Tauri API はモック（e2e/mocks）。
// モーダル挙動に加え、axe-core(vendored)で serious/critical 違反ゼロを自動検証する。

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem("provider", "ollama");
    localStorage.setItem("locale", "ja"); // 日本語UIで決定的に検証。
    localStorage.setItem("onboarded", "1"); // 通常状態で検証(オンボは別途)。
  });
});

// axe-core を vendored ファイルから注入し、WCAG2a/2aa の serious/critical 違反を返す。
// npm 依存を増やさない(package-lock の libc 事故回避)ため e2e/vendor/axe.min.js を使う。
async function axeSeriousViolations(
  page: Page,
): Promise<{ id: string; impact: string; nodes: number }[]> {
  await page.addScriptTag({ path: "e2e/vendor/axe.min.js" });
  return await page.evaluate(async () => {
    // @ts-expect-error axe は注入済みのグローバル。
    const results = await axe.run(document, {
      runOnly: { type: "tag", values: ["wcag2a", "wcag2aa"] },
    });
    return results.violations
      .filter((v: { impact?: string }) => v.impact === "serious" || v.impact === "critical")
      .map((v: { id: string; impact?: string; nodes: unknown[] }) => ({
        id: v.id,
        impact: v.impact ?? "",
        nodes: v.nodes.length,
      }));
  });
}

test("axe: メイン画面に serious/critical 違反がない", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "QuickScribe" })).toBeVisible();
  const v = await axeSeriousViolations(page);
  expect(v, `違反: ${JSON.stringify(v)}`).toEqual([]);
});

test("axe: 設定ダイアログに serious/critical 違反がない", async ({ page }) => {
  await page.goto("/");
  await page.getByTestId("settings-btn").click();
  await expect(page.getByRole("dialog", { name: "設定" })).toBeVisible();
  const v = await axeSeriousViolations(page);
  expect(v, `違反: ${JSON.stringify(v)}`).toEqual([]);
});

test("axe: ジャーナルに serious/critical 違反がない", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "ジャーナル" }).click();
  await expect(page.getByRole("dialog", { name: "ジャーナル" })).toBeVisible();
  const v = await axeSeriousViolations(page);
  expect(v, `違反: ${JSON.stringify(v)}`).toEqual([]);
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
