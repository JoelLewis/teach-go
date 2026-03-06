describe("Problem Training", () => {
  it("should navigate to problems list", async () => {
    // Start from home
    const title = await $("h1");
    if ((await title.getText()) !== "GoSensei") {
      const homeBtn = await $("button=Home");
      await homeBtn.click();
    }

    const problemsBtn = await $("button=Practice Problems");
    await problemsBtn.click();

    const header = await $("h1=Practice Problems");
    await expect(header).toBeDisplayed();
  });

  it("should show problem list with categories", async () => {
    // Category filter buttons should be visible
    const allBtn = await $("button=All");
    await expect(allBtn).toBeDisplayed();
  });

  it("should start a problem and show setup stones", async () => {
    // Click "Recommended Problem" to start one
    const recommendedBtn = await $("button=Recommended Problem");
    await recommendedBtn.click();

    // Board should appear with setup stones
    const board = await $('[data-testid="go-board"]');
    await board.waitForDisplayed({ timeout: 10000 });

    // There should be at least one stone (setup position)
    const stones = await $$('[data-testid="stone"]');
    expect(stones.length).toBeGreaterThan(0);
  });

  it("should show problem prompt", async () => {
    // The problem prompt text should be displayed
    const prompt = await $(".text-lg.font-medium");
    await expect(prompt).toBeDisplayed();
  });

  it("should show hint buttons", async () => {
    const areaHint = await $("button=Area");
    await expect(areaHint).toBeDisplayed();

    const candidatesHint = await $("button=Candidates");
    await expect(candidatesHint).toBeDisplayed();

    const answerHint = await $("button=Answer");
    await expect(answerHint).toBeDisplayed();
  });

  it("should show highlight when hint is used", async () => {
    const areaHint = await $("button=Area");
    await areaHint.click();

    // Wait for highlight to appear on board
    await browser.pause(500);

    // A highlight element should be rendered in the SVG
    const highlight = await $(".highlight");
    await expect(highlight).toBeDisplayed();
  });

  it("should allow skipping a problem", async () => {
    const skipBtn = await $("button=Skip");
    await skipBtn.click();

    // Should return to list after a brief pause
    await browser.pause(1500);

    const header = await $("h1=Practice Problems");
    await expect(header).toBeDisplayed();
  });

  it("should navigate back home", async () => {
    const homeBtn = await $("button=Home");
    await homeBtn.click();

    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");
  });
});
