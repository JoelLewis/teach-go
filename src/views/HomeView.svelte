<script lang="ts">
  import { onMount } from "svelte";
  import NewGameDialog from "../components/NewGameDialog.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import * as api from "../lib/api/commands";
  import type { SavedGame, NewGameConfig } from "../lib/api/types";

  type Props = {
    onStartGame: (config: NewGameConfig) => void;
    onLoadGame: (gameId: number) => void;
    onStartReview: (gameId: number) => void;
    onShowDashboard: () => void;
    onStartProblems: () => void;
  };

  let { onStartGame, onLoadGame, onStartReview, onShowDashboard, onStartProblems }: Props = $props();

  let showNewGameDialog = $state(false);
  let recentGames = $state<SavedGame[]>([]);

  onMount(async () => {
    try {
      recentGames = await api.listGames();
    } catch {
      // Silently fail — no games yet is fine
    }
  });

  function formatResult(result: string): string {
    if (result.endsWith("+R")) return `${result[0] === "B" ? "Black" : "White"} by resignation`;
    if (result === "0") return "Draw";
    const match = result.match(/^([BW])\+(.+)$/);
    if (match) return `${match[1] === "B" ? "Black" : "White"} by ${match[2]} pts`;
    return result;
  }
</script>

<div class="flex h-full flex-col items-center justify-center gap-8">
  <div class="text-center">
    <h1 class="text-5xl font-bold text-amber-500">GoSensei</h1>
    <p class="mt-2 text-lg text-stone-400">Your AI Go tutor</p>
  </div>

  <div class="flex flex-col items-center gap-3">
    <button
      onclick={() => (showNewGameDialog = true)}
      class="rounded-lg bg-amber-700 px-8 py-4 text-lg font-semibold text-white shadow-lg transition hover:bg-amber-600 hover:shadow-xl"
    >
      New Game
    </button>
    <button
      onclick={onStartProblems}
      class="rounded-lg bg-teal-700 px-8 py-4 text-lg font-semibold text-white shadow-lg transition hover:bg-teal-600 hover:shadow-xl"
    >
      Practice Problems
    </button>
    <button
      onclick={onShowDashboard}
      class="rounded-lg bg-stone-700 px-6 py-2 text-sm font-semibold text-stone-200 transition hover:bg-stone-600"
    >
      Progress
    </button>
  </div>

  <p class="max-w-md text-center text-sm text-stone-500">
    Choose your board size and color, then let GoSensei guide you.
  </p>

  {#if recentGames.length > 0}
    <div class="w-full max-w-md">
      <h2 class="mb-2 text-sm font-semibold text-stone-400">Recent Games</h2>
      <div class="flex flex-col gap-1">
        {#each recentGames.slice(0, 10) as game}
          <div class="flex items-center gap-1 rounded bg-stone-800 px-3 py-2 text-sm text-stone-300">
            <button
              onclick={() => onLoadGame(game.id)}
              class="flex flex-1 items-center justify-between text-left hover:text-stone-100"
            >
              <span>{game.board_size}x{game.board_size}</span>
              <span class="text-stone-400">{formatResult(game.result)}</span>
              <span class="text-xs text-stone-500">{game.played_at.slice(0, 10)}</span>
            </button>
            <button
              onclick={() => onStartReview(game.id)}
              class="rounded bg-stone-700 px-2 py-0.5 text-xs text-amber-400 hover:bg-stone-600"
              title="Review this game"
            >
              Review
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if showNewGameDialog}
    <NewGameDialog
      settings={settingsStore.value}
      onClose={() => (showNewGameDialog = false)}
      onStart={(config) => {
        showNewGameDialog = false;
        onStartGame(config);
      }}
    />
  {/if}
</div>
