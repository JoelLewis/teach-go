<script lang="ts">
  import { onMount } from "svelte";
  import BoardSvg from "../lib/board/BoardSvg.svelte";
  import BoardToast from "../components/BoardToast.svelte";
  import GameControls from "../components/GameControls.svelte";
  import MoveHistory from "../components/MoveHistory.svelte";
  import CoachingPanel from "../components/CoachingPanel.svelte";
  import DifficultyPrompt from "../components/DifficultyPrompt.svelte";
  import ScoreBar from "../components/ScoreBar.svelte";
  import { gameStore } from "../lib/stores/game.svelte";
  import { downloadStore } from "../lib/stores/download.svelte";
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
  let inputLocked = $state(false);
  let startingGame = $state(false);
  let configStrengthApplied = $state(false);
  let pendingUnlisteners: Array<Promise<() => void>> = [];

  const noop = () => {};

  // Whether we're viewing a past position (not the current game state)
  let latestCoachingMessage = $derived(
    coachingStore.messages.length > 0
      ? coachingStore.messages[coachingStore.messages.length - 1]
      : null
  );

  let isViewingHistory = $derived(viewingMove !== null && viewingMove !== (gameStore.state?.move_number ?? 0));
  let displayState = $derived(isViewingHistory && viewingState ? viewingState : gameStore.state);
  let isPlayerTurn = $derived(gameStore.state?.current_color === playerColor);
  let canPlayBoard = $derived(
    !isViewingHistory &&
    !!gameStore.state &&
    gameStore.state.phase === "Playing" &&
    isPlayerTurn &&
    !engineStore.aiThinking &&
    !inputLocked,
  );

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

    downloadStore.startListening();
    downloadStore.refresh();
    checkSetupAndStart();

    return () => {
      for (const p of pendingUnlisteners) {
        p.then((unlisten) => unlisten()).catch(() => {});
      }
      downloadStore.cleanup();
    };
  });

  // Auto-start game when KataGo becomes ready
  $effect(() => {
    if (downloadStore.katagoReady && !gameStore.state && !startingGame) {
      startNewGame();
    }
  });

  // Check for difficulty suggestion when game finishes (once per game)
  $effect(() => {
    if (gameStore.state?.phase === "Finished" && !difficultyChecked) {
      difficultyChecked = true;
      checkDifficulty();
    }
  });

  async function checkSetupAndStart() {
    await downloadStore.refresh();
    if (downloadStore.katagoReady) {
      startNewGame();
    }
    // If not ready, the $effect above will start the game when KataGo finishes downloading
  }

  async function startNewGame() {
    if (startingGame) return;

    startingGame = true;
    try {
      if (!configStrengthApplied && config?.aiStrength && config.aiStrength !== settingsStore.value.ai_strength) {
        const nextSettings = await api.updateSettings({
          ...settingsStore.value,
          ai_strength: config.aiStrength,
        });
        settingsStore.update(nextSettings);
      }
      configStrengthApplied = true;

      inputLocked = false;
      engineError = null;
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
    } finally {
      startingGame = false;
    }
  }

  async function triggerAiMove() {
    if (!gameStore.state || gameStore.state.phase === "Finished") return;

    const aiColor = playerColor === "black" ? "white" : "black";
    if (gameStore.state.current_color !== aiColor) return;

    try {
      inputLocked = true;
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
    } finally {
      inputLocked = false;
    }
  }

  async function handleIntersectionClick(row: number, col: number) {
    if (!canPlayBoard || !gameStore.state) return;

    const prevCaptures = gameStore.state
      ? gameStore.state.captures_black + gameStore.state.captures_white
      : 0;

    try {
      inputLocked = true;
      const state = await api.playMove(row, col);
      const newCaptures = state.captures_black + state.captures_white;
      if (newCaptures > prevCaptures) {
        sounds.play("capture");
      } else {
        sounds.play("stone");
      }
      gameStore.set(state);
      triggerCoaching();
      await triggerAiMove();
    } catch (e) {
      console.warn("Move rejected:", e);
    } finally {
      inputLocked = false;
    }
  }

  async function handlePass() {
    if (!gameStore.state || gameStore.state.phase === "Finished" || !isPlayerTurn || inputLocked || engineStore.aiThinking) return;
    try {
      inputLocked = true;
      const state = await api.passTurn();
      sounds.play("pass");
      gameStore.set(state);
      await triggerAiMove();
    } catch (e) {
      gameStore.setError(String(e));
    } finally {
      inputLocked = false;
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
    if (!gameStore.state || gameStore.state.phase === "Finished" || inputLocked || engineStore.aiThinking) return;
    try {
      inputLocked = true;
      const [state] = await api.resign();
      gameStore.set(state);
    } catch (e) {
      gameStore.setError(String(e));
    } finally {
      inputLocked = false;
    }
  }

  async function handleUndo() {
    if (inputLocked || engineStore.aiThinking) return;
    try {
      inputLocked = true;
      const state = await api.undoMove();
      gameStore.set(state);
    } catch (e) {
      gameStore.setError(String(e));
    } finally {
      inputLocked = false;
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

<div class="flex h-full flex-col lg:flex-row">
  <!-- Board area -->
  <div class="flex flex-1 min-w-0 min-h-[50vh] lg:min-h-0 items-center justify-center p-4">
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
          interactive={canPlayBoard}
          onIntersectionClick={isViewingHistory ? noop : handleIntersectionClick}
        />
        {#if isViewingHistory}
          <div class="absolute bottom-3 left-1/2 -translate-x-1/2">
            <button
              onclick={returnToCurrent}
              class="btn btn-secondary btn-sm shadow-md"
            >
              Return to current (move {gameStore.state?.move_number ?? 0})
            </button>
          </div>
        {/if}
        <BoardToast
          message={latestCoachingMessage}
          onClickMessage={(moveNumber) => handleNavigate(moveNumber)}
        />
      </div>
    {/if}
  </div>

  <!-- Right panel -->
  <div class="flex w-full lg:w-80 flex-col gap-4 border-t lg:border-t-0 lg:border-l overflow-y-auto max-h-[50vh] lg:max-h-none p-4" style="border-color: var(--panel-border);">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold" style="color: var(--text-primary);">Game</h2>
      <button
        onclick={onGoHome}
        class="text-sm transition-opacity hover:opacity-70"
        style="color: var(--text-secondary);"
      >
        Home
      </button>
    </div>

    {#if !downloadStore.katagoReady}
      <div class="rounded p-3 text-sm" style="background-color: color-mix(in srgb, var(--info) 15%, transparent); color: var(--info);">
        {#if downloadStore.katagoDownloading}
          <div class="mb-1 font-semibold">Downloading KataGo{downloadStore.katagoPhase ? ` (${downloadStore.katagoPhase})` : ""}...</div>
          <div class="h-2 w-full overflow-hidden rounded" style="background-color: var(--surface-secondary);">
            <div class="h-full rounded transition-all duration-300" style="width: {downloadStore.katagoProgress}%; background-color: var(--accent-primary);"></div>
          </div>
          <div class="mt-1 text-xs" style="color: var(--text-dim);">{Math.round(downloadStore.katagoProgress)}%</div>
        {:else if downloadStore.katagoError}
          <div style="color: var(--danger);">Download failed: {downloadStore.katagoError}</div>
          <button
            onclick={() => downloadStore.retry()}
            class="btn btn-primary btn-sm mt-2"
          >
            Retry
          </button>
        {:else}
          <div>Waiting for KataGo download...</div>
        {/if}
      </div>
    {/if}

    {#if gameStore.state}
      <div class="text-sm" style="color: var(--text-secondary);">
        <span
          class="inline-block h-3 w-3 rounded-full {gameStore.state.current_color === 'black' ? 'bg-stone-900' : 'bg-stone-100'}"
          style="{gameStore.state.current_color === 'black' ? `box-shadow: 0 0 0 1px var(--border-subtle);` : ''}"
        ></span>
        {gameStore.state.current_color === "black" ? "Black" : "White"} to play
        &mdash; Move {gameStore.state.move_number}
      </div>

      {#if engineError}
        <div class="rounded p-2 text-xs" style="background-color: color-mix(in srgb, var(--danger) 20%, transparent); color: var(--danger);">
          {engineError}
          <div class="mt-1" style="color: var(--text-dim);">Game continues as human-vs-human.</div>
        </div>
      {:else if engineStore.aiThinking}
        <div
          class="flex items-center gap-2 rounded p-2 text-sm"
          style="background-color: color-mix(in srgb, var(--info) 20%, transparent); color: var(--info);"
        >
          <span class="inline-block h-3 w-3 animate-spin rounded-full border-2 border-t-transparent" style="border-color: var(--info); border-top-color: transparent;"></span>
          AI is thinking&hellip;
        </div>
      {/if}

      <div class="flex flex-col gap-1.5">
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
            engineStore.aiThinking ||
            inputLocked}
        />
      </div>

      <MoveHistory game={gameStore.state} viewingMove={viewingMove} onNavigate={handleNavigate} />

      {#if pendingFeedback}
        <button
          onclick={revealPendingFeedback}
          class="btn btn-primary btn-sm"
        >
          Show Feedback
        </button>
      {/if}

      <div class="mt-3">
        <CoachingPanel messages={coachingStore.messages} streamingMoveNumber={coachingStore.streamingMoveNumber} onNavigate={handleNavigate} />
      </div>

      {#if gameStore.state.phase === "Finished"}
        <div
          class="rounded p-3 text-center text-sm"
          style="background-color: var(--surface-secondary); color: var(--text-primary);"
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
            class="btn btn-primary btn-sm mt-2"
          >
            Review Game
          </button>
        </div>
      {/if}
    {:else if gameStore.error}
      <div class="rounded p-3 text-sm" style="background-color: color-mix(in srgb, var(--danger) 20%, transparent); color: var(--danger);">
        {gameStore.error}
      </div>
    {:else}
      <p class="text-sm" style="color: var(--text-dim);">Loading...</p>
    {/if}
  </div>
</div>

{#if difficultySuggestion}
  <DifficultyPrompt
    suggestion={difficultySuggestion}
    onAccept={acceptDifficulty}
    onDismiss={() => (difficultySuggestion = null)}
  />
{/if}
