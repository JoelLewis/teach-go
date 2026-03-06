<script lang="ts">
  import { onMount } from "svelte";
  import BoardSvg from "../lib/board/BoardSvg.svelte";
  import WinRateChart from "../components/WinRateChart.svelte";
  import ReviewControls from "../components/ReviewControls.svelte";
  import ReviewMovePanel from "../components/ReviewMovePanel.svelte";
  import SetupDialog from "../components/SetupDialog.svelte";
  import { reviewStore } from "../lib/stores/review.svelte";
  import { setupStore } from "../lib/stores/setup.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { boardThemeForName } from "../lib/board/themes";
  import { onReviewProgress } from "../lib/api/events";
  import * as api from "../lib/api/commands";
  import type { GameState, StoneColor } from "../lib/api/types";
  import type { Highlight } from "../lib/board/BoardSvg.svelte";

  type Props = {
    gameId?: number;
    onGoHome: () => void;
  };

  let { gameId, onGoHome }: Props = $props();

  let boardState = $state<GameState | null>(null);
  let error = $state<string | null>(null);
  let pendingUnlisteners: Array<Promise<() => void>> = [];
  let showSetupDialog = $state(false);
  let generatingProblems = $state(false);
  let generatedCount = $state<number | null>(null);
  let moveGeneration = 0;

  onMount(() => {
    checkSetupAndReview();

    // Keyboard navigation
    function handleKeydown(e: KeyboardEvent) {
      if (e.key === "ArrowLeft") {
        e.preventDefault();
        reviewStore.prevMove();
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        reviewStore.nextMove();
      } else if (e.key === "Home") {
        e.preventDefault();
        reviewStore.goToMove(0);
      } else if (e.key === "End" && reviewStore.data) {
        e.preventDefault();
        reviewStore.goToMove(reviewStore.data.total_moves);
      }
    }

    window.addEventListener("keydown", handleKeydown);

    return () => {
      for (const p of pendingUnlisteners) {
        p.then((unlisten) => unlisten()).catch(() => {});
      }
      window.removeEventListener("keydown", handleKeydown);
      reviewStore.clear();
      setupStore.cleanup();
    };
  });

  async function checkSetupAndReview() {
    await setupStore.refresh();
    if (setupStore.status !== "ready") {
      showSetupDialog = true;
      return;
    }
    startReview();
  }

  async function startReview() {
    try {
      // Subscribe to progress events
      pendingUnlisteners.push(onReviewProgress((progress) => {
        reviewStore.setProgress(progress);
        if (progress.is_complete) {
          loadReviewData();
        }
      }));

      // Start the review analysis
      await api.startReview(gameId);

      // Load initial board position
      boardState = await api.getReviewPosition(0);
    } catch (e) {
      error = String(e);
    }
  }

  async function loadReviewData() {
    try {
      const data = await api.getReviewData();
      reviewStore.setData(data);
    } catch (e) {
      error = String(e);
    }
  }

  // Reactively fetch position, ownership, and variations when currentMove changes.
  // Uses a generation counter to discard stale responses from rapid navigation.
  $effect(() => {
    const move = reviewStore.currentMove;
    const show = reviewStore.showOwnership;
    const hasData = !!reviewStore.data;
    const gen = ++moveGeneration;

    api.getReviewPosition(move).then((state) => {
      if (gen === moveGeneration && state) boardState = state;
    }).catch((e) => {
      console.warn("Failed to fetch position:", e);
    });

    if (show && hasData) {
      api.getOwnershipAt(move).then((data) => {
        if (gen === moveGeneration) reviewStore.setOwnership(data);
      }).catch((e) => {
        console.warn("Failed to fetch ownership:", e);
      });
    } else {
      reviewStore.setOwnership(null);
    }

    if (hasData) {
      api.getReviewVariations(move).then((vars) => {
        if (gen === moveGeneration) reviewStore.setVariations(vars);
      }).catch(() => {
        if (gen === moveGeneration) reviewStore.setVariations([]);
      });
    }
  });

  function handleMoveSelect(move: number) {
    reviewStore.goToMove(move);
  }

  async function handleGenerateProblems() {
    generatingProblems = true;
    try {
      const count = await api.generateProblemsFromGame();
      generatedCount = count;
    } catch (e) {
      error = String(e);
    }
    generatingProblems = false;
  }

  function handleGoHome() {
    reviewStore.clear();
    onGoHome();
  }

  // No-op click handler for review board (read-only)
  function noop() {}

  // Derived state
  let isAnalyzing = $derived(
    reviewStore.progress !== null && !reviewStore.progress.is_complete,
  );
  let progressPercent = $derived(
    reviewStore.progress
      ? Math.round(
          (reviewStore.progress.analyzed_positions /
            reviewStore.progress.total_positions) *
            100,
        )
      : 0,
  );
  let variationHighlights: Highlight[] = $derived(
    reviewStore.variations.length > 0
      ? [{
          type: "candidates" as const,
          points: reviewStore.variations.map((v): [number, number] => [v.row, v.col]),
        }]
      : [],
  );
</script>

