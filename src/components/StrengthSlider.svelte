<script lang="ts">
  import { RANK_LADDER, formatRank, rankIndex } from "../lib/ranks";

  type Props = {
    value: string;
    onChange: (value: string) => void;
    id?: string;
  };

  let { value, onChange, id = "ai-strength-slider" }: Props = $props();

  const index = $derived(rankIndex(value));

  function handleInput(e: Event) {
    const idx = Number((e.target as HTMLInputElement).value);
    const token = RANK_LADDER[idx];
    if (token) onChange(token);
  }
</script>

<div class="block text-sm" style="color: var(--text-secondary);">
  <label for={id} class="mb-1 flex items-baseline justify-between">
    <span>AI Strength</span>
    <span class="font-medium" style="color: var(--text-heading);">{formatRank(value)}</span>
  </label>
  <input
    {id}
    type="range"
    min="0"
    max={RANK_LADDER.length - 1}
    step="1"
    value={index}
    oninput={handleInput}
    aria-valuetext={formatRank(value)}
    class="w-full"
    style="accent-color: var(--accent-primary);"
  />
  <div class="flex justify-between text-xs" style="color: var(--text-dim);" aria-hidden="true">
    <span>20 kyu</span>
    <span>Full strength</span>
  </div>
</div>
