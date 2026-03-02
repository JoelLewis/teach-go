<script lang="ts">
  import { onMount } from "svelte";
  import RadarChart from "../components/RadarChart.svelte";
  import * as api from "../lib/api/commands";
  import type { SkillProfile, ProblemStats } from "../lib/api/types";

  type Props = {
    onGoHome: () => void;
  };

  let { onGoHome }: Props = $props();

  let profile = $state<SkillProfile | null>(null);
  let problemStats = $state<ProblemStats | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const [p, ps] = await Promise.all([
        api.getSkillProfile(),
        api.getProblemStats(),
      ]);
      profile = p;
      problemStats = ps;
    } catch (e) {
      error = String(e);
    }
  });

  function formatRank(rank: number): string {
    const rounded = Math.round(rank);
    if (rounded <= 1) return "~1 dan";
    return `~${rounded} kyu`;
  }
</script>

<div class="flex h-full flex-col items-center p-8">
  <div class="mb-6 flex w-full max-w-md items-center justify-between">
    <h1 class="text-2xl font-bold text-stone-100">Progress</h1>
    <button
      onclick={onGoHome}
      class="text-sm text-stone-400 hover:text-stone-200"
    >
      Home
    </button>
  </div>

  {#if error}
    <div class="rounded bg-red-900/50 p-3 text-sm text-red-200">{error}</div>
  {:else if !profile}
    <p class="text-sm text-stone-500">Loading...</p>
  {:else if profile.games_played === 0}
    <div class="mt-16 text-center">
      <p class="text-lg text-stone-400">No games played yet</p>
      <p class="mt-2 text-sm text-stone-500">
        Play your first game to see your skill profile.
      </p>
      <button
        onclick={onGoHome}
        class="mt-6 rounded-lg bg-amber-700 px-6 py-3 text-sm font-semibold text-white hover:bg-amber-600"
      >
        Start Playing
      </button>
    </div>
  {:else}
    <div class="w-full max-w-md">
      <div class="mb-6 text-center">
        <p class="text-3xl font-bold text-amber-500">
          {formatRank(profile.overall_rank)}
        </p>
        <p class="text-sm text-stone-400">Estimated rank</p>
      </div>

      <RadarChart {profile} />

      <div class="mt-6 flex justify-between text-sm text-stone-400">
        <span>{profile.games_played} game{profile.games_played === 1 ? "" : "s"} played</span>
        <span>Updated {profile.last_updated.slice(0, 10)}</span>
      </div>

      {#if problemStats && problemStats.total_attempted > 0}
        <div class="mt-8">
          <h2 class="mb-3 text-lg font-semibold text-stone-200">Problem Training</h2>
          <div class="flex gap-4 text-center">
            <div class="flex-1 rounded bg-stone-800 p-3">
              <p class="text-2xl font-bold text-emerald-400">{problemStats.total_solved}</p>
              <p class="text-xs text-stone-500">Solved</p>
            </div>
            <div class="flex-1 rounded bg-stone-800 p-3">
              <p class="text-2xl font-bold text-stone-200">{problemStats.total_attempted}</p>
              <p class="text-xs text-stone-500">Attempted</p>
            </div>
            <div class="flex-1 rounded bg-stone-800 p-3">
              <p class="text-2xl font-bold text-amber-400">{problemStats.accuracy_percent}%</p>
              <p class="text-xs text-stone-500">Accuracy</p>
            </div>
          </div>

          {#if problemStats.per_category.length > 0}
            <div class="mt-3 flex flex-col gap-1">
              {#each problemStats.per_category as cat}
                <div class="flex items-center justify-between rounded bg-stone-800 px-3 py-1.5 text-sm">
                  <span class="text-stone-300">{cat.category}</span>
                  <span class="text-xs text-stone-500">
                    {cat.solved}/{cat.attempted}
                  </span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>
