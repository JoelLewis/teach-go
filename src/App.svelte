<script lang="ts">
  import HomeView from "./views/HomeView.svelte";
  import PlayView from "./views/PlayView.svelte";
  import ReviewView from "./views/ReviewView.svelte";
  import DashboardView from "./views/DashboardView.svelte";
  import ProblemView from "./views/ProblemView.svelte";
  import { gameStore } from "./lib/stores/game.svelte";
  import * as api from "./lib/api/commands";
  import type { NewGameConfig } from "./lib/api/types";

  let currentView = $state<"home" | "play" | "review" | "dashboard" | "problem">("home");
  let reviewGameId = $state<number | undefined>(undefined);
  let gameConfig = $state<NewGameConfig | undefined>(undefined);

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

<main class="h-full bg-stone-900 text-stone-100">
  {#if currentView === "home"}
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
