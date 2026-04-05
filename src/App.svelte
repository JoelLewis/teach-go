<script lang="ts">
  import { onMount } from "svelte";
  import HomeView from "./views/HomeView.svelte";
  import PlayView from "./views/PlayView.svelte";
  import ReviewView from "./views/ReviewView.svelte";
  import DashboardView from "./views/DashboardView.svelte";
  import ProblemView from "./views/ProblemView.svelte";
  import OnboardingView from "./views/OnboardingView.svelte";
  import { gameStore } from "./lib/stores/game.svelte";
  import { themeStore } from "./lib/stores/theme.svelte";
  import { settingsStore } from "./lib/stores/settings.svelte";
  import * as api from "./lib/api/commands";
  import type { NewGameConfig, ThemeName } from "./lib/api/types";

  let currentView = $state<"home" | "play" | "review" | "dashboard" | "problem" | "onboarding">("home");
  let reviewGameId = $state<number | undefined>(undefined);
  let gameConfig = $state<NewGameConfig | undefined>(undefined);
  let transitioning = $state(false);
  let transitionBoardSize = $state(9);

  onMount(async () => {
    try {
      const settings = await api.getSettings();
      settingsStore.update(settings);
      themeStore.set(settings.theme as ThemeName);
      if (!settings.onboarding_completed) {
        currentView = "onboarding";
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
      themeStore.set("study");
    }
  });

  async function startGame(config: NewGameConfig) {
    transitionBoardSize = config.boardSize;
    transitioning = true;
    await new Promise(resolve => setTimeout(resolve, 400));
    gameConfig = config;
    currentView = "play";
    setTimeout(() => { transitioning = false; }, 50);
  }

  function goHome() {
    currentView = "home";
  }

  function showDashboard() {
    currentView = "dashboard";
  }

  function startReview(gameId?: number) {
    reviewGameId = gameId;
    currentView = "review";
  }

  function startProblems() {
    currentView = "problem";
  }

  async function loadGame(gameId: number) {
    try {
      const state = await api.loadSavedGame(gameId);
      gameStore.set(state);
      currentView = "play";
    } catch (e) {
      console.error("Failed to load game:", e);
    }
  }
</script>

<main class="h-full" style="background-color: var(--surface-primary, #1c1917); color: var(--text-primary, #f5f5f4);">
  {#if currentView === "onboarding"}
    <OnboardingView onComplete={() => { currentView = "home"; }} />
  {:else if currentView === "home"}
    <HomeView onStartGame={startGame} onLoadGame={loadGame} onStartReview={startReview} onShowDashboard={showDashboard} onStartProblems={startProblems} />
  {:else if currentView === "play"}
    <PlayView config={gameConfig} onGoHome={goHome} onStartReview={() => startReview()} />
  {:else if currentView === "review"}
    <ReviewView gameId={reviewGameId} onGoHome={goHome} />
  {:else if currentView === "dashboard"}
    <DashboardView onGoHome={goHome} />
  {:else if currentView === "problem"}
    <ProblemView onGoHome={goHome} />
  {/if}
</main>

{#if transitioning}
  <div
    class="fixed inset-0 z-40 flex items-center justify-center"
    style="background-color: var(--surface-board); animation: board-fade-in 400ms cubic-bezier(0.33, 1, 0.68, 1) forwards;"
  >
    <svg
      viewBox="0 0 600 600"
      style="width: 80vh; height: 80vh;"
    >
      {#each Array(transitionBoardSize) as _, i}
        {@const padding = 36}
        {@const cellSize = (600 - 2 * padding) / (transitionBoardSize - 1)}
        {@const p = padding + i * cellSize}
        {@const start = padding}
        {@const end = padding + (transitionBoardSize - 1) * cellSize}
        <line x1={start} y1={p} x2={end} y2={p} stroke="currentColor" stroke-width="1" opacity="0.5" />
        <line x1={p} y1={start} x2={p} y2={end} stroke="currentColor" stroke-width="1" opacity="0.5" />
      {/each}
    </svg>
  </div>
{/if}

<style>
  @keyframes board-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
