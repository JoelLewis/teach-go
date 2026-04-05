<script lang="ts">
  import BoardSvg, { type Highlight } from "../lib/board/BoardSvg.svelte";
  import { boardThemeForName } from "../lib/board/themes";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { tutorialExercises } from "../lib/onboarding/exercises";
  import * as api from "../lib/api/commands";
  import type { StoneColor, StonePosition } from "../lib/api/types";

  type Props = {
    onComplete: () => void;
  };

  let { onComplete }: Props = $props();

  type Step = "welcome" | "tutorial" | "ready";
  let step = $state<Step>("welcome");

  // Tutorial state
  let tutorialIndex = $state(0);
  let tutorialFeedback = $state<string | null>(null);
  let tutorialBusy = $state(false);
  let tutorialStones = $state<StonePosition[]>([]);
  let tutorialAttempts = $state(0);

  const currentExercise = $derived(tutorialExercises[tutorialIndex] ?? null);

  // Reset tutorial stones when exercise changes
  $effect(() => {
    const ex = currentExercise;
    if (ex) {
      tutorialStones = [
        ...ex.setupBlack.map(([row, col]) => ({ row, col, color: "black" as StoneColor })),
        ...ex.setupWhite.map(([row, col]) => ({ row, col, color: "white" as StoneColor })),
      ];
    } else {
      tutorialStones = [];
    }
  });

  const tutorialHints = $derived<Highlight[]>(
    currentExercise && !tutorialBusy
      ? [{ type: "candidates", points: [currentExercise.correctMove] }]
      : [],
  );

  let inferredRank = $derived.by(() => {
    const avgAttempts = tutorialAttempts / Math.max(1, tutorialExercises.length);
    if (avgAttempts <= 1.2) return 15;
    if (avgAttempts <= 2.0) return 20;
    return 25;
  });

  function handleTutorialClick(row: number, col: number) {
    if (!currentExercise || tutorialBusy) return;
    tutorialAttempts++;
    const [cr, cc] = currentExercise.correctMove;
    if (row === cr && col === cc) {
      tutorialBusy = true;
      // Place the player's stone
      tutorialStones = [
        ...tutorialStones,
        { row, col, color: currentExercise.playerColor },
      ];
      // Remove captured stones
      if (currentExercise.captures.length > 0) {
        const captureSet = new Set(
          currentExercise.captures.map(([r, c]) => `${r},${c}`),
        );
        tutorialStones = tutorialStones.filter(
          (s) => !captureSet.has(`${s.row},${s.col}`),
        );
      }
      tutorialFeedback = currentExercise.successMessage;
      setTimeout(() => {
        tutorialFeedback = null;
        tutorialBusy = false;
        if (tutorialIndex < tutorialExercises.length - 1) {
          tutorialIndex++;
        } else {
          step = "ready";
        }
      }, 2500);
    } else {
      tutorialBusy = true;
      tutorialFeedback = "Not quite — try again!";
      setTimeout(() => {
        tutorialFeedback = null;
        tutorialBusy = false;
      }, 3000);
    }
  }

  function skipExercise() {
    if (tutorialBusy) return;
    if (tutorialIndex < tutorialExercises.length - 1) {
      tutorialIndex++;
    } else {
      step = "ready";
    }
  }

  async function finishOnboarding() {
    try {
      const settings = {
        ...settingsStore.value,
        onboarding_completed: true,
      };
      await api.updateSettings(settings);
      settingsStore.update(settings);
    } catch (e) {
      console.error("Failed to save onboarding:", e);
    }
    onComplete();
  }
</script>

