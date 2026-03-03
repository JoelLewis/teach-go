<script lang="ts">
  import { onMount } from "svelte";
  import BoardCanvas from "../lib/board/BoardCanvas.svelte";
  import { problemStore } from "../lib/stores/problem.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { boardThemeForName } from "../lib/board/themes";
  import { play as playSound } from "../lib/audio/sounds";
  import type { Highlight } from "../lib/board/renderer";
  import * as api from "../lib/api/commands";
  import type { StoneColor, ProblemSummary } from "../lib/api/types";

  type Props = {
    onGoHome: () => void;
  };

  let { onGoHome }: Props = $props();

  let showList = $state(true);
  let categoryFilter = $state<string | null>(null);
  let sourceFilter = $state<"all" | "generated">("all");
  let importing = $state(false);
  let importMessage = $state<string | null>(null);

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
      problemStore.clear();
    };
  });

  async function loadProblems() {
    try {
      const list = await api.listProblems(categoryFilter ?? undefined);
      problemStore.setProblems(list);
    } catch (e) {
      problemStore.setError(String(e));
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
          setTimeout(() => problemStore.clearFeedback(), 1500);
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
      setTimeout(() => {
        problemStore.clear();
        showList = true;
        loadProblems();
      }, 800);
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
    setTimeout(() => { importMessage = null; }, 5000);
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

  let feedbackColorClass = $derived(
    problemStore.feedbackType === "correct" ? "text-emerald-400" :
    problemStore.feedbackType === "solved" ? "text-emerald-300" :
    problemStore.feedbackType === "wrong" ? "text-red-400" :
    problemStore.feedbackType === "failed" ? "text-stone-400" :
    "text-stone-300"
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
    flashHighlight = { type: "flash", point, color };
    setTimeout(() => { flashHighlight = null; }, 500);
  }
</script>

{#if showList}
  <!-- Problem list view -->
  <div class="flex h-full flex-col items-center p-8">
    <div class="mb-6 flex w-full max-w-lg items-center justify-between">
      <h1 class="text-2xl font-bold text-stone-100">Practice Problems</h1>
      <button
        onclick={handleGoHome}
        class="text-sm text-stone-400 hover:text-stone-200"
      >
        Home
      </button>
    </div>

    <!-- Recommended + Import + Category filter -->
    <div class="mb-4 flex flex-col gap-3">
      <div class="flex gap-2">
        <button
          onclick={startRecommended}
          class="flex-1 rounded-lg bg-teal-700 px-6 py-3 text-sm font-semibold text-white transition hover:bg-teal-600"
        >
          Recommended Problem
        </button>
        <button
          onclick={handleImport}
          disabled={importing}
          class="rounded-lg bg-stone-700 px-4 py-3 text-sm text-stone-300 transition hover:bg-stone-600 disabled:opacity-50"
        >
          {importing ? "Importing..." : "Import SGF"}
        </button>
      </div>
      {#if importMessage}
        <div class="rounded bg-stone-700/50 px-3 py-2 text-xs text-stone-300">
          {importMessage}
        </div>
      {/if}
      <div class="flex flex-wrap gap-2">
        {#each CATEGORIES as cat}
          <button
            onclick={() => { categoryFilter = cat.value; loadProblems(); }}
            class="rounded-full px-3 py-1 text-xs font-medium transition {
              categoryFilter === cat.value
                ? 'bg-teal-700 text-teal-100'
                : 'bg-stone-700 text-stone-300 hover:bg-stone-600'
            }"
          >
            {cat.label}
          </button>
        {/each}
      </div>
    </div>

    {#if problemStore.error}
      <div class="rounded bg-red-900/50 p-3 text-sm text-red-200">{problemStore.error}</div>
    {:else if problemStore.problems.length === 0}
      <p class="mt-8 text-sm text-stone-500">No problems available.</p>
    {:else}
      <div class="flex w-full max-w-lg flex-col gap-1">
        {#each problemStore.problems as problem}
          <button
            onclick={() => selectProblem(problem.id)}
            class="flex items-center justify-between rounded bg-stone-800 px-4 py-3 text-left text-sm text-stone-300 transition hover:bg-stone-700"
          >
            <div class="flex flex-col gap-0.5">
              <span class="text-stone-100">{problem.prompt}</span>
              <span class="text-xs text-stone-500">
                {categoryLabel(problem.category)} · {problem.board_size}×{problem.board_size}
              </span>
            </div>
            <span class="text-xs text-stone-400">{difficultyLabel(problem.difficulty)}</span>
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
      <BoardCanvas
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
    <div class="flex w-72 flex-col gap-3 border-l border-stone-700 p-4">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-stone-400">{categoryLabel(problemStore.state.category)}</h2>
        <button
          onclick={handleGoHome}
          class="text-sm text-stone-400 hover:text-stone-200"
        >
          Home
        </button>
      </div>

      <!-- Prompt -->
      <p class="text-lg font-medium text-stone-100">{problemStore.state.prompt}</p>

      <!-- Status badge -->
      {#if problemStore.state.status === "Solved"}
        <div class="rounded bg-emerald-900/50 px-3 py-1.5 text-center text-sm font-semibold text-emerald-300">
          Solved!
        </div>
      {:else if problemStore.state.status === "Failed"}
        <div class="rounded bg-red-900/50 px-3 py-1.5 text-center text-sm font-semibold text-red-300">
          Failed
        </div>
      {/if}

      <!-- Feedback -->
      {#if problemStore.feedback}
        <p class="text-sm font-medium {feedbackColorClass}">
          {problemStore.feedback}
        </p>
      {/if}

      <!-- Hints -->
      {#if problemStore.state.status === "InProgress"}
        <div class="flex flex-col gap-1.5">
          <p class="text-xs font-semibold text-stone-500">Hints</p>
          <div class="flex gap-2">
            <button
              onclick={() => handleHint("Area")}
              class="rounded bg-stone-700 px-3 py-1.5 text-xs text-stone-300 hover:bg-stone-600"
            >
              Area
            </button>
            <button
              onclick={() => handleHint("Candidates")}
              class="rounded bg-stone-700 px-3 py-1.5 text-xs text-stone-300 hover:bg-stone-600"
            >
              Candidates
            </button>
            <button
              onclick={() => handleHint("Answer")}
              class="rounded bg-stone-700 px-3 py-1.5 text-xs text-stone-300 hover:bg-stone-600"
            >
              Answer
            </button>
          </div>
        </div>

        <!-- Hint display -->
        {#if problemStore.hintData && problemStore.hintData.type !== "None"}
          <div class="rounded bg-amber-900/30 px-3 py-2 text-xs text-amber-200">
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

        <button
          onclick={handleSkip}
          class="mt-2 rounded bg-stone-700 px-3 py-1.5 text-sm text-stone-400 hover:bg-stone-600 hover:text-stone-200"
        >
          Skip
        </button>
      {:else}
        <!-- Problem finished — show next button -->
        <button
          onclick={handleNextProblem}
          class="mt-2 rounded bg-teal-700 px-4 py-2 text-sm font-semibold text-white hover:bg-teal-600"
        >
          Next Problem
        </button>
      {/if}

      <!-- Stats -->
      <div class="mt-auto flex justify-between text-xs text-stone-500">
        <span>Attempts: {problemStore.state.attempts}</span>
        <span>Hints: {problemStore.state.hints_used}</span>
      </div>
    </div>
  </div>
{/if}
