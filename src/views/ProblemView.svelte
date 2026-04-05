<script lang="ts">
  import { onMount } from "svelte";
  import BoardSvg from "../lib/board/BoardSvg.svelte";
  import { problemStore } from "../lib/stores/problem.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { boardThemeForName } from "../lib/board/themes";
  import { play as playSound } from "../lib/audio/sounds";
  import type { Highlight } from "../lib/board/BoardSvg.svelte";
  import * as api from "../lib/api/commands";
  import type { StoneColor, ProblemSummary } from "../lib/api/types";

  type Props = {
    onGoHome: () => void;
  };

  let { onGoHome }: Props = $props();

  let showList = $state(true);
  let categoryFilter = $state<string | null>(null);
  let sourceFilter = $state<"all" | "generated">("all");
  let loading = $state(true);
  let importing = $state(false);
  let importMessage = $state<string | null>(null);
  let activeTimeouts: ReturnType<typeof setTimeout>[] = [];
  let flashTimeoutId: ReturnType<typeof setTimeout> | undefined;
  let importTimeoutId: ReturnType<typeof setTimeout> | undefined;

  const CATEGORIES = [
    { value: null, label: "All" },
    { value: "LifeDeath", label: "Life & Death" },
    { value: "Tesuji", label: "Tesuji" },
    { value: "Shape", label: "Shape" },
    { value: "Endgame", label: "Endgame" },
    { value: "Ko", label: "Ko" },
    { value: "CapturingRace", label: "Capturing Race" },
  ];

  onMount(() => {
    loadProblems();
    return () => {
      activeTimeouts.forEach(clearTimeout);
      problemStore.clear();
    };
  });

  async function loadProblems() {
    try {
      const list = await api.listProblems(categoryFilter ?? undefined);
      problemStore.setProblems(list);
    } catch (e) {
      problemStore.setError(String(e));
    } finally {
      loading = false;
    }
  }

  async function startRecommended() {
    problemStore.setLoading(true);
    try {
      const ps = await api.getRecommendedProblem();
      problemStore.setState(ps);
      problemStore.setFeedback(null);
      problemStore.setHint(null);
      showList = false;
    } catch (e) {
      problemStore.setError(String(e));
    }
    problemStore.setLoading(false);
  }

  async function selectProblem(id: number) {
    problemStore.setLoading(true);
    try {
      const ps = await api.startProblem(id);
      problemStore.setState(ps);
      problemStore.setFeedback(null);
      problemStore.setHint(null);
      showList = false;
    } catch (e) {
      problemStore.setError(String(e));
    }
    problemStore.setLoading(false);
  }

  async function handleSolveMove(row: number, col: number) {
    if (!problemStore.state || problemStore.state.status !== "InProgress") return;

    try {
      const result = await api.solveMove(row, col);
      problemStore.setState({
        ...problemStore.state,
        board_state: result.board_state,
        status: result.status,
        attempts: problemStore.state.attempts + 1,
      });

      if (result.move_result.type === "Correct") {
        playSound("stone");
        showFlash([row, col], "correct");
        if (result.move_result.solved) {
          problemStore.setFeedback("Solved!", "solved");
          playSound("correct");
        } else {
          problemStore.setFeedback("Correct! Continue...", "correct");
          const id = setTimeout(() => problemStore.clearFeedback(), 1500);
          activeTimeouts.push(id);
        }
      } else {
        playSound("wrong");
        showFlash([row, col], "wrong");
        problemStore.setFeedback(result.move_result.message, "wrong");
      }
    } catch (e) {
      problemStore.setError(String(e));
    }
  }

  async function handleHint(level: string) {
    try {
      const hint = await api.getHint(level);
      problemStore.setHint(hint);
      if (problemStore.state) {
        problemStore.setState({
          ...problemStore.state,
          hints_used: problemStore.state.hints_used + 1,
        });
      }
    } catch (e) {
      problemStore.setError(String(e));
    }
  }

  async function handleSkip() {
    try {
      await api.skipProblem();
      problemStore.setFeedback("Skipped", "failed");
      // Return to list after a beat
      const id = setTimeout(() => {
        problemStore.clear();
        showList = true;
        loadProblems();
      }, 800);
      activeTimeouts.push(id);
    } catch (e) {
      problemStore.setError(String(e));
    }
  }

  async function handleImport() {
    importing = true;
    importMessage = null;

    try {
      const result = await api.importProblemsFromSgf();
      if (!result) {
        // User cancelled file picker
        importing = false;
        return;
      }
      if (result.errors.length > 0) {
        importMessage = `Imported ${result.imported} problem${result.imported !== 1 ? "s" : ""}. ${result.errors.length} error${result.errors.length !== 1 ? "s" : ""}.`;
      } else {
        importMessage = `Imported ${result.imported} problem${result.imported !== 1 ? "s" : ""}.`;
      }
      loadProblems();
    } catch (e) {
      importMessage = `Import failed: ${e}`;
    }

    importing = false;
    clearTimeout(importTimeoutId);
    importTimeoutId = setTimeout(() => { importMessage = null; }, 5000);
    activeTimeouts.push(importTimeoutId);
  }

  function handleNextProblem() {
    problemStore.clear();
    showList = true;
    loadProblems();
  }

  function handleGoHome() {
    problemStore.clear();
    onGoHome();
  }

  function difficultyLabel(d: number): string {
    const r = Math.round(d);
    if (r <= 1) return "~1 dan";
    return `~${r} kyu`;
  }

  function categoryLabel(cat: string): string {
    switch (cat) {
      case "LifeDeath": return "Life & Death";
      case "CapturingRace": return "Capturing Race";
      default: return cat;
    }
  }

  let feedbackColor = $derived(
    problemStore.feedbackType === "correct" ? "var(--success)" :
    problemStore.feedbackType === "solved" ? "var(--success)" :
    problemStore.feedbackType === "wrong" ? "var(--danger)" :
    problemStore.feedbackType === "failed" ? "var(--text-secondary)" :
    "var(--text-secondary)"
  );

  let flashHighlight = $state<Highlight | null>(null);

  let highlights = $derived.by((): Highlight[] => {
    const result: Highlight[] = [];
    const hint = problemStore.hintData;
    if (hint && hint.type === "Area") {
      result.push({
        type: "area",
        minRow: hint.min_row,
        maxRow: hint.max_row,
        minCol: hint.min_col,
        maxCol: hint.max_col,
      });
    } else if (hint && hint.type === "Candidates") {
      result.push({ type: "candidates", points: hint.points });
    } else if (hint && hint.type === "Answer" && hint.point) {
      result.push({ type: "answer", point: hint.point });
    }
    if (flashHighlight) {
      result.push(flashHighlight);
    }
    return result;
  });

  function showFlash(point: [number, number], color: "correct" | "wrong") {
    clearTimeout(flashTimeoutId);
    flashHighlight = { type: "flash", point, color };
    flashTimeoutId = setTimeout(() => { flashHighlight = null; }, 500);
    activeTimeouts.push(flashTimeoutId);
  }
