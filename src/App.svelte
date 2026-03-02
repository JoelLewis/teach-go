<script lang="ts">
  import HomeView from "./views/HomeView.svelte";
  import PlayView from "./views/PlayView.svelte";
  import ReviewView from "./views/ReviewView.svelte";
  import { gameStore } from "./lib/stores/game.svelte";
  import * as api from "./lib/api/commands";

  let currentView = $state<"home" | "play" | "review">("home");
  let reviewGameId = $state<number | undefined>(undefined);

  function startGame() {
    currentView = "play";
  }

  function goHome() {
    currentView = "home";
  }

  function startReview(gameId?: number) {
    reviewGameId = gameId;
    currentView = "review";
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
    <HomeView onStartGame={startGame} onLoadGame={loadGame} onStartReview={startReview} />
  {:else if currentView === "play"}
    <PlayView onGoHome={goHome} onStartReview={() => startReview()} />
  {:else if currentView === "review"}
    <ReviewView gameId={reviewGameId} onGoHome={goHome} />
  {/if}
</main>
