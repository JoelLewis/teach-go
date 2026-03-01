<script lang="ts">
  import type { GameState } from "../lib/api/types";

  type Props = {
    game: GameState;
  };

  let { game }: Props = $props();
</script>

<div class="flex flex-col gap-1">
  <h3 class="text-sm font-semibold text-stone-400">Move History</h3>
  <div class="max-h-48 overflow-y-auto rounded bg-stone-800 p-2 text-xs">
    {#if game.move_number === 0}
      <p class="text-stone-500 italic">No moves yet</p>
    {:else}
      <p class="text-stone-300">
        Move {game.move_number} &mdash;
        {game.current_color === "black" ? "White" : "Black"} played
        {#if game.last_move}
          at ({game.last_move[0]}, {game.last_move[1]})
        {/if}
      </p>
    {/if}
  </div>
</div>
