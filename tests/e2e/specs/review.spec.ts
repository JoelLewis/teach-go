describe("Review", () => {
  // First, play a short game to review
  before(async () => {
    // Start a new game
    const newGameBtn = await $("button=New Game");
    await newGameBtn.click();

    const startBtn = await $("button=Start Game");
    await startBtn.click();

    // Wait for board to appear
    const board = await $('[data-testid="go-board"]');
    await board.waitForDisplayed();

    // Play a few moves
    const moves = [
      [2, 2], [2, 6], [6, 6], [6, 2], [4, 4],
    ];

    for (const [row, col] of moves) {
      const intersection = await $(`[data-testid="intersection"][data-row="${row}"][data-col="${col}"]`);
      await intersection.click();
      // Wait for AI response (if applicable)
      await browser.pause(500);
    }

    // Resign to end the game
    const resignBtn = await $("button=Resign");
    await resignBtn.click();

    // Wait for game over
    const gameOver = await $("*=Game Over");
    await gameOver.waitForDisplayed();
  });

  it("should navigate to review from game over", async () => {
    const reviewBtn = await $("button=Review Game");
    await reviewBtn.click();

    // Review view should show
    const header = await $("h2=Game Review");
    await expect(header).toBeDisplayed();
  });

  it("should show analysis progress or data", async () => {
    // Either the analysis is in progress or complete
    const reviewContent = await $(".flex.h-full");
    await expect(reviewContent).toBeDisplayed();
  });

  it("should display the board in review mode", async () => {
    const board = await $('[data-testid="go-board"]');
    await expect(board).toBeDisplayed();
  });

  it("should support keyboard navigation", async () => {
    // Press right arrow to go forward
    await browser.keys("ArrowRight");
    await browser.pause(300);

    // Press left arrow to go back
    await browser.keys("ArrowLeft");
    await browser.pause(300);

    // Board should still be displayed
    const board = await $('[data-testid="go-board"]');
    await expect(board).toBeDisplayed();
  });

  it("should navigate home from review", async () => {
    const homeBtn = await $("button=Home");
    await homeBtn.click();

    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");
  });
});