<div class="flex h-full items-center justify-center" style="background-color: var(--surface-primary, #1c1917); color: var(--text-primary, #f5f5f4);">
  {#if step === "welcome"}
    <div class="relative flex max-w-md flex-col items-center gap-6 text-center">
      <!-- Ghosted 9x9 board grid background -->
      <svg
        class="pointer-events-none absolute inset-0"
        viewBox="0 0 200 200"
        style="opacity: 0.12;"
      >
        {#each Array(9) as _, i}
          <line
            x1={10 + i * 22.5}
            y1="10"
            x2={10 + i * 22.5}
            y2="190"
            stroke="var(--surface-board, #b8860b)"
            stroke-width="0.5"
          />
          <line
            x1="10"
            y1={10 + i * 22.5}
            x2="190"
            y2={10 + i * 22.5}
            stroke="var(--surface-board, #b8860b)"
            stroke-width="0.5"
          />
        {/each}
      </svg>

      <h1 class="text-4xl font-bold" style="color: var(--accent-primary, #c9a84c);">Welcome to GoSensei</h1>
      <p class="text-lg" style="color: var(--text-secondary, #a8a29e);">
        Go is one of the oldest and deepest strategy games. GoSensei is your personal AI tutor.
      </p>
      <div class="flex flex-col gap-4 text-left" style="color: var(--text-secondary);">
        <div class="flex items-start gap-3">
          <span class="mt-1 inline-block h-3 w-3 rounded-full" style="background-color: var(--accent-primary);"></span>
          <p><strong style="color: var(--text-primary);">Play against AI</strong> with real-time coaching</p>
        </div>
        <div class="flex items-start gap-3">
          <span class="mt-1 inline-block h-3 w-3 rounded-full" style="background-color: var(--accent-primary);"></span>
          <p><strong style="color: var(--text-primary);">Solve problems</strong> with hints</p>
        </div>
        <div class="flex items-start gap-3">
          <span class="mt-1 inline-block h-3 w-3 rounded-full" style="background-color: var(--accent-primary);"></span>
          <p><strong style="color: var(--text-primary);">Track your progress</strong></p>
        </div>
      </div>
      <button
        onclick={() => (step = "tutorial")}
        class="btn btn-primary btn-lg"
      >
        Let's Begin
      </button>
    </div>

  {:else if step === "tutorial" && currentExercise}
    <div class="flex gap-6">
      <div class="flex flex-col items-center gap-3">
        <h2 class="text-xl font-bold" style="color: var(--text-primary);">{currentExercise.title}</h2>
        <p class="max-w-xs text-sm" style="color: var(--text-secondary);">{currentExercise.instruction}</p>
        <BoardSvg
          boardSize={currentExercise.boardSize}
          stones={tutorialStones}
          currentColor={currentExercise.playerColor}
          lastMove={null}
          highlights={tutorialHints}
          theme={boardThemeForName(themeStore.active)}
          onIntersectionClick={handleTutorialClick}
        />
        {#if tutorialFeedback}
          <div
            class="max-w-xs rounded px-4 py-2 text-sm font-medium"
            style="background-color: var(--panel-bg); color: {tutorialFeedback.startsWith('Not') ? 'var(--danger)' : 'var(--success)'};"
          >
            {tutorialFeedback}
          </div>
        {/if}
        <div class="flex items-center gap-3">
          <span class="text-xs" style="color: var(--text-dim);">
            Exercise {tutorialIndex + 1} of {tutorialExercises.length}
          </span>
          <button
            onclick={skipExercise}
            disabled={tutorialBusy}
            class="text-xs underline disabled:opacity-50"
            style="color: var(--text-dim);"
          >
            Skip
          </button>
        </div>
      </div>
    </div>

  {:else if step === "ready"}
    <div class="flex max-w-md flex-col items-center gap-6 text-center">
      <h2 class="text-2xl font-bold" style="color: var(--text-primary);">You're ready!</h2>
      <p class="text-lg font-semibold" style="color: var(--accent-primary);">
        Estimated starting level: ~{inferredRank} kyu
      </p>
      <p class="text-sm" style="color: var(--text-dim);">
        This is just a starting point — GoSensei adjusts as you play.
      </p>
      <button
        onclick={finishOnboarding}
        class="btn btn-primary btn-lg"
      >
        Start Playing
      </button>
    </div>
  {/if}
</div>
