<script lang="ts">
  import BoardSvg, { type Highlight } from "../lib/board/BoardSvg.svelte";
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
  let tutorialBusy = $state(false);
  let tutorialStones = $state<StonePosition[]>([]);

  // Calibration state
  let calibrationState = $state<GameState | null>(null);
  let calibrationMoveCount = $state(0);
  let calibrationBusy = $state(false);
  let calibrationError = $state<string | null>(null);
  let debugLog = $state<string[]>([]);

  // Download state tracking for calibration
  let pendingCalibrationLevel = $state("");

  // Profile state
  let profileRank = $state(25);

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
    if (downloadStore.katagoReady && pendingCalibrationLevel && !calibrationState) {
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
      await downloadStore.startListening();
    }
  }

  function handleTutorialClick(row: number, col: number) {
    if (!currentExercise || tutorialBusy) return;
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
          checkSetupAndCalibrate("never");
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
      checkSetupAndCalibrate("never");
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
      dbg(`starting: strength=${strength}`);
      const settings = { ...settingsStore.value, ai_strength: strength };
      await api.updateSettings(settings);
      settingsStore.update(settings);

      // Engine start is best-effort — calibration works without AI
      try {
        dbg("starting engine...");
        await api.startEngine();
        dbg("engine ready");
      } catch (engineErr) {
        dbg(`engine failed: ${engineErr}`);
        calibrationError = "AI engine unavailable — you can still place stones to calibrate.";
      }

      calibrationState = await api.newGame(9, 6.5, "black");
      dbg(`game created: phase=${calibrationState?.phase} color=${calibrationState?.current_color}`);
      calibrationMoveCount = 0;
    } catch (e) {
      dbg(`FATAL: ${e}`);
      finishOnboarding();
    }
  }

  function dbg(msg: string) {
    debugLog = [...debugLog.slice(-9), msg];
    console.log("[CALIBRATION]", msg);
  }

  async function handleCalibrationMove(row: number, col: number) {
    dbg(`click (${row},${col}) phase=${calibrationState?.phase} busy=${calibrationBusy}`);
    if (!calibrationState || calibrationState.phase !== "Playing" || calibrationBusy) return;
    calibrationBusy = true;
    calibrationError = null;
    try {
      dbg(`playMove(${row},${col})...`);
      calibrationState = await api.playMove(row, col);
      calibrationMoveCount++;
      dbg(`move ok, phase=${calibrationState.phase} stones=${calibrationState.stones.length}`);
      if (calibrationState.phase === "Playing") {
        try {
          dbg("requesting AI move...");
          calibrationState = await api.requestAiMove();
          dbg(`AI moved, phase=${calibrationState.phase}`);
        } catch (aiErr) {
          dbg(`AI error: ${aiErr}`);
          calibrationError = "AI couldn't respond — you can keep playing or end calibration.";
        }
      }
    } catch (e) {
      dbg(`move error: ${e}`);
      calibrationError = `Move failed: ${e}`;
    } finally {
      calibrationBusy = false;
    }
  }

  async function handleCalibrationPass() {
    if (!calibrationState || calibrationState.phase !== "Playing" || calibrationBusy) return;
    calibrationBusy = true;
    calibrationError = null;
    try {
      calibrationState = await api.passTurn();
      calibrationMoveCount++;
      if (calibrationState.phase === "Playing") {
        try {
          calibrationState = await api.requestAiMove();
        } catch (aiErr) {
          console.error("[CALIBRATION] AI move failed after pass:", aiErr);
          calibrationError = "AI couldn't respond — you can keep playing or end calibration.";
        }
      }
    } catch (e) {
      console.error("[CALIBRATION] Pass failed:", e);
      calibrationError = "Pass failed — try again.";
    } finally {
      calibrationBusy = false;
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

  function difficultyLabel(level: string): string {
    if (level === "ranked") return "advanced";
    if (level === "casual") return "intermediate";
    return "beginner";
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
        {#if calibrationError}
          <div class="rounded px-4 py-2 text-sm" style="background-color: var(--panel-bg); color: var(--danger);">
            {calibrationError}
          </div>
        {/if}
        {#if debugLog.length > 0}
          <div class="max-w-xs rounded px-3 py-2 text-left font-mono text-xs" style="background-color: #111; color: #0f0; max-height: 120px; overflow-y: auto;">
            {#each debugLog as line}
              <div>{line}</div>
            {/each}
          </div>
        {/if}
        {#if calibrationBusy}
          <div class="text-xs" style="color: var(--text-dim);">Thinking...</div>
        {/if}
        <div class="flex items-center gap-3">
          <span class="text-xs" style="color: var(--text-dim);">Your moves: {calibrationMoveCount}</span>
          <button
            onclick={handleCalibrationPass}
            disabled={calibrationBusy}
            class="rounded px-3 py-1 text-xs disabled:opacity-50"
            style="background-color: var(--surface-secondary); color: var(--text-secondary);"
          >
            Pass
          </button>
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
        <strong style="color: var(--text-primary);">{difficultyLabel(experienceLevel)} difficulty</strong>.
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
