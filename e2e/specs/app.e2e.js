// 実起動E2E（音声不要のUI挙動）: ウィンドウが立ち上がり、録音ボタンが表示され、
// クリックで「停止」へトグルし、再クリックで「録音開始」に戻ることを検証する。
// これにより「ビルドできる」でなく「起動して動く」をCIで保証する（前回のE2E不在の是正）。

describe("QuickScribe アプリ起動", () => {
  it("見出しと録音ボタンが表示される", async () => {
    const heading = await $("h1");
    await expect(heading).toHaveText("QuickScribe");

    const btn = await $("button.record");
    await expect(btn).toBeDisplayed();
    await expect(btn).toHaveText(expect.stringContaining("録音開始"));
  });

  it("録音ボタンが開始↔停止でトグルする", async () => {
    const btn = await $("button.record");

    await btn.click();
    await expect(btn).toHaveText(expect.stringContaining("停止"));

    await btn.click();
    await expect(btn).toHaveText(expect.stringContaining("録音開始"));
  });
});
