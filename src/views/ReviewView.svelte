<script lang="ts">
  import { onMount } from "svelte";
  import BoardCanvas from "../lib/board/BoardCanvas.svelte";
  import WinRateChart from "../components/WinRateChart.svelte";
  import ReviewControls from "../components/ReviewControls.svelte";
  import ReviewMovePanel from "../components/ReviewMovePanel.svelte";
  import { reviewStore } from "../lib/stores/review.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { boardThemeForName } from "../lib/board/themes";
  import { onReviewProgress } from "../lib/api/events";
  import * as api from "../lib/api/commands";
  import type { GameState, StoneColor } from "../lib/api/types";
  import type { Highlight } from "../lib/board/renderer";

  type Props = {
    gameId?: number;
    onGoHome: () => void;
  };

  let { gameId, onGoHome }: Props = $props();

  let boardState = $state<GameState | null>(null);
  let error = $state<string | null>(null);
  let unlisteners: Array<() => void> = [];
  let generatingProblems = $state(false);
  let generatedCount = $state<number | null>(null);

  onMount(() => {
    startReview();

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
      for (const unlisten of unlisteners) unlisten();
      window.removeEventListener("keydown", handleKeydown);
      reviewStore.clear();
    };
  });

  async function startReview() {
    try {
      // Subscribe to progress events
      const unlisten = await onReviewProgress((progress) => {
        reviewStore.setProgress(progress);
        if (progress.is_complete) {
          loadReviewData();
        }
      });
      unlisteners.push(unlisten);

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

  // Reactively fetch board position when currentMove changes
  $effect(() => {
    const move = reviewStore.currentMove;
    fetchPosition(move);
  });

  async function fetchPosition(moveNumber: number) {
    try {
      boardState = await api.getReviewPosition(moveNumber);
    } catch (e) {
      console.warn("Failed to fetch position:", e);
    }
  }

  // Reactively fetch ownership when toggle is on and move changes
  $effect(() => {
    const move = reviewStore.currentMove;
    const show = reviewStore.showOwnership;
    if (show && reviewStore.data) {
      fetchOwnership(move);
    } else {
      reviewStore.setOwnership(null);
    }
  });

  async function fetchOwnership(moveNumber: number) {
    try {
      const data = await api.getOwnershipAt(moveNumber);
      reviewStore.setOwnership(data);
    } catch (e) {
      console.warn("Failed to fetch ownership:", e);
    }
  }

  // Reactively fetch variations when move changes and review is complete
  $effect(() => {
    const move = reviewStore.currentMove;
    if (reviewStore.data) {
      fetchVariations(move);
    }
  });

  async function fetchVariations(moveNumber: number) {
    try {
      const vars = await api.getReviewVariations(moveNumber);
      reviewStore.setVariations(vars);
    } catch {
      reviewStore.setVariations([]);
    }
  }

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
      <BoardCanvas
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
