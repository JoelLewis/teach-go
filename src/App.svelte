<script lang="ts">
  import HomeView from "./views/HomeView.svelte";
  import PlayView from "./views/PlayView.svelte";
  import { gameStore } from "./lib/stores/game.svelte";
  import * as api from "./lib/api/commands";

  let currentView = $state<"home" | "play">("home");

  function startGame() {
    currentView = "play";
  }

  function goHome() {
    currentView = "home";
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
    <HomeView onStartGame={startGame} onLoadGame={loadGame} />
  {:else}
    <PlayView onGoHome={goHome} />
  {/if}
</main>
