describe("Navigation", () => {
  it("should start on home view", async () => {
    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");
  });

  it("should navigate to New Game dialog and back", async () => {
    const newGameBtn = await $("button=New Game");
    await newGameBtn.click();

    // Dialog should be visible
    const startBtn = await $("button=Start Game");
    await expect(startBtn).toBeDisplayed();

    // Close dialog (click backdrop or close button if exists)
    // The NewGameDialog uses a click overlay to close
    await browser.keys("Escape");
    // Wait for dialog to close
    await browser.pause(300);
  });

  it("should navigate to Play view and back to Home", async () => {
    const newGameBtn = await $("button=New Game");
    await newGameBtn.click();

    const startBtn = await $("button=Start Game");
    await startBtn.click();

    // Verify board is displayed
    const board = await $('[data-testid="go-board"]');
    await expect(board).toBeDisplayed();

    // Click Home button in the sidebar
    const homeBtn = await $("button=Home");
    await homeBtn.click();

    // Should be back on home view
    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");
  });

  it("should navigate to Problems view and back", async () => {
    const problemsBtn = await $("button=Practice Problems");
    await problemsBtn.click();

    // Problems list header should be visible
    const header = await $("h1=Practice Problems");
    await expect(header).toBeDisplayed();

    // Go back home
    const homeBtn = await $("button=Home");
    await homeBtn.click();

    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");
  });

  it("should navigate to Dashboard view and back", async () => {
    const dashboardBtn = await $("button=Progress");
    await dashboardBtn.click();

    // Dashboard should show some content
    const homeBtn = await $("button=Home");
    await expect(homeBtn).toBeDisplayed();

    await homeBtn.click();

    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");
  });
});
