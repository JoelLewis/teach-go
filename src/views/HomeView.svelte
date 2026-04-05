<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import SettingsDialog from "../components/SettingsDialog.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { downloadStore } from "../lib/stores/download.svelte";
  import * as api from "../lib/api/commands";
  import type { SavedGame, NewGameConfig, ThemeName, SkillProfile } from "../lib/api/types";

  type Props = {
    onStartGame: (config: NewGameConfig) => void;
    onLoadGame: (gameId: number) => void;
    onStartReview: (gameId: number) => void;
    onShowDashboard: () => void;
    onStartProblems: () => void;
  };

  let { onStartGame, onLoadGame, onStartReview, onShowDashboard, onStartProblems }: Props = $props();

  let showSettingsDialog = $state(false);
  let showNewGameOptions = $state(false);
  let recentGames = $state<SavedGame[]>([]);
  let skillProfile = $state<SkillProfile | null>(null);

  let ghostBoardSize = $state(settingsStore.value.board_size);
  let boardSize = $state(settingsStore.value.board_size);
  let colorChoice = $state<"black" | "white" | "auto">("black");
  let aiStrength = $state(settingsStore.value.ai_strength);

  const STAR_POINTS: Record<number, [number, number][]> = {
    9: [[2, 2], [2, 6], [4, 4], [6, 2], [6, 6]],
    13: [[3, 3], [3, 9], [6, 6], [9, 3], [9, 9]],
    19: [[3, 3], [3, 9], [3, 15], [9, 3], [9, 9], [9, 15], [15, 3], [15, 9], [15, 15]],
  };

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
      const [games, profile] = await Promise.all([
        api.listGames(),
        api.getSkillProfile(),
      ]);
      recentGames = games;
      if (profile.games_played > 0) {
        skillProfile = profile;
      }
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

  function handleStart() {
    const playerColor: "black" | "white" =
      colorChoice === "auto"
        ? (Math.random() < 0.5 ? "black" : "white")
        : colorChoice;
    showNewGameOptions = false;
    onStartGame({ boardSize, playerColor, aiStrength });
  }

  function handleBoardSizeChange(e: Event) {
    const val = Number((e.target as HTMLSelectElement).value);
    boardSize = val;
    ghostBoardSize = val;
  }
</script>

