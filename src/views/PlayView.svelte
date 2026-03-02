<script lang="ts">
  import { onMount } from "svelte";
  import BoardCanvas from "../lib/board/BoardCanvas.svelte";
  import GameControls from "../components/GameControls.svelte";
  import MoveHistory from "../components/MoveHistory.svelte";
  import CoachingPanel from "../components/CoachingPanel.svelte";
  import ScoreBar from "../components/ScoreBar.svelte";
  import { gameStore } from "../lib/stores/game.svelte";
  import { coachingStore } from "../lib/stores/coaching.svelte";
  import { engineStore } from "../lib/stores/engine.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import * as sounds from "../lib/audio/sounds";
  import { onEngineStatus, onAiThinking } from "../lib/api/events";
  import * as api from "../lib/api/commands";
  import type { StoneColor } from "../lib/api/types";

  type Props = {
    onGoHome: () => void;
  };

  let { onGoHome }: Props = $props();

  let boardSize = $state(9);
  let playerColor = $state<StoneColor>("black");
  let unlisteners: Array<() => void> = [];

  // Keep sound state in sync with settings
  $effect(() => {
    sounds.setEnabled(settingsStore.value.sound_enabled);
  });

  onMount(() => {
    // Subscribe to backend events
    onEngineStatus((status) => engineStore.setStatus(status)).then((u) =>
      unlisteners.push(u),
    );
    onAiThinking((thinking) => engineStore.setAiThinking(thinking)).then((u) =>
      unlisteners.push(u),
    );

    startNewGame();

    return () => {
      for (const unlisten of unlisteners) unlisten();
    };
  });

  async function startNewGame() {
    try {
      const state = await api.newGame(boardSize, undefined, playerColor);
      gameStore.set(state);
      coachingStore.clear();
      // If player is white, AI (black) moves first
      if (playerColor === "white") {
        await triggerAiMove();
      }
    } catch (e) {
      gameStore.setError(String(e));
    }
  }

  async function triggerAiMove() {
    if (!gameStore.state || gameStore.state.phase === "Finished") return;

    const aiColor = playerColor === "black" ? "white" : "black";
    if (gameStore.state.current_color !== aiColor) return;

    try {
      const state = await api.requestAiMove();
      gameStore.set(state);
    } catch (e) {
      // Non-fatal — show as warning, game continues as human-vs-human
      console.warn("AI move failed:", e);
    }
  }

  async function handleIntersectionClick(row: number, col: number) {
    if (!gameStore.state || gameStore.state.phase === "Finished") return;
    if (engineStore.aiThinking) return;

    const prevCaptures = gameStore.state
      ? gameStore.state.captures_black + gameStore.state.captures_white
      : 0;

    try {
      const state = await api.playMove(row, col);
      const newCaptures = state.captures_black + state.captures_white;
      if (newCaptures > prevCaptures) {
        sounds.play("capture");
      } else {
        sounds.play("stone");
      }
      gameStore.set(state);
      triggerCoaching();
      triggerAiMove();
    } catch (e) {
      console.warn("Move rejected:", e);
    }
  }

  async function handlePass() {
    if (engineStore.aiThinking) return;
    try {
      const state = await api.passTurn();
      sounds.play("pass");
      gameStore.set(state);
      triggerAiMove();
    } catch (e) {
      gameStore.setError(String(e));
    }
  }

  async function triggerCoaching() {
    try {
      const feedback = await api.getCoachingFeedback();
      if (feedback) coachingStore.add(feedback);
    } catch (e) {
      console.warn("Coaching unavailable:", e);
    }
  }

  async function handleResign() {
    if (engineStore.aiThinking) return;
    try {
      const [state] = await api.resign();
      gameStore.set(state);
    } catch (e) {
      gameStore.setError(String(e));
    }
  }

  async function handleUndo() {
    if (engineStore.aiThinking) return;
    try {
      const state = await api.undoMove();
      gameStore.set(state);
    } catch (e) {
      gameStore.setError(String(e));
    }
  }

  async function handleSave() {
    try {
      await api.saveGameSgf();
    } catch (e) {
      gameStore.setError(String(e));
    }
  }

  async function handleLoad() {
    try {
      const state = await api.loadGameSgf();
      if (state) {
        gameStore.set(state);
        boardSize = state.board_size;
        coachingStore.clear();
      }
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
        showCoordinates={settingsStore.value.show_coordinates}
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

      {#if engineStore.aiThinking}
        <div
          class="flex items-center gap-2 rounded bg-blue-900/40 p-2 text-sm text-blue-200"
        >
          <span class="inline-block h-3 w-3 animate-spin rounded-full border-2 border-blue-400 border-t-transparent"></span>
          AI is thinking&hellip;
        </div>
      {/if}

      <ScoreBar
        capturesBlack={gameStore.state.captures_black}
        capturesWhite={gameStore.state.captures_white}
      />

      <GameControls
        onPass={handlePass}
        onResign={handleResign}
        onUndo={handleUndo}
        onNewGame={startNewGame}
        onSave={handleSave}
        onLoad={handleLoad}
        disabled={gameStore.state.phase === "Finished" ||
          engineStore.aiThinking}
      />

      <MoveHistory game={gameStore.state} />

      <CoachingPanel messages={coachingStore.messages} />

      {#if gameStore.state.phase === "Finished"}
        <div
          class="rounded bg-amber-900/50 p-3 text-center text-sm text-amber-200"
        >
          Game Over
          {#if gameStore.state.result}
            {#if gameStore.state.result === "Draw"}
              &mdash; Draw
            {:else if typeof gameStore.state.result === "object" && "Resignation" in gameStore.state.result}
              &mdash; {gameStore.state.result.Resignation.winner === "black"
                ? "Black"
                : "White"} wins by resignation
            {:else if typeof gameStore.state.result === "object" && "Score" in gameStore.state.result}
              &mdash; {gameStore.state.result.Score.winner === "black"
                ? "Black"
                : "White"} wins by {gameStore.state.result.Score.margin} points
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
