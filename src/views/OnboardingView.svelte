<script lang="ts">
  import BoardSvg from "../lib/board/BoardSvg.svelte";
  import { boardThemeForName } from "../lib/board/themes";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { downloadStore } from "../lib/stores/download.svelte";
  import { tutorialExercises, type TutorialExercise } from "../lib/onboarding/exercises";
  import * as api from "../lib/api/commands";
  import type { StoneColor, StonePosition, GameState } from "../lib/api/types";

  type Props = {
    onComplete: () => void;
  };

  let { onComplete }: Props = $props();

  type Step = "welcome1" | "welcome2" | "experience" | "tutorial" | "calibration" | "profile" | "done";
  let step = $state<Step>("welcome1");
  let experienceLevel = $state("");

  // Tutorial state
  let tutorialIndex = $state(0);
  let tutorialFeedback = $state<string | null>(null);

  // Calibration state
  let calibrationState = $state<GameState | null>(null);
  let calibrationMoveCount = $state(0);

  // Download state tracking for calibration
  let pendingCalibrationLevel = $state("");

  // Profile state
  let profileRank = $state(25);

  const currentExercise = $derived(tutorialExercises[tutorialIndex] ?? null);

  const tutorialStones = $derived<StonePosition[]>(
    currentExercise
      ? [
          ...currentExercise.setupBlack.map(([row, col]) => ({ row, col, color: "black" as StoneColor })),
          ...currentExercise.setupWhite.map(([row, col]) => ({ row, col, color: "white" as StoneColor })),
        ]
      : [],
  );

  function selectExperience(level: string) {
    experienceLevel = level;
    if (level === "never") {
      step = "tutorial";
    } else {
      checkSetupAndCalibrate(level);
    }
  }

  // Auto-start calibration when KataGo becomes ready
  $effect(() => {
    if (downloadStore.katagoReady && pendingCalibrationLevel && step !== "calibration") {
      startCalibration(pendingCalibrationLevel);
    }
  });

  async function checkSetupAndCalibrate(level: string) {
    await downloadStore.refresh();
    if (downloadStore.katagoReady) {
      startCalibration(level);
    } else {
      pendingCalibrationLevel = level;
      step = "calibration";
      downloadStore.startListening();
    }
  }

  function handleTutorialClick(row: number, col: number) {
    if (!currentExercise) return;
    const [cr, cc] = currentExercise.correctMove;
    if (row === cr && col === cc) {
      tutorialFeedback = currentExercise.successMessage;
      setTimeout(() => {
        tutorialFeedback = null;
        if (tutorialIndex < tutorialExercises.length - 1) {
          tutorialIndex++;
        } else {
          checkSetupAndCalibrate("never");
        }
      }, 2500);
    } else {
      tutorialFeedback = "Not quite — try again!";
      setTimeout(() => (tutorialFeedback = null), 1500);
    }
  }

  async function startCalibration(level: string) {
    step = "calibration";
    const strength =
      level === "never" || level === "rules"
        ? "beginner"
        : level === "casual"
          ? "intermediate"
          : "advanced";
    try {
      const settings = { ...settingsStore.value, ai_strength: strength };
      await api.updateSettings(settings);
      settingsStore.update(settings);
      await api.startEngine();
      calibrationState = await api.newGame(9, 6.5, "black");
      calibrationMoveCount = 0;
    } catch (e) {
      console.error("Failed to start calibration:", e);
      finishOnboarding();
    }
  }

  async function handleCalibrationMove(row: number, col: number) {
    if (!calibrationState || calibrationState.phase !== "Playing") return;
    try {
      calibrationState = await api.playMove(row, col);
      calibrationMoveCount++;
      if (calibrationState.phase === "Playing") {
        calibrationState = await api.requestAiMove();
        calibrationMoveCount++;
      }
    } catch (e) {
      console.error("Calibration move failed:", e);
    }
  }

  async function endCalibration() {
    try {
      const profile = await api.getSkillProfile();
      profileRank = Math.round(profile.overall_rank);
    } catch {
      profileRank = 25;
    }
    step = "profile";
  }

  async function finishOnboarding() {
    try {
      const settings = {
        ...settingsStore.value,
        onboarding_completed: true,
        experience_level: experienceLevel,
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
  {#if step === "welcome1"}
    <div class="flex max-w-md flex-col items-center gap-6 text-center">
      <h1 class="text-4xl font-bold" style="color: var(--accent-primary, #c9a84c);">Welcome to GoSensei</h1>
      <p class="text-lg" style="color: var(--text-secondary, #a8a29e);">
        Go is one of the oldest and deepest strategy games in the world.
        GoSensei is your personal AI tutor — here to help you learn, improve, and enjoy the game.
      </p>
      <button
        onclick={() => (step = "welcome2")}
        class="rounded-lg px-8 py-3 text-lg font-semibold transition hover:opacity-90"
        style="background-color: var(--btn-bg); color: var(--btn-text);"
      >
        Next
      </button>
      <div class="flex gap-2">
        <div class="h-2 w-8 rounded" style="background-color: var(--accent-primary);"></div>
        <div class="h-2 w-8 rounded bg-stone-600"></div>
      </div>
    </div>

  {:else if step === "welcome2"}
    <div class="flex max-w-md flex-col items-center gap-6 text-center">
      <h2 class="text-2xl font-bold" style="color: var(--text-primary);">How GoSensei Helps</h2>
      <div class="flex flex-col gap-4 text-left" style="color: var(--text-secondary);">
        <div class="flex items-start gap-3">
          <span class="mt-1 inline-block h-3 w-3 rounded-full" style="background-color: var(--accent-primary);"></span>
          <p><strong style="color: var(--text-primary);">Adaptive AI opponent</strong> that matches your level and grows with you</p>
        </div>
        <div class="flex items-start gap-3">
          <span class="mt-1 inline-block h-3 w-3 rounded-full" style="background-color: var(--accent-primary);"></span>
          <p><strong style="color: var(--text-primary);">Coaching after each move</strong> — explains mistakes and suggests better plays</p>
        </div>
        <div class="flex items-start gap-3">
          <span class="mt-1 inline-block h-3 w-3 rounded-full" style="background-color: var(--accent-primary);"></span>
          <p><strong style="color: var(--text-primary);">Practice problems</strong> that target your weaknesses with spaced repetition</p>
        </div>
      </div>
      <button
        onclick={() => (step = "experience")}
        class="rounded-lg px-8 py-3 text-lg font-semibold transition hover:opacity-90"
        style="background-color: var(--btn-bg); color: var(--btn-text);"
      >
        Let's Begin
      </button>
      <div class="flex gap-2">
        <div class="h-2 w-8 rounded bg-stone-600"></div>
        <div class="h-2 w-8 rounded" style="background-color: var(--accent-primary);"></div>
      </div>
    </div>

  {:else if step === "experience"}
    <div class="flex max-w-lg flex-col items-center gap-6 text-center">
      <h2 class="text-2xl font-bold" style="color: var(--text-primary);">What's your experience with Go?</h2>
      <div class="grid grid-cols-2 gap-3">
        {#each [
          { level: "never", label: "Never played", desc: "I'm completely new" },
          { level: "rules", label: "Know the rules", desc: "I understand the basics" },
          { level: "casual", label: "Play casually", desc: "I've played some games" },
          { level: "ranked", label: "I have a rank", desc: "I play regularly" },
        ] as option}
          <button
            onclick={() => selectExperience(option.level)}
            class="flex flex-col items-center gap-1 rounded-lg border p-4 text-center transition hover:opacity-90"
            style="border-color: var(--panel-border); background-color: var(--panel-bg);"
          >
            <span class="font-semibold" style="color: var(--text-primary);">{option.label}</span>
            <span class="text-xs" style="color: var(--text-dim);">{option.desc}</span>
          </button>
        {/each}
      </div>
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
          theme={boardThemeForName(themeStore.active)}
          onIntersectionClick={handleTutorialClick}
        />
        {#if tutorialFeedback}
          <div
            class="rounded px-4 py-2 text-sm font-medium"
            style="background-color: var(--panel-bg); color: {tutorialFeedback.startsWith('Not') ? 'var(--danger)' : 'var(--success)'};"
          >
            {tutorialFeedback}
          </div>
        {/if}
        <div class="text-xs" style="color: var(--text-dim);">
          Exercise {tutorialIndex + 1} of {tutorialExercises.length}
        </div>
      </div>
    </div>

  {:else if step === "calibration" && !downloadStore.katagoReady}
    <div class="flex max-w-md flex-col items-center gap-6 text-center">
      <h2 class="text-2xl font-bold" style="color: var(--text-primary);">Preparing Calibration</h2>
      <p class="text-sm" style="color: var(--text-secondary);">
        Downloading KataGo AI engine. This is a one-time download.
      </p>
      {#if downloadStore.katagoDownloading}
        <div class="w-full max-w-xs">
          <div class="h-2 w-full overflow-hidden rounded" style="background-color: var(--surface-secondary);">
            <div class="h-full rounded transition-all duration-300" style="width: {downloadStore.katagoProgress}%; background-color: var(--accent-primary);"></div>
          </div>
          <div class="mt-1 text-xs" style="color: var(--text-dim);">{Math.round(downloadStore.katagoProgress)}%</div>
        </div>
      {:else if downloadStore.katagoError}
        <div class="text-sm" style="color: var(--danger);">Download failed: {downloadStore.katagoError}</div>
        <div class="flex gap-2">
          <button
            onclick={() => downloadStore.retry()}
            class="rounded px-4 py-2 text-sm font-semibold"
            style="background-color: var(--btn-bg); color: var(--btn-text);"
          >
            Retry
          </button>
          <button onclick={finishOnboarding} class="rounded px-4 py-2 text-sm" style="background-color: var(--surface-secondary); color: var(--text-secondary);">
            Skip for now
          </button>
        </div>
      {:else}
        <div class="text-sm" style="color: var(--text-dim);">Starting download...</div>
      {/if}
    </div>

  {:else if step === "calibration" && calibrationState}
    <div class="flex gap-6">
      <div class="flex flex-col items-center gap-3">
        <h2 class="text-xl font-bold" style="color: var(--text-primary);">Calibration Game</h2>
        <p class="max-w-xs text-sm" style="color: var(--text-secondary);">
          Play a few moves so GoSensei can estimate your level. No pressure — just play naturally!
        </p>
        <BoardSvg
          boardSize={calibrationState.board_size}
          stones={calibrationState.stones}
          currentColor={calibrationState.current_color}
          lastMove={calibrationState.last_move}
          animate
          theme={boardThemeForName(themeStore.active)}
          onIntersectionClick={handleCalibrationMove}
        />
        <div class="flex items-center gap-3">
          <span class="text-xs" style="color: var(--text-dim);">Move {calibrationMoveCount}</span>
          {#if calibrationMoveCount >= 10}
            <button
              onclick={endCalibration}
              class="rounded px-4 py-2 text-sm font-semibold transition hover:opacity-90"
              style="background-color: var(--btn-bg); color: var(--btn-text);"
            >
              That's enough — show my profile
            </button>
          {:else}
            <span class="text-xs" style="color: var(--text-dim);">(play at least 10 moves)</span>
          {/if}
        </div>
      </div>
    </div>

  {:else if step === "profile"}
    <div class="flex max-w-md flex-col items-center gap-6 text-center">
      <h2 class="text-2xl font-bold" style="color: var(--text-primary);">Your Starting Profile</h2>
      <div class="rounded-lg border p-6" style="border-color: var(--panel-border); background-color: var(--panel-bg);">
        <div class="text-4xl font-bold" style="color: var(--accent-primary);">{profileRank} kyu</div>
        <p class="mt-1 text-sm" style="color: var(--text-dim);">Estimated starting level</p>
      </div>
      <p class="text-sm" style="color: var(--text-secondary);">
        We recommend starting with <strong style="color: var(--text-primary);">9x9 games</strong> at
        <strong style="color: var(--text-primary);">{experienceLevel === "ranked" ? "advanced" : "beginner"} difficulty</strong>.
        GoSensei will adjust as you improve.
      </p>
      <button
        onclick={finishOnboarding}
        class="rounded-lg px-8 py-3 text-lg font-semibold transition hover:opacity-90"
        style="background-color: var(--btn-bg); color: var(--btn-text);"
      >
        Start Playing
      </button>
    </div>
  {/if}
</div>

