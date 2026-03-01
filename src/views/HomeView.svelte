<script lang="ts">
  type Props = {
    onStartGame: () => void;
  };

  let { onStartGame }: Props = $props();
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
</div>
