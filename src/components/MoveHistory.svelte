<script lang="ts">
  import type { GameState, MoveEntry } from "../lib/api/types";
  import { toGtp } from "../lib/utils/coordinates";

  type Props = {
    game: GameState;
    viewingMove?: number | null;
    onNavigate?: (moveNumber: number) => void;
  };

  let { game, viewingMove = null, onNavigate }: Props = $props();

  function moveLabel(entry: MoveEntry): string {
    if (entry.is_pass) return "Pass";
    if (entry.row != null && entry.col != null) {
      return toGtp(entry.row, entry.col, game.board_size);
    }
    return "?";
  }

  // Group moves into pairs: [black, white?]
  let pairs = $derived.by(() => {
    const result: { number: number; black: MoveEntry | null; white: MoveEntry | null }[] = [];
    for (let i = 0; i < game.moves.length; i += 2) {
      const black = game.moves[i] ?? null;
      const white = game.moves[i + 1] ?? null;
      result.push({ number: Math.floor(i / 2) + 1, black, white });
    }
    return result;
  });
</script>

<div class="flex flex-col gap-1">
  <h3 class="text-sm font-semibold text-stone-400">Move History</h3>
  <div class="max-h-48 overflow-y-auto rounded bg-stone-800 p-2 text-xs font-mono">
    {#if game.moves.length === 0}
      <p class="text-stone-500 italic font-sans">No moves yet</p>
    {:else}
      <div class="flex flex-col gap-0.5">
        {#each pairs as pair}
          <div class="flex gap-1 items-baseline">
            <span class="w-6 text-right text-stone-600 shrink-0">{pair.number}.</span>
            {#if pair.black}
              <button
                class="px-1 rounded hover:bg-stone-700 {viewingMove === pair.black.move_number ? 'bg-stone-600 text-white' : 'text-stone-300'}"
                onclick={() => onNavigate?.(pair.black!.move_number)}
              >
                {moveLabel(pair.black)}
              </button>
            {/if}
            {#if pair.white}
              <button
                class="px-1 rounded hover:bg-stone-700 {viewingMove === pair.white.move_number ? 'bg-stone-600 text-white' : 'text-stone-300'}"
                onclick={() => onNavigate?.(pair.white!.move_number)}
              >
                {moveLabel(pair.white)}
              </button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
