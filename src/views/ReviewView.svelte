<script lang="ts">
  import { onMount } from "svelte";
  import BoardCanvas from "../lib/board/BoardCanvas.svelte";
  import WinRateChart from "../components/WinRateChart.svelte";
  import ReviewControls from "../components/ReviewControls.svelte";
  import ReviewMovePanel from "../components/ReviewMovePanel.svelte";
  import { reviewStore } from "../lib/stores/review.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { onReviewProgress } from "../lib/api/events";
  import * as api from "../lib/api/commands";
  import type { GameState, StoneColor } from "../lib/api/types";

  type Props = {
    gameId?: number;
    onGoHome: () => void;
  };

  let { gameId, onGoHome }: Props = $props();

  let boardState = $state<GameState | null>(null);
  let error = $state<string | null>(null);
  let unlisteners: Array<() => void> = [];

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

  function handleMoveSelect(move: number) {
    reviewStore.goToMove(move);
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

      <!-- Move annotation panel -->
      <ReviewMovePanel analysis={reviewStore.currentAnalysis} />

      <!-- Summary stats -->
      {#if reviewStore.data.top_mistakes.length > 0}
        <div class="rounded bg-stone-800 p-2 text-xs text-stone-400">
          Top mistakes at moves: {reviewStore.data.top_mistakes.join(", ")}
        </div>
      {/if}
    {:else if !isAnalyzing && !error}
      <div class="text-sm text-stone-500">Starting analysis...</div>
    {/if}
  </div>
</div>
