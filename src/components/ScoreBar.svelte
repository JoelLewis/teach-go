<script lang="ts">
  type Props = {
    capturesBlack: number;
    capturesWhite: number;
  };

  let { capturesBlack, capturesWhite }: Props = $props();
</script>

<div class="flex items-center justify-between rounded bg-stone-800 p-2 text-sm">
  <div class="flex items-center gap-2">
    <span class="inline-block h-3 w-3 rounded-full bg-stone-900 ring-1 ring-stone-600"></span>
    <span class="text-stone-300">Black: {capturesBlack} captures</span>
  </div>
  <div class="flex items-center gap-2">
    <span class="text-stone-300">White: {capturesWhite} captures</span>
    <span class="inline-block h-3 w-3 rounded-full bg-stone-100 ring-1 ring-stone-600"></span>
  </div>
</div>
