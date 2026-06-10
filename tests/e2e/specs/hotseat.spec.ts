describe("Hotseat Mode", () => {
  before(async () => {
    // Return to home if a previous spec left us in a game view
    const homeBtn = await $("button=Home");
    if (await homeBtn.isDisplayed().catch(() => false)) {
      await homeBtn.click();
    }

    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");
  });

  it("should start a human-vs-human game immediately", async () => {
    const newGameBtn = await $("button=New Game");
    await newGameBtn.click();

    // Choose Human opponent — color and AI strength options disappear
    const humanBtn = await $("button=Human");
    await humanBtn.click();

    const startBtn = await $("button=Start");
    await startBtn.click();

    // Board should appear without waiting on any engine download
    const board = await $('[data-testid="go-board"]');
    await board.waitForDisplayed();

    // Hotseat shows the prominent turn indicator
    const turnIndicator = await $("*=Black's turn");
    await expect(turnIndicator).toBeDisplayed();
  });

  it("should alternate stone colors between the two local players", async () => {
    const firstIntersection = await $('[data-testid="intersection"][data-row="2"][data-col="2"]');
    await firstIntersection.click();

    const blackStone = await $('[data-testid="stone"][data-row="2"][data-col="2"]');
    await expect(blackStone).toBeDisplayed();
    await expect(blackStone).toHaveAttribute("data-color", "black");

    const secondIntersection = await $('[data-testid="intersection"][data-row="6"][data-col="6"]');
    await secondIntersection.click();

    const whiteStone = await $('[data-testid="stone"][data-row="6"][data-col="6"]');
    await expect(whiteStone).toBeDisplayed();
    await expect(whiteStone).toHaveAttribute("data-color", "white");
  });

  it("should never show the AI thinking indicator", async () => {
    const aiThinking = await $("*=AI is thinking");
    await expect(aiThinking).not.toBeDisplayed();
  });

  it("should undo exactly the last stone", async () => {
    const undoBtn = await $("button=Undo");
    await undoBtn.click();

    // White's stone is removed...
    const whiteStone = await $('[data-testid="stone"][data-row="6"][data-col="6"]');
    await expect(whiteStone).not.toBeDisplayed();

    // ...but Black's stone remains
    const blackStone = await $('[data-testid="stone"][data-row="2"][data-col="2"]');
    await expect(blackStone).toBeDisplayed();

    // It's White's turn again
    const turnIndicator = await $("*=White's turn");
    await expect(turnIndicator).toBeDisplayed();
  });

  it("should resign the side to move and show game over", async () => {
    // In hotseat the resign button names the side to move (White, after the undo)
    const resignBtn = await $("button=Resign (White)");
    await expect(resignBtn).toBeDisplayed();
    await resignBtn.click();

    const gameOver = await $("*=Game Over");
    await gameOver.waitForDisplayed();

    const reviewBtn = await $("button=Review Game");
    await expect(reviewBtn).toBeDisplayed();
  });
});
