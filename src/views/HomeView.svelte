<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import NewGameDialog from "../components/NewGameDialog.svelte";
  import SettingsDialog from "../components/SettingsDialog.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { downloadStore } from "../lib/stores/download.svelte";
  import * as api from "../lib/api/commands";
  import type { SavedGame, NewGameConfig, ThemeName } from "../lib/api/types";

  type Props = {
    onStartGame: (config: NewGameConfig) => void;
    onLoadGame: (gameId: number) => void;
    onStartReview: (gameId: number) => void;
    onShowDashboard: () => void;
    onStartProblems: () => void;
  };

  let { onStartGame, onLoadGame, onStartReview, onShowDashboard, onStartProblems }: Props = $props();

  let showNewGameDialog = $state(false);
  let showSettingsDialog = $state(false);
  let recentGames = $state<SavedGame[]>([]);

  async function handleSettingsSave(updated: typeof settingsStore.value) {
    try {
      const saved = await api.updateSettings(updated);
      settingsStore.update(saved);
      themeStore.set(saved.theme as ThemeName);
      showSettingsDialog = false;
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  onMount(async () => {
    downloadStore.startListening();
    downloadStore.refresh();
    try {
      recentGames = await api.listGames();
    } catch {
      // Silently fail — no games yet is fine
    }
  });

  onDestroy(() => {
    downloadStore.cleanup();
  });

  function formatResult(result: string): string {
    if (result.endsWith("+R")) return `${result[0] === "B" ? "Black" : "White"} by resignation`;
    if (result === "0") return "Draw";
    const match = result.match(/^([BW])\+(.+)$/);
    if (match) return `${match[1] === "B" ? "Black" : "White"} by ${match[2]} pts`;
    return result;
  }
</script>

<div class="flex h-full flex-col items-center justify-center gap-8" style="background-color: var(--surface-primary); color: var(--text-primary);">
  <div class="text-center">
    <h1 class="text-5xl font-bold" style="color: var(--accent-primary);">GoSensei</h1>
    <p class="mt-2 text-lg" style="color: var(--text-secondary);">Your AI Go tutor</p>
  </div>

  <div class="flex flex-col items-center gap-3">
    <button
      onclick={() => (showNewGameDialog = true)}
      class="rounded-lg px-8 py-4 text-lg font-semibold shadow-lg transition hover:shadow-xl"
      style="background-color: var(--btn-bg); color: var(--btn-text);"
    >
      New Game
    </button>
    <button
      onclick={onStartProblems}
      class="rounded-lg px-8 py-4 text-lg font-semibold shadow-lg transition hover:shadow-xl"
      style="background-color: var(--accent-secondary); color: var(--btn-text);"
    >
      Practice Problems
    </button>
    <div class="flex gap-2">
      <button
        onclick={onShowDashboard}
        class="rounded-lg px-6 py-2 text-sm font-semibold transition"
        style="background-color: var(--panel-bg); color: var(--text-primary);"
      >
        Progress
      </button>
      <button
        onclick={() => (showSettingsDialog = true)}
        class="rounded-lg px-3 py-2 text-sm transition"
        style="background-color: var(--panel-bg); color: var(--text-primary);"
        title="Settings"
      >
        ⚙
      </button>
    </div>
  </div>

  {#if downloadStore.anyDownloading}
    <div class="w-full max-w-md rounded p-3 text-sm" style="background-color: color-mix(in srgb, var(--info) 10%, transparent); color: var(--text-secondary);">
      {#if downloadStore.katagoDownloading}
        <div class="flex items-center gap-2">
          <span class="inline-block h-3 w-3 animate-spin rounded-full border-2 border-t-transparent" style="border-color: var(--info); border-top-color: transparent;"></span>
          Downloading KataGo... {Math.round(downloadStore.katagoProgress)}%
        </div>
      {:else if downloadStore.llmDownloading}
        <div class="flex items-center gap-2">
          <span class="inline-block h-3 w-3 animate-spin rounded-full border-2 border-t-transparent" style="border-color: var(--info); border-top-color: transparent;"></span>
          Downloading AI coach... {Math.round(downloadStore.llmProgress)}%
        </div>
      {/if}
      <div class="mt-1 h-1.5 w-full overflow-hidden rounded" style="background-color: var(--surface-secondary);">
        <div class="h-full rounded transition-all duration-300" style="width: {downloadStore.katagoDownloading ? downloadStore.katagoProgress : downloadStore.llmProgress}%; background-color: var(--accent-primary);"></div>
      </div>
    </div>
  {/if}

  {#if downloadStore.katagoError || downloadStore.llmError}
    <div class="w-full max-w-md rounded p-3 text-sm" style="background-color: color-mix(in srgb, var(--danger) 10%, transparent); color: var(--danger);">
      {#if downloadStore.katagoError}
        <div>KataGo download failed: {downloadStore.katagoError}</div>
      {/if}
      {#if downloadStore.llmError}
        <div>AI coach download failed: {downloadStore.llmError}</div>
      {/if}
      <button
        onclick={() => downloadStore.retry()}
        class="mt-2 rounded px-3 py-1 text-xs font-semibold"
        style="background-color: var(--btn-bg); color: var(--btn-text);"
      >
        Retry
      </button>
    </div>
  {/if}

  <p class="max-w-md text-center text-sm" style="color: var(--text-dim);">
    Choose your board size and color, then let GoSensei guide you.
  </p>

  {#if recentGames.length > 0}
    <div class="w-full max-w-md">
      <h2 class="mb-2 text-sm font-semibold" style="color: var(--text-secondary);">Recent Games</h2>
      <div class="flex flex-col gap-1">
        {#each recentGames.slice(0, 10) as game}
          <div class="flex items-center gap-1 rounded px-3 py-2 text-sm" style="background-color: var(--panel-bg); color: var(--text-primary);">
            <button
              onclick={() => onLoadGame(game.id)}
              class="flex flex-1 items-center justify-between text-left"
            >
              <span>{game.board_size}x{game.board_size}</span>
              <span style="color: var(--text-secondary);">{formatResult(game.result)}</span>
              <span class="text-xs" style="color: var(--text-dim);">{game.played_at.slice(0, 10)}</span>
            </button>
            <button
              onclick={() => onStartReview(game.id)}
              class="rounded px-2 py-0.5 text-xs"
              style="background-color: var(--surface-primary); color: var(--accent-primary);"
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

  {#if showSettingsDialog}
    <SettingsDialog
      settings={settingsStore.value}
      onClose={() => (showSettingsDialog = false)}
      onSave={handleSettingsSave}
    />
  {/if}
</div>
