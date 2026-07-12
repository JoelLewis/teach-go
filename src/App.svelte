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
  import { navHistory, type AppView, type NavEntry } from "./lib/stores/navHistory.svelte";
  import * as api from "./lib/api/commands";
  import type { NewGameConfig, ThemeName } from "./lib/api/types";

  let currentView = $state<AppView>("home");
  let reviewGameId = $state<number | undefined>(undefined);

  if (import.meta.env.DEV) {
    // Lets window.__playtest.getView() report the active view (dev builds only).
    import("./lib/dev/playtest").then((p) => p.registerViewGetter(() => currentView));
  }

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

  function currentNavEntry(): NavEntry | null {
    if (currentView === "onboarding") return null;
    return { view: currentView, reviewGameId };
  }

  function navigateTo(view: NavEntry["view"], gameId?: number) {
    const from = currentNavEntry();
    if (from && (from.view !== view || from.reviewGameId !== gameId)) {
      navHistory.push(from);
    }
    reviewGameId = gameId;
    currentView = view;
  }

  function goBack() {
    const current = currentNavEntry();
    if (!current) return;
    const entry = navHistory.goBack(current);
    if (!entry) return;
    reviewGameId = entry.reviewGameId;
    currentView = entry.view;
  }

  function goForward() {
    const current = currentNavEntry();
    if (!current) return;
    const entry = navHistory.goForward(current);
    if (!entry) return;
    reviewGameId = entry.reviewGameId;
    currentView = entry.view;
  }

  function isTypingTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    return target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
  }

  function isDialogOpen(): boolean {
    return document.querySelector('[role="dialog"], dialog[open]') !== null;
  }

  function handleNavKeydown(e: KeyboardEvent) {
    if (isTypingTarget(e.target) || isDialogOpen()) return;
    const cmd = e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey;
    const alt = e.altKey && !e.metaKey && !e.ctrlKey && !e.shiftKey;
    if ((cmd && e.key === "[") || (alt && e.key === "ArrowLeft")) {
      e.preventDefault();
      goBack();
    } else if ((cmd && e.key === "]") || (alt && e.key === "ArrowRight")) {
      e.preventDefault();
      goForward();
    }
  }

  function handleNavMousedown(e: MouseEvent) {
    if (isDialogOpen()) return;
    if (e.button === 3) {
      e.preventDefault();
      goBack();
    } else if (e.button === 4) {
      e.preventDefault();
      goForward();
    }
  }

  async function startGame(config: NewGameConfig) {
    transitionBoardSize = config.boardSize;
    transitioning = true;
    await new Promise(resolve => setTimeout(resolve, 400));
    // Clear any previous game so PlayView starts a fresh one for this config
    // (PlayView skips startNewGame when a game is already in the store).
    gameStore.clear();
    gameConfig = config;
    navigateTo("play");
    setTimeout(() => { transitioning = false; }, 50);
  }

  function goHome() {
    navigateTo("home");
  }

  function showDashboard() {
    navigateTo("dashboard");
  }

  function startReview(gameId?: number) {
    navigateTo("review", gameId);
  }

  function startProblems() {
    navigateTo("problem");
  }

  async function loadGame(gameId: number) {
    try {
      const state = await api.loadSavedGame(gameId);
      gameStore.set(state);
      // Drop any stale config (hotseat / vs-AI) from a previous session —
      // PlayView must not start a fresh game over the loaded one.
      gameConfig = undefined;
      navigateTo("play");
    } catch (e) {
      console.error("Failed to load game:", e);
    }
  }
</script>

<svelte:window onkeydown={handleNavKeydown} onmousedown={handleNavMousedown} />

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
