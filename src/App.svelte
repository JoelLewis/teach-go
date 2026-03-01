<script lang="ts">
  import HomeView from "./views/HomeView.svelte";
  import PlayView from "./views/PlayView.svelte";

  let currentView = $state<"home" | "play">("home");

  function startGame() {
    currentView = "play";
  }

  function goHome() {
    currentView = "home";
  }
</script>

<main class="h-full bg-stone-900 text-stone-100">
  {#if currentView === "home"}
    <HomeView onStartGame={startGame} />
  {:else}
    <PlayView onGoHome={goHome} />
  {/if}
</main>
