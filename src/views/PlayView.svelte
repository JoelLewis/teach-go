<script lang="ts">
  import BoardCanvas from "../lib/board/BoardCanvas.svelte";
  import GameControls from "../components/GameControls.svelte";
  import MoveHistory from "../components/MoveHistory.svelte";
  import CoachingPanel from "../components/CoachingPanel.svelte";
  import ScoreBar from "../components/ScoreBar.svelte";
  import { gameStore } from "../lib/stores/game.svelte";
  import { coachingStore } from "../lib/stores/coaching.svelte";
  import * as api from "../lib/api/commands";
  import type { StoneColor } from "../lib/api/types";

  type Props = {
    onGoHome: () => void;
  };

  let { onGoHome }: Props = $props();

  let boardSize = $state(9);

  // Initialize game on mount
  $effect(() => {
    startNewGame();
  });

  async function startNewGame() {
    try {
      const state = await api.newGame(boardSize);
      gameStore.set(state);
      coachingStore.clear();
    } catch (e) {
      gameStore.setError(String(e));
    }
  }

  async function handleIntersectionClick(row: number, col: number) {
    if (!gameStore.state || gameStore.state.phase === "Finished") return;

    try {
      const state = await api.playMove(row, col);
      gameStore.set(state);
    } catch (e) {
      // Ignore illegal move errors silently — the board just won't change
      console.warn("Move rejected:", e);
    }
  }

  async function handlePass() {
    try {
      const state = await api.passTurn();
      gameStore.set(state);
    } catch (e) {
      gameStore.setError(String(e));
    }
  }

  async function handleResign() {
    try {
      const [state] = await api.resign();
      gameStore.set(state);
    } catch (e) {
      gameStore.setError(String(e));
    }
  }

  async function handleUndo() {
    try {
      const state = await api.undoMove();
      gameStore.set(state);
    } catch (e) {
      gameStore.setError(String(e));
    }
  }
</script>

<div class="flex h-full">
  <!-- Board area -->
  <div class="flex flex-1 items-center justify-center p-4">
    {#if gameStore.state}
      <BoardCanvas
        {boardSize}
        stones={gameStore.state.stones}
        currentColor={gameStore.state.current_color as StoneColor}
        lastMove={gameStore.state.last_move}
        onIntersectionClick={handleIntersectionClick}
      />
    {/if}
  </div>

  <!-- Right panel -->
  <div class="flex w-80 flex-col gap-4 border-l border-stone-700 p-4">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold text-stone-200">Game</h2>
      <button
        onclick={onGoHome}
        class="text-sm text-stone-400 hover:text-stone-200"
      >
        Home
      </button>
    </div>

    {#if gameStore.state}
      <div class="text-sm text-stone-300">
        <span
          class="inline-block h-3 w-3 rounded-full {gameStore.state.current_color === 'black' ? 'bg-stone-900 ring-1 ring-stone-500' : 'bg-stone-100'}"
        ></span>
        {gameStore.state.current_color === "black" ? "Black" : "White"} to play
        &mdash; Move {gameStore.state.move_number}
      </div>

      <ScoreBar
        capturesBlack={gameStore.state.captures_black}
        capturesWhite={gameStore.state.captures_white}
      />

      <GameControls
        onPass={handlePass}
        onResign={handleResign}
        onUndo={handleUndo}
        onNewGame={startNewGame}
        disabled={gameStore.state.phase === "Finished"}
      />

      <MoveHistory game={gameStore.state} />

      <CoachingPanel messages={coachingStore.messages} />

      {#if gameStore.state.phase === "Finished"}
        <div class="rounded bg-amber-900/50 p-3 text-center text-sm text-amber-200">
          Game Over
          {#if gameStore.state.result}
            {#if gameStore.state.result === "Draw"}
              &mdash; Draw
            {:else if typeof gameStore.state.result === "object" && "Resignation" in gameStore.state.result}
              &mdash; {gameStore.state.result.Resignation.winner === "black" ? "Black" : "White"} wins by resignation
            {:else if typeof gameStore.state.result === "object" && "Score" in gameStore.state.result}
              &mdash; {gameStore.state.result.Score.winner === "black" ? "Black" : "White"} wins by {gameStore.state.result.Score.margin}
            {/if}
          {/if}
        </div>
      {/if}
    {:else if gameStore.error}
      <div class="rounded bg-red-900/50 p-3 text-sm text-red-200">
        {gameStore.error}
      </div>
    {:else}
      <p class="text-sm text-stone-500">Loading...</p>
    {/if}
  </div>
</div>
