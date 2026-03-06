describe("Game Flow", () => {
  it("should show home screen with New Game button", async () => {
    const title = await $("h1");
    await expect(title).toHaveText("GoSensei");

    const newGameBtn = await $("button=New Game");
    await expect(newGameBtn).toBeDisplayed();
  });

  it("should open new game dialog", async () => {
    const newGameBtn = await $("button=New Game");
    await newGameBtn.click();

    // Dialog should appear with board size options
    const dialog = await $("button=Start Game");
    await expect(dialog).toBeDisplayed();
  });

  it("should start a 9x9 game and show the board", async () => {
    // Start game (dialog should already be open from previous test)
    const startBtn = await $("button=Start Game");
    await startBtn.click();

    // Board should appear
    const board = await $('[data-testid="go-board"]');
    await expect(board).toBeDisplayed();

    // Should show "Black to play" indicator
    const turnIndicator = await $("*=to play");
    await expect(turnIndicator).toBeDisplayed();
  });

  it("should place a stone by clicking an intersection", async () => {
    // Click center intersection (4,4 on a 9x9 board)
    const intersection = await $('[data-testid="intersection"][data-row="4"][data-col="4"]');
    await intersection.click();

    // A stone should appear at that position
    const stone = await $('[data-testid="stone"][data-row="4"][data-col="4"]');
    await expect(stone).toBeDisplayed();
    await expect(stone).toHaveAttribute("data-color", "black");
  });

  it("should show last move indicator", async () => {
    const lastMove = await $('[data-testid="last-move"]');
    await expect(lastMove).toBeDisplayed();
  });

  it("should handle pass", async () => {
    const passBtn = await $("button=Pass");
    await passBtn.click();

    // Move number should increment
    const moveInfo = await $("*=Move");
    await expect(moveInfo).toBeDisplayed();
  });

  it("should handle undo", async () => {
    const undoBtn = await $("button=Undo");
    await undoBtn.click();

    // Board should still have the first stone but pass should be undone
    const board = await $('[data-testid="go-board"]');
    await expect(board).toBeDisplayed();
  });

  it("should handle resign and show game over", async () => {
    const resignBtn = await $("button=Resign");
    await resignBtn.click();

    // Game over banner should appear
    const gameOver = await $("*=Game Over");
    await expect(gameOver).toBeDisplayed();
  });

  it("should show Review Game button after game ends", async () => {
    const reviewBtn = await $("button=Review Game");
    await expect(reviewBtn).toBeDisplayed();
  });
});
