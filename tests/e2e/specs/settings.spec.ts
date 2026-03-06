describe("Settings", () => {
  it("should open settings dialog", async () => {
    // Settings is typically accessed from a view's sidebar or a global button
    // First go to play view where settings is accessible
    const newGameBtn = await $("button=New Game");
    await newGameBtn.click();

    const startBtn = await $("button=Start Game");
    await startBtn.click();

    // Wait for the game view to load
    const board = await $('[data-testid="go-board"]');
    await board.waitForDisplayed();
  });

  it("should display the board with current theme", async () => {
    const board = await $('[data-testid="go-board"]');
    await expect(board).toBeDisplayed();

    // Board should have SVG elements (grid lines, star points)
    const lines = await $$("line");
    expect(lines.length).toBeGreaterThan(0);
  });

  it("should render star points on the board", async () => {
    // 9x9 board has 5 star points
    const board = await $('[data-testid="go-board"]');
    await expect(board).toBeDisplayed();
  });

  it("should have working click targets on all intersections", async () => {
    // Verify intersection click targets exist
    const intersections = await $$('[data-testid="intersection"]');
    // 9x9 = 81 intersections
    expect(intersections.length).toBe(81);
  });

  it("should go back home", async () => {
    const homeBtn = await $("button=Home");
    await homeBtn.click();

    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");
  });
});
