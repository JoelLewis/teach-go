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

  function startGame(config: NewGameConfig) {
    gameConfig = config;
    currentView = "play";
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
