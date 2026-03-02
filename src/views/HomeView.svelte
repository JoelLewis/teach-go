<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "../lib/api/commands";
  import type { SavedGame } from "../lib/api/types";

  type Props = {
    onStartGame: () => void;
    onLoadGame: (gameId: number) => void;
    onStartReview: (gameId: number) => void;
  };

  let { onStartGame, onLoadGame, onStartReview }: Props = $props();

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

  <button
    onclick={onStartGame}
    class="rounded-lg bg-amber-700 px-8 py-4 text-lg font-semibold text-white shadow-lg transition hover:bg-amber-600 hover:shadow-xl"
  >
    New Game
  </button>

  <p class="max-w-md text-center text-sm text-stone-500">
    Start with a 9x9 board to learn the fundamentals.
    GoSensei will guide you through each move.
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
</div>