<div class="flex h-full">
  <!-- Board area -->
  <div class="flex flex-1 items-center justify-center p-4">
    {#if boardState}
      <BoardSvg
        boardSize={boardState.board_size}
        stones={boardState.stones}
        currentColor={boardState.current_color as StoneColor}
        lastMove={boardState.last_move}
        showCoordinates={settingsStore.value.show_coordinates}
        ownership={reviewStore.showOwnership ? reviewStore.ownership : null}
        highlights={variationHighlights}
        theme={boardThemeForName(themeStore.active)}
        onIntersectionClick={noop}
      />
    {:else}
      <div class="text-stone-500">Loading board...</div>
    {/if}
  </div>

  <!-- Right panel -->
  <div class="flex w-80 flex-col gap-3 border-l border-stone-700 p-4">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold text-stone-200">Game Review</h2>
      <button
        onclick={handleGoHome}
        class="text-sm text-stone-400 hover:text-stone-200"
      >
        Home
      </button>
    </div>

    {#if error}
      <div class="rounded bg-red-900/50 p-3 text-sm text-red-200">
        {error}
      </div>
    {/if}

    {#if isAnalyzing}
      <!-- Progress bar during analysis -->
      <div class="flex flex-col gap-2">
        <div class="text-sm text-stone-400">
          Analyzing positions... {progressPercent}%
        </div>
        <div class="h-2 w-full overflow-hidden rounded bg-stone-700">
          <div
            class="h-full rounded bg-amber-600 transition-all duration-300"
            style="width: {progressPercent}%"
          ></div>
        </div>
        {#if reviewStore.progress}
          <div class="text-xs text-stone-500">
            {reviewStore.progress.analyzed_positions} / {reviewStore.progress.total_positions} positions
          </div>
        {/if}
      </div>
    {/if}

    {#if reviewStore.data}
      <!-- Win-rate chart -->
      <WinRateChart
        analyses={reviewStore.data.move_analyses}
        currentMove={reviewStore.currentMove}
        topMistakes={reviewStore.data.top_mistakes}
        onMoveSelect={handleMoveSelect}
      />

      <!-- Navigation controls -->
      <ReviewControls
        currentMove={reviewStore.currentMove}
        totalMoves={reviewStore.data.total_moves}
        hasMistakes={reviewStore.data.top_mistakes.length > 0}
        onFirst={() => reviewStore.goToMove(0)}
        onPrev={() => reviewStore.prevMove()}
        onNext={() => reviewStore.nextMove()}
        onLast={() => reviewStore.goToMove(reviewStore.data!.total_moves)}
        onPrevMistake={() => reviewStore.prevMistake()}
        onNextMistake={() => reviewStore.nextMistake()}
      />

      <!-- Territory toggle -->
      <button
        onclick={() => reviewStore.toggleOwnership()}
        class="rounded px-3 py-1.5 text-sm font-medium {reviewStore.showOwnership
          ? 'bg-emerald-700 text-emerald-100'
          : 'bg-stone-700 text-stone-300 hover:bg-stone-600'}"
      >
        {reviewStore.showOwnership ? 'Hide' : 'Show'} Territory
      </button>

      <!-- Move annotation panel -->
      <ReviewMovePanel analysis={reviewStore.currentAnalysis} />

      <!-- Alternative moves from SGF variations -->
      {#if reviewStore.variations.length > 0}
        <div class="rounded bg-stone-800 p-3 text-sm">
          <h3 class="mb-1 text-xs font-semibold text-stone-400">Alternative Moves</h3>
          {#each reviewStore.variations as v}
            <div class="flex items-center gap-2 text-stone-300">
              <span class="font-mono text-amber-400">
                {String.fromCharCode(65 + (v.col >= 8 ? v.col + 1 : v.col))}{boardState ? boardState.board_size - v.row : ""}
              </span>
              {#if v.comment}
                <span class="truncate text-stone-400">{v.comment}</span>
              {/if}
              <span class="text-xs text-stone-500">({v.continuation_length} moves)</span>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Summary stats -->
      {#if reviewStore.data.top_mistakes.length > 0}
        <div class="rounded bg-stone-800 p-2 text-xs text-stone-400">
          Top mistakes at moves: {reviewStore.data.top_mistakes.join(", ")}
        </div>

        <!-- Generate problems from mistakes -->
        <button
          onclick={handleGenerateProblems}
          disabled={generatingProblems}
          class="rounded bg-teal-800 px-3 py-1.5 text-sm font-medium text-teal-100 transition hover:bg-teal-700 disabled:opacity-50"
        >
          {generatingProblems
            ? "Generating..."
            : generatedCount !== null
              ? `Generated ${generatedCount} problems`
              : "Generate Practice Problems"}
        </button>
      {/if}
    {:else if !isAnalyzing && !error}
      <div class="text-sm text-stone-500">Starting analysis...</div>
    {/if}
  </div>
</div>

{#if showSetupDialog}
  <SetupDialog
    onComplete={() => { showSetupDialog = false; startReview(); }}
    onSkip={() => { showSetupDialog = false; onGoHome(); }}
  />
{/if}
