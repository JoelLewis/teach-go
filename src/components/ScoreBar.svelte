<script lang="ts">
  type Props = {
    capturesBlack: number;
    capturesWhite: number;
  };

  let { capturesBlack, capturesWhite }: Props = $props();
</script>

<div class="flex items-center justify-between rounded p-2 text-sm" style="background-color: var(--surface-card);">
  <div class="flex items-center gap-2">
    <span class="inline-block h-3 w-3 rounded-full bg-stone-900" style="box-shadow: 0 0 0 1px var(--border-subtle);"></span>
    <span style="color: var(--text-on-card);">Black: {capturesBlack} captures</span>
  </div>
  <div class="flex items-center gap-2">
    <span style="color: var(--text-on-card);">White: {capturesWhite} captures</span>
    <span class="inline-block h-3 w-3 rounded-full bg-stone-100" style="box-shadow: 0 0 0 1px var(--border-subtle);"></span>
  </div>
</div>