</script>

{#if showList}
  <!-- Problem list view -->
  <div class="flex h-full flex-col items-center overflow-y-auto p-8">
    <div class="mb-6 flex w-full max-w-lg items-center justify-between">
      <h1 class="text-2xl font-bold" style="color: var(--text-primary);">Practice Problems</h1>
      <button
        onclick={handleGoHome}
        class="text-sm transition-opacity hover:opacity-70"
        style="color: var(--text-secondary);"
      >
        Home
      </button>
    </div>

    <!-- Recommended + Import + Category filter -->
    <div class="mb-4 flex flex-col gap-3">
      <div class="flex gap-2">
        <button
          onclick={startRecommended}
          class="btn flex-1"
          style="background-color: var(--accent-secondary); color: white;"
        >
          Recommended Problem
        </button>
        <button
          onclick={handleImport}
          disabled={importing}
          class="btn btn-secondary"
        >
          {importing ? "Importing..." : "Import SGF"}
        </button>
      </div>
      {#if importMessage}
        <div class="rounded px-3 py-2 text-xs" style="background-color: var(--panel-bg); color: var(--text-secondary);">
          {importMessage}
        </div>
      {/if}
      <div class="flex flex-wrap gap-2">
        {#each CATEGORIES as cat}
          <button
            onclick={() => { categoryFilter = cat.value; loadProblems(); }}
            class="rounded-full px-3 py-1 text-xs font-medium transition-opacity hover:opacity-90"
            style="background-color: {categoryFilter === cat.value ? 'var(--accent-secondary)' : 'var(--panel-bg)'}; color: {categoryFilter === cat.value ? 'white' : 'var(--text-secondary)'};"
          >
            {cat.label}
          </button>
        {/each}
      </div>
    </div>

    {#if problemStore.error}
      <div class="rounded p-3 text-sm" style="background-color: color-mix(in srgb, var(--danger) 20%, transparent); color: var(--danger);">{problemStore.error}</div>
    {:else if loading && problemStore.problems.length === 0}
      <p class="mt-8 text-sm" style="color: var(--text-dim);">Loading problems...</p>
    {:else if problemStore.problems.length === 0}
      <p class="mt-8 text-sm" style="color: var(--text-dim);">No problems available.</p>
    {:else}
      <div class="flex w-full max-w-lg flex-col gap-1">
        {#each problemStore.problems as problem}
          <button
            onclick={() => selectProblem(problem.id)}
            class="flex items-center justify-between rounded px-4 py-3 text-left text-sm transition-opacity hover:opacity-90"
            style="background-color: var(--surface-secondary); color: var(--text-secondary);"
          >
            <div class="flex flex-col gap-0.5">
              <span style="color: var(--text-primary);">{problem.prompt}</span>
              <span class="text-xs" style="color: var(--text-dim);">
                {categoryLabel(problem.category)} · {problem.board_size}×{problem.board_size}
              </span>
            </div>
            <span class="text-xs" style="color: var(--text-secondary);">{difficultyLabel(problem.difficulty)}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
{:else if problemStore.state}
  <!-- Problem solving view -->
  <div class="flex h-full">
    <!-- Board area -->
    <div class="flex flex-1 items-center justify-center p-4">
      <BoardSvg
        boardSize={problemStore.state.board_state.board_size}
        stones={problemStore.state.board_state.stones}
        currentColor={problemStore.state.board_state.current_color as StoneColor}
        lastMove={problemStore.state.board_state.last_move}
        showCoordinates={settingsStore.value.show_coordinates}
        {highlights}
        theme={boardThemeForName(themeStore.active)}
        animate
        onIntersectionClick={handleSolveMove}
      />
    </div>

    <!-- Right panel -->
    <div class="flex w-72 flex-col gap-4 border-l p-4" style="border-color: var(--panel-border);">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold" style="color: var(--text-secondary);">{categoryLabel(problemStore.state.category)}</h2>
        <button
          onclick={handleGoHome}
          class="text-sm transition-opacity hover:opacity-70"
          style="color: var(--text-secondary);"
        >
          Home
        </button>
      </div>

      <!-- Problem info: prompt + status + feedback -->
      <div class="flex flex-col gap-2">
        <p class="text-lg font-medium" style="color: var(--text-primary);">{problemStore.state.prompt}</p>

        <!-- Status badge -->
        {#if problemStore.state.status === "Solved"}
          <div class="rounded px-3 py-1.5 text-center text-sm font-semibold" style="background-color: color-mix(in srgb, var(--success) 20%, transparent); color: var(--success);">
            Solved!
          </div>
        {:else if problemStore.state.status === "Failed"}
          <div class="rounded px-3 py-1.5 text-center text-sm font-semibold" style="background-color: color-mix(in srgb, var(--danger) 20%, transparent); color: var(--danger);">
            Failed
          </div>
        {/if}

        <!-- Feedback -->
        {#if problemStore.feedback}
          <p class="text-sm font-medium" style="color: {feedbackColor};">
            {problemStore.feedback}
          </p>
        {/if}
      </div>

      {#if problemStore.state.status === "InProgress"}
        <!-- Hints section (gap-4 from problem info via parent gap-4) -->
        <div class="flex flex-col gap-1.5">
          <p class="text-xs font-semibold" style="color: var(--text-dim);">Hints</p>
          <div class="flex gap-2">
            <button
              onclick={() => handleHint("Area")}
              class="btn btn-ghost btn-sm"
            >
              Area
            </button>
            <button
              onclick={() => handleHint("Candidates")}
              class="btn btn-ghost btn-sm"
            >
              Candidates
            </button>
            <button
              onclick={() => handleHint("Answer")}
              class="btn btn-ghost btn-sm"
            >
              Answer
            </button>
          </div>

          <!-- Hint display -->
          {#if problemStore.hintData && problemStore.hintData.type !== "None"}
            <div class="rounded px-3 py-2 text-xs" style="background-color: color-mix(in srgb, var(--accent-primary) 20%, transparent); color: var(--accent-primary);">
              {#if problemStore.hintData.type === "Area"}
                Look in rows {problemStore.hintData.min_row + 1}-{problemStore.hintData.max_row + 1},
                columns {problemStore.hintData.min_col + 1}-{problemStore.hintData.max_col + 1}
              {:else if problemStore.hintData.type === "Candidates"}
                Candidate points: {problemStore.hintData.points.map(([r, c]) => `(${r + 1},${c + 1})`).join(", ")}
              {:else if problemStore.hintData.type === "Answer"}
                {problemStore.hintData.message}
                {#if problemStore.hintData.point}
                  — ({problemStore.hintData.point[0] + 1},{problemStore.hintData.point[1] + 1})
                {/if}
              {/if}
            </div>
          {/if}
        </div>

        <!-- Skip action (gap-5 from hints: outer gap-4 + mt-1) -->
        <button
          onclick={handleSkip}
          class="btn btn-secondary btn-sm mt-1"
        >
          Skip
        </button>
      {:else}
        <!-- Problem finished — next action (gap-5 from problem info: outer gap-4 + mt-1) -->
        <button
          onclick={handleNextProblem}
          class="btn mt-1"
          style="background-color: var(--accent-secondary); color: white;"
        >
          Next Problem
        </button>
      {/if}

      <!-- Stats -->
      <div class="mt-auto flex justify-between text-xs" style="color: var(--text-dim);">
        <span>Attempts: {problemStore.state.attempts}</span>
        <span>Hints: {problemStore.state.hints_used}</span>
      </div>
    </div>
  </div>
{/if}