<div class="relative flex h-full flex-col items-center justify-center overflow-hidden" style="background-color: var(--surface-primary); color: var(--text-primary);">
  <!-- Ghosted board background -->
  <div class="pointer-events-none absolute inset-0 flex items-center justify-center" style="opacity: 0.12;">
    {#key ghostBoardSize}
      {@const size = ghostBoardSize}
      {@const cellSize = 30}
      {@const padding = cellSize}
      {@const totalSize = (size - 1) * cellSize + padding * 2}
      <svg
        width="{totalSize}"
        height="{totalSize}"
        viewBox="0 0 {totalSize} {totalSize}"
        style="max-height: 80vh; max-width: 80vw;"
        aria-hidden="true"
      >
        {#each Array(size) as _, i}
          <!-- Horizontal line -->
          <line
            x1={padding}
            y1={padding + i * cellSize}
            x2={padding + (size - 1) * cellSize}
            y2={padding + i * cellSize}
            stroke="var(--surface-board)"
            stroke-width="1"
          />
          <!-- Vertical line -->
          <line
            x1={padding + i * cellSize}
            y1={padding}
            x2={padding + i * cellSize}
            y2={padding + (size - 1) * cellSize}
            stroke="var(--surface-board)"
            stroke-width="1"
          />
        {/each}
        {#each STAR_POINTS[size] ?? [] as [row, col]}
          <circle
            cx={padding + col * cellSize}
            cy={padding + row * cellSize}
            r="3"
            fill="var(--surface-board)"
          />
        {/each}
      </svg>
    {/key}
  </div>

  <!-- Content layer -->
  <div class="relative z-10 flex flex-col items-center">
    <!-- Title block -->
    <div class="text-center">
      <h1 class="text-4xl font-bold" style="color: var(--accent-primary);">GoSensei</h1>
      <p class="mt-1 text-sm italic" style="color: var(--text-secondary);">Your AI Go tutor</p>
      {#if skillProfile}
        <p class="mt-1 text-sm font-semibold" style="color: var(--accent-primary);">~{Math.round(skillProfile.overall_rank)} kyu</p>
      {/if}
    </div>

    <!-- gap-10 -->
    <div class="h-10"></div>

    <!-- New Game button -->
    <button
      onclick={() => (showNewGameOptions = !showNewGameOptions)}
      class="btn btn-primary btn-lg shadow-md"
    >
      New Game
    </button>

    <!-- Inline new game options -->
    <div
      class="w-72 overflow-hidden transition-all duration-300"
      style="max-height: {showNewGameOptions ? '400px' : '0px'};"
    >
      <div class="flex flex-col gap-3 pt-4">
        <label class="block text-sm" style="color: var(--text-secondary);">Board Size
          <select
            value={boardSize}
            onchange={handleBoardSizeChange}
            class="mt-1 w-full rounded px-3 py-2"
            style="background-color: var(--surface-input); color: var(--text-heading);"
          >
            <option value={9}>9x9</option>
            <option value={13}>13x13</option>
            <option value={19}>19x19</option>
          </select>
        </label>

        <div>
          <span class="mb-1 block text-sm" style="color: var(--text-secondary);">Your Color</span>
          <div class="flex gap-2">
            {#each [
              { value: "black", label: "Black" },
              { value: "white", label: "White" },
              { value: "auto", label: "Auto" },
            ] as option}
              <button
                onclick={() => (colorChoice = option.value as "black" | "white" | "auto")}
                class="btn btn-sm flex-1 {colorChoice === option.value ? 'btn-primary' : 'btn-secondary'}"
              >
                {option.label}
              </button>
            {/each}
          </div>
        </div>

        <label class="block text-sm" style="color: var(--text-secondary);">AI Strength
          <select
            bind:value={aiStrength}
            class="mt-1 w-full rounded px-3 py-2"
            style="background-color: var(--surface-input); color: var(--text-heading);"
          >
            <option value="beginner">Beginner (25-20 kyu)</option>
            <option value="intermediate">Intermediate (19-10 kyu)</option>
            <option value="advanced">Advanced (9-1 kyu)</option>
            <option value="dan">Dan (1 dan+)</option>
          </select>
        </label>

        <div class="flex items-center gap-2">
          <button onclick={handleStart} class="btn btn-primary">
            Start
          </button>
          <button onclick={() => (showNewGameOptions = false)} class="btn btn-ghost btn-sm">
            Cancel
          </button>
        </div>
      </div>
    </div>

    <!-- gap-4 -->
    <div class="h-4"></div>

    <!-- Secondary nav -->
    <div class="flex gap-2">
      <button onclick={onStartProblems} class="btn btn-secondary btn-sm">
        Problems
      </button>
      <button onclick={onShowDashboard} class="btn btn-secondary btn-sm">
        Progress
      </button>
      <button
        onclick={() => (showSettingsDialog = true)}
        class="btn btn-ghost btn-sm"
        title="Settings"
      >
        ⚙
      </button>
    </div>

    <!-- gap-8 -->
    <div class="h-8"></div>

    <!-- Last game footnote -->
    {#if recentGames.length > 0}
      {@const game = recentGames[0]}
      <button
        onclick={() => onLoadGame(game.id)}
        class="text-xs transition-opacity hover:opacity-80"
        style="color: var(--text-dim);"
      >
        Last game: {game.board_size}&times;{game.board_size} &middot; {formatResult(game.result)} &middot; {game.played_at.slice(0, 10)}
      </button>
    {/if}

    <!-- Download progress -->
    {#if downloadStore.anyDownloading}
      <div class="mt-4 w-full max-w-md rounded p-3 text-sm" style="background-color: color-mix(in srgb, var(--info) 10%, transparent); color: var(--text-secondary);">
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
      <div class="mt-2 w-full max-w-md rounded p-3 text-sm" style="background-color: color-mix(in srgb, var(--danger) 10%, transparent); color: var(--danger);">
        {#if downloadStore.katagoError}
          <div>KataGo download failed: {downloadStore.katagoError}</div>
        {/if}
        {#if downloadStore.llmError}
          <div>AI coach download failed: {downloadStore.llmError}</div>
        {/if}
        <button
          onclick={() => downloadStore.retry()}
          class="btn btn-primary btn-sm mt-2"
        >
          Retry
        </button>
      </div>
    {/if}
  </div>

  {#if showSettingsDialog}
    <SettingsDialog
      settings={settingsStore.value}
      onClose={() => (showSettingsDialog = false)}
      onSave={handleSettingsSave}
    />
  {/if}
</div>
