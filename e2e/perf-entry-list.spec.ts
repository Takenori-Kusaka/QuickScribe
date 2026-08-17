import { test, expect, type Page } from "@playwright/test";

// エントリ一覧の描画コスト計測（#666）。Tauri API はモック（e2e/mocks/core.ts）で、
// `?entries=N` により list_entries が N 件を返す。
//
// CI からは意図的に外している（playwright.config.ts の testMatch は screenshots|a11y|demo のみで、
// このファイルは一致しない）。時間依存の数値なので、実行環境が変われば絶対値も変わる。実行方法:
//   npx playwright test --config e2e/playwright.perf.config.ts
//
// 比較の取り方: 「開いて 50 行が出るまで(staged)」と「そこから残りを展開して 1000 行になるまで(full)」を
// 分けて測る。この2つは同一の開封動作を分割したものなので、**staged + full が従来実装（開いた瞬間に
// 全件を DOM 化）の所要時間に相当**し、staged が今の初期描画にあたる。同一ビルド内で測るため
// ビルド差・機種差が入らない。

const N = 1000;

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem("provider", "ollama");
    localStorage.setItem("locale", "ja");
    localStorage.setItem("onboarded", "1");
  });
});

/**
 * ボタン押下から一覧が `expected` 行に達しレイアウトが確定するまでを **ページ内で** 計測する(ms)。
 * Playwright 越しの click/待機はドライバの往復(数百ms)に埋もれて描画コストが見えないため、
 * クリックも待機も page.evaluate の中で完結させる。
 */
async function measureRender(page: Page, buttonPattern: string, expected: number): Promise<number> {
  return await page.evaluate(
    async ({ buttonPattern, expected }) => {
      const re = new RegExp(buttonPattern);
      // ジャーナルを開くボタンはラベルが aria-label 側にあるため、両方を対象にする。
      const btn = [...document.querySelectorAll("button")].find((b) =>
        re.test(`${b.textContent ?? ""} ${b.getAttribute("aria-label") ?? ""}`),
      );
      if (!btn) throw new Error(`button not found: ${buttonPattern}`);
      const rows = () => document.querySelectorAll(".entry-list li").length;
      const t0 = performance.now();
      btn.click();
      // Svelte の反映は非同期。行数が揃うまでマイクロタスクで待つ（rAF の16ms量子化を避ける）。
      for (let i = 0; i < 100_000 && rows() < expected; i++) await Promise.resolve();
      // 生成した DOM のレイアウトを強制的に確定させてから止める（構築だけでなく整形コストも含める）。
      void (document.querySelector(".entry-list") as HTMLElement | null)?.offsetHeight;
      return performance.now() - t0;
    },
    { buttonPattern, expected },
  );
}

test(`${N}件: 段階表示で初期描画の行数と所要時間が下がる`, async ({ page }) => {
  await page.goto(`/?entries=${N}`);
  await expect(page.getByRole("button", { name: "ジャーナル" })).toBeVisible();
  const staged = await measureRender(page, "ジャーナル", 50);
  const rows = await page.locator(".entry-list li").count();

  // 全件展開＝従来実装（全件を一度に DOM 化）相当の状態を同一ビルドで再現する。
  const full = await measureRender(page, "他 \\d+ 件を表示", N);
  await expect(page.locator(".entry-list li")).toHaveCount(N);

  console.log(
    `[#666] entries=${N} staged=${rows}行/${staged.toFixed(1)}ms full=${N}行/${full.toFixed(1)}ms`,
  );

  // 構造的な事実（時間に依らない）: 初期描画の行数が定数で頭打ちになる。
  expect(rows).toBe(50);
  expect(staged).toBeLessThan(full);
});

test(`${N}件: 検索入力の応答（打鍵の所要時間と反映）`, async ({ page }) => {
  await page.goto(`/?entries=${N}`);
  await page.getByRole("button", { name: "ジャーナル" }).click();
  await expect(page.locator(".entry-list li")).toHaveCount(50);

  const search = page.getByLabel(/検索/);
  const t0 = await page.evaluate(() => performance.now());
  await search.pressSequentially("bulk-99-", { delay: 10 });
  const t1 = await page.evaluate(() => performance.now());
  console.log(`[#666] entries=${N} 8打鍵の所要 ${(t1 - t0).toFixed(1)}ms`);

  // デバウンス後に絞り込みが反映される（全件から1件へ）。
  await expect(page.locator(".entry-list li")).toHaveCount(1);
});

test(`${N}件: 段階表示でも全件へ到達できる`, async ({ page }) => {
  await page.goto(`/?entries=${N}`);
  await page.getByRole("button", { name: "ジャーナル" }).click();
  await expect(page.locator(".entry-list li")).toHaveCount(50);
  await page.getByRole("button", { name: /他 \d+ 件を表示/ }).click();
  await expect(page.locator(".entry-list li")).toHaveCount(N);
  await page.getByRole("button", { name: "折りたたむ" }).click();
  await expect(page.locator(".entry-list li")).toHaveCount(50);
});
