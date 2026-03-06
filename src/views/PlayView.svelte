<script lang="ts">
  import { onMount } from "svelte";
  import BoardSvg from "../lib/board/BoardSvg.svelte";
  import GameControls from "../components/GameControls.svelte";
  import MoveHistory from "../components/MoveHistory.svelte";
  import CoachingPanel from "../components/CoachingPanel.svelte";
  import DifficultyPrompt from "../components/DifficultyPrompt.svelte";
  import SetupDialog from "../components/SetupDialog.svelte";
  import ScoreBar from "../components/ScoreBar.svelte";
  import { gameStore } from "../lib/stores/game.svelte";
  import { setupStore } from "../lib/stores/setup.svelte";
  import { coachingStore } from "../lib/stores/coaching.svelte";
  import { engineStore } from "../lib/stores/engine.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { boardThemeForName } from "../lib/board/themes";
  import * as sounds from "../lib/audio/sounds";
  import { onEngineStatus, onAiThinking, onCoachingStream } from "../lib/api/events";
  import * as api from "../lib/api/commands";
  import type { CoachingMessage, DifficultySuggestion, GameState, StoneColor, NewGameConfig } from "../lib/api/types";

  type Props = {
    config?: NewGameConfig;
    onGoHome: () => void;
    onStartReview: () => void;
  };

  let { config, onGoHome, onStartReview }: Props = $props();

  let boardSize = $state(config?.boardSize ?? settingsStore.value.board_size);
  let playerColor = $state<StoneColor>(config?.playerColor ?? "black");
  let viewingMove = $state<number | null>(null);
  let viewingState = $state<GameState | null>(null);
  let pendingFeedback = $state<CoachingMessage | null>(null);
  let difficultySuggestion = $state<DifficultySuggestion | null>(null);
  let difficultyChecked = $state(false);
  let engineError = $state<string | null>(null);
  let showSetupDialog = $state(false);
  let pendingUnlisteners: Array<Promise<() => void>> = [];

  const noop = () => {};

  // Whether we're viewing a past position (not the current game state)
  let isViewingHistory = $derived(viewingMove !== null && viewingMove !== (gameStore.state?.move_number ?? 0));
  let displayState = $derived(isViewingHistory && viewingState ? viewingState : gameStore.state);

  // Keep sound state in sync with settings
  $effect(() => {
    sounds.setEnabled(settingsStore.value.sound_enabled);
  });

  onMount(() => {
    // Subscribe to backend events
    pendingUnlisteners.push(
      onEngineStatus((status) => engineStore.setStatus(status)),
    );
    pendingUnlisteners.push(
      onAiThinking((thinking) => engineStore.setAiThinking(thinking)),
    );
    pendingUnlisteners.push(
      onCoachingStream((chunk) => {
        if (chunk.is_complete) {
          coachingStore.completeStream(chunk.move_number);
        } else {
          coachingStore.appendStream(chunk.move_number, chunk.text_delta);
        }
      }),
    );

    checkSetupAndStart();

    return () => {
      for (const p of pendingUnlisteners) {
        p.then((unlisten) => unlisten()).catch(() => {});
      }
      setupStore.cleanup();
    };
  });

  // Check for difficulty suggestion when game finishes (once per game)
  $effect(() => {
    if (gameStore.state?.phase === "Finished" && !difficultyChecked) {
      difficultyChecked = true;
      checkDifficulty();
    }
  });

  async function checkSetupAndStart() {
    await setupStore.refresh();
    if (setupStore.status !== "ready") {
      showSetupDialog = true;
      return;
    }
    startNewGame();
  }

  async function startNewGame() {
    try {
      const state = await api.newGame(boardSize, settingsStore.value.komi, playerColor);
      gameStore.set(state);
      coachingStore.clear();
      difficultyChecked = false;
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
      engineError = null;
    } catch (e) {
      const msg = String(e);
      console.warn("AI move failed:", msg);
      if (msg.includes("not found") || msg.includes("not available")) {
        engineError = "KataGo engine not found. Install KataGo or set KATAGO_BINARY to play against AI.";
      } else {
        engineError = `AI engine error: ${msg}`;
      }
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
    const timing = settingsStore.value.feedback_timing;
    if (timing === "post_game") return; // Coaching deferred to review

    try {
      const feedback = await api.getCoachingFeedback();
      if (timing === "on_demand") {
        // Store feedback but don't show it yet
        pendingFeedback = feedback;
        coachingStore.setLastMoveSeverity(null);
      } else {
        // immediate mode
        if (feedback) {
          coachingStore.add(feedback);
        } else {
          coachingStore.setLastMoveSeverity(null);
        }
      }
    } catch (e) {
      console.warn("Coaching unavailable:", e);
    }
  }

  function revealPendingFeedback() {
    if (pendingFeedback) {
      coachingStore.add(pendingFeedback);
      pendingFeedback = null;
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

  async function checkDifficulty() {
    try {
      difficultySuggestion = await api.checkDifficultySuggestion();
    } catch {
      // Non-fatal
    }
  }

  async function acceptDifficulty() {
    if (!difficultySuggestion) return;
    const strengthMap: Record<string, string[]> = {
      up: ["beginner", "intermediate", "advanced", "dan"],
      down: ["dan", "advanced", "intermediate", "beginner"],
    };
    const levels = strengthMap[difficultySuggestion.direction] ?? [];
    const currentIdx = levels.indexOf(settingsStore.value.ai_strength);
    if (currentIdx >= 0 && currentIdx < levels.length - 1) {
      const newStrength = levels[currentIdx + 1];
      const updated = await api.updateSettings({
        ...settingsStore.value,
        ai_strength: newStrength,
      });
      settingsStore.update(updated);
    }
    difficultySuggestion = null;
  }

  async function handleNavigate(moveNumber: number) {
    if (moveNumber === (gameStore.state?.move_number ?? 0)) {
      // Return to current position
      viewingMove = null;
      viewingState = null;
      return;
    }
    try {
      viewingState = await api.getGamePosition(moveNumber);
      viewingMove = moveNumber;
    } catch (e) {
      console.warn("Navigation failed:", e);
    }
  }

  function returnToCurrent() {
    viewingMove = null;
    viewingState = null;
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
    {#if displayState}
      <div class="relative">
        <BoardSvg
          {boardSize}
          stones={displayState.stones}
          currentColor={displayState.current_color as StoneColor}
          lastMove={displayState.last_move}
          showCoordinates={settingsStore.value.show_coordinates}
          lastMoveSeverity={isViewingHistory ? null : coachingStore.lastMoveSeverity}
          theme={boardThemeForName(themeStore.active)}
          animate={!isViewingHistory}
          onIntersectionClick={isViewingHistory ? noop : handleIntersectionClick}
        />
        {#if isViewingHistory}
          <div class="absolute bottom-3 left-1/2 -translate-x-1/2">
            <button
              onclick={returnToCurrent}
              class="rounded bg-stone-700/90 px-3 py-1 text-xs font-semibold text-stone-200 hover:bg-stone-600 shadow-lg"
            >
              Return to current (move {gameStore.state?.move_number ?? 0})
            </button>
          </div>
        {/if}
      </div>
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

      {#if engineError}
        <div class="rounded bg-amber-900/50 p-2 text-xs text-amber-200">
          {engineError}
          <div class="mt-1 text-amber-400/70">Game continues as human-vs-human.</div>
        </div>
      {:else if engineStore.aiThinking}
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

      <MoveHistory game={gameStore.state} viewingMove={viewingMove} onNavigate={handleNavigate} />

      {#if pendingFeedback}
        <button
          onclick={revealPendingFeedback}
          class="rounded bg-amber-800/60 px-3 py-1.5 text-xs text-amber-200 hover:bg-amber-700/60"
        >
          Show Feedback
        </button>
      {/if}

      <CoachingPanel messages={coachingStore.messages} streamingMoveNumber={coachingStore.streamingMoveNumber} onNavigate={handleNavigate} />

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
          <button
            onclick={onStartReview}
            class="mt-2 rounded bg-amber-700 px-4 py-1 text-sm font-semibold text-white hover:bg-amber-600"
          >
            Review Game
          </button>
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

{#if showSetupDialog}
  <SetupDialog
    onComplete={() => { showSetupDialog = false; startNewGame(); }}
    onSkip={() => { showSetupDialog = false; startNewGame(); }}
  />
{/if}

{#if difficultySuggestion}
  <DifficultyPrompt
    suggestion={difficultySuggestion}
    onAccept={acceptDifficulty}
    onDismiss={() => (difficultySuggestion = null)}
  />
{/if}
