// 実起動E2E（音声不要のUI挙動）: ウィンドウが立ち上がり、録音ボタンが表示され、
// クリックで「停止」へトグルし、再クリックで「録音開始」に戻ることを検証する。
// これにより「ビルドできる」でなく「起動して動く」をCIで保証する（前回のE2E不在の是正）。

describe("QuickScribe アプリ起動", () => {
  it("見出しと録音ボタンが表示される", async () => {
    // 実起動webviewの描画完了を明示的に待ってから検証する（即時アサートの起動レースを排除）。
    const heading = await $("h1");
    await heading.waitForDisplayed({ timeout: 20000 });
    await expect(heading).toHaveText("QuickScribe");

    const btn = await $('[data-testid="record-btn"]');
    await btn.waitForDisplayed({ timeout: 20000 });
    await expect(btn).toHaveText(expect.stringContaining("録音開始"));
  });

  it("録音ボタンが開始↔停止でトグルする", async () => {
    // 録音バックエンドは E2E(QUICKSCRIBE_E2E=1)時 no-op のため、トグルは状態のみで決定的。
    // トグルは click→IPC(start/stop_recording)→状態反映まで非同期。expect の既定待ち(短い)では
    // まれに取りこぼすため、waitforTimeout(20s)の waitUntil で状態変化を明示的に待つ(#412 根治)。
    const btn = await $('[data-testid="record-btn"]');
    await btn.waitForClickable({ timeout: 20000 });

    await btn.click();
    await btn.waitUntil(async () => (await btn.getText()).includes("停止"), {
      timeout: 20000,
      timeoutMsg: "録音開始のクリック後に「停止」表記へ遷移しませんでした",
    });

    await btn.waitForClickable({ timeout: 20000 });
    await btn.click();
    await btn.waitUntil(async () => (await btn.getText()).includes("録音開始"), {
      timeout: 20000,
      timeoutMsg: "停止のクリック後に「録音開始」表記へ戻りませんでした",
    });
  });
});
