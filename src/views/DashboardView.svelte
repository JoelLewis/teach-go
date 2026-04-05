<script lang="ts">
  import { onMount } from "svelte";
  import RadarChart from "../components/RadarChart.svelte";
  import SkillTrendChart from "../components/SkillTrendChart.svelte";
  import * as api from "../lib/api/commands";
  import type { SkillProfile, SkillSnapshot, ProblemStats } from "../lib/api/types";

  type WindowOption = 7 | 30 | null;

  type Props = {
    onGoHome: () => void;
  };

  let { onGoHome }: Props = $props();

  let profile = $state<SkillProfile | null>(null);
  let problemStats = $state<ProblemStats | null>(null);
  let history = $state<SkillSnapshot[]>([]);
  let historyWindow = $state<WindowOption>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      const [p, ps, h] = await Promise.all([
        api.getSkillProfile(),
        api.getProblemStats(),
        api.getSkillHistory(),
      ]);
      profile = p;
      problemStats = ps;
      history = h;
    } catch (e) {
      error = String(e);
    }
  });

  async function changeWindow(days: WindowOption) {
    historyWindow = days;
    try {
      history = await api.getSkillHistory(days ?? undefined);
    } catch {
      // Keep existing history on error
    }
  }

  function formatRank(rank: number): string {
    const rounded = Math.round(rank);
    if (rounded <= 1) return "~1 dan";
    return `~${rounded} kyu`;
  }
</script>

<div class="flex h-full flex-col items-center overflow-y-auto p-8">
  <div class="mb-6 flex w-full max-w-md items-center justify-between">
    <h1 class="text-2xl font-bold" style="color: var(--text-primary);">Progress</h1>
    <button
      onclick={onGoHome}
      class="text-sm transition-opacity hover:opacity-70"
      style="color: var(--text-secondary);"
    >
      Home
    </button>
  </div>

  {#if error}
    <div class="rounded p-3 text-sm" style="background: color-mix(in srgb, var(--danger) 30%, transparent); color: var(--danger);">{error}</div>
  {:else if !profile}
    <p class="text-sm" style="color: var(--text-dim);">Loading...</p>
  {:else if profile.games_played === 0}
    <div class="mt-16 text-center">
      <p class="text-lg" style="color: var(--text-secondary);">No games played yet</p>
      <p class="mt-2 text-sm" style="color: var(--text-dim);">
        Play your first game to see your skill profile.
      </p>
      <button
        onclick={onGoHome}
        class="btn btn-primary mt-6"
      >
        Start Playing
      </button>
    </div>
  {:else}
    <div class="w-full max-w-md">
      <div class="mb-6 text-center">
        <p class="text-3xl font-bold" style="color: var(--accent-primary);">
          {formatRank(profile.overall_rank)}
        </p>
        <p class="text-sm" style="color: var(--text-secondary);">Estimated rank</p>
      </div>

      <RadarChart {profile} />

      {#if history.length >= 2}
        <div class="mt-6">
          <SkillTrendChart
            snapshots={history}
            onWindowChange={changeWindow}
            activeWindow={historyWindow}
          />
        </div>
      {/if}

      <div class="mt-6 flex justify-between text-sm" style="color: var(--text-secondary);">
        <span>{profile.games_played} game{profile.games_played === 1 ? "" : "s"} played</span>
        <span>Updated {profile.last_updated.slice(0, 10)}</span>
      </div>

      {#if problemStats && problemStats.total_attempted > 0}
        <div class="mt-8">
          <h2 class="mb-3 text-lg font-semibold" style="color: var(--text-primary);">Problem Training</h2>
          <div class="flex gap-4 text-center">
            <div class="flex-1 rounded p-3" style="background: var(--surface-secondary);">
              <p class="text-2xl font-bold" style="color: var(--success);">{problemStats.total_solved}</p>
              <p class="text-xs" style="color: var(--text-dim);">Solved</p>
            </div>
            <div class="flex-1 rounded p-3" style="background: var(--surface-secondary);">
              <p class="text-2xl font-bold" style="color: var(--text-primary);">{problemStats.total_attempted}</p>
              <p class="text-xs" style="color: var(--text-dim);">Attempted</p>
            </div>
            <div class="flex-1 rounded p-3" style="background: var(--surface-secondary);">
              <p class="text-2xl font-bold" style="color: var(--accent-primary);">{problemStats.accuracy_percent}%</p>
              <p class="text-xs" style="color: var(--text-dim);">Accuracy</p>
            </div>
          </div>

          {#if problemStats.per_category.length > 0}
            <div class="mt-3 flex flex-col gap-1">
              {#each problemStats.per_category as cat}
                <div class="flex items-center justify-between rounded px-3 py-1.5 text-sm" style="background: var(--surface-secondary);">
                  <span style="color: var(--text-primary);">{cat.category}</span>
                  <span class="text-xs" style="color: var(--text-dim);">
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
