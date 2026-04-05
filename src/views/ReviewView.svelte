<script lang="ts">
  import { onMount } from "svelte";
  import BoardSvg from "../lib/board/BoardSvg.svelte";
  import WinRateChart from "../components/WinRateChart.svelte";
  import ReviewControls from "../components/ReviewControls.svelte";
  import ReviewMovePanel from "../components/ReviewMovePanel.svelte";
  import { reviewStore } from "../lib/stores/review.svelte";
  import { downloadStore } from "../lib/stores/download.svelte";
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
  let generatingProblems = $state(false);
  let generatedCount = $state<number | null>(null);
  let moveGeneration = 0;

  onMount(() => {
    downloadStore.startListening();
    downloadStore.refresh();
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
      downloadStore.cleanup();
    };
  });

  // Auto-start review when KataGo becomes ready
  $effect(() => {
    if (downloadStore.katagoReady && !reviewStore.data && !reviewStore.progress && !error) {
      startReview();
    }
  });

  async function checkSetupAndReview() {
    await downloadStore.refresh();
    if (downloadStore.katagoReady) {
      startReview();
    }
    // If not ready, the $effect above will start when KataGo finishes downloading
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

<div class="flex h-full flex-col lg:flex-row">
  <!-- Board area -->
  <div class="flex flex-1 min-w-0 min-h-[50vh] lg:min-h-0 items-center justify-center p-4">
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
        interactive={false}
        onIntersectionClick={noop}
      />
    {:else}
      <div style="color: var(--text-dim);">Loading board...</div>
    {/if}
  </div>

  <!-- Right panel -->
  <div class="flex w-full lg:w-80 flex-col gap-4 border-t lg:border-t-0 lg:border-l overflow-y-auto max-h-[50vh] lg:max-h-none p-4" style="border-color: var(--panel-border);">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold" style="color: var(--text-primary);">Game Review</h2>
      <button
        onclick={handleGoHome}
        class="text-sm transition-opacity hover:opacity-70"
        style="color: var(--text-secondary);"
      >
        Home
      </button>
    </div>

    {#if !downloadStore.katagoReady}
      <div class="rounded p-3 text-sm" style="background-color: color-mix(in srgb, var(--info) 15%, transparent); color: var(--info);">
        {#if downloadStore.katagoDownloading}
          <div class="mb-1 font-semibold">Downloading KataGo{downloadStore.katagoPhase ? ` (${downloadStore.katagoPhase})` : ""}...</div>
          <div class="h-2 w-full overflow-hidden rounded" style="background-color: var(--surface-secondary);">
            <div class="h-full rounded transition-all duration-300" style="width: {downloadStore.katagoProgress}%; background-color: var(--accent-primary);"></div>
          </div>
          <div class="mt-1 text-xs" style="color: var(--text-dim);">{Math.round(downloadStore.katagoProgress)}%</div>
        {:else if downloadStore.katagoError}
          <div style="color: var(--danger);">Download failed: {downloadStore.katagoError}</div>
          <button
            onclick={() => downloadStore.retry()}
            class="btn btn-primary btn-sm mt-2"
          >
            Retry
          </button>
        {:else}
          <div>Waiting for KataGo download...</div>
        {/if}
      </div>
    {/if}

    {#if error}
      <div class="rounded p-3 text-sm" style="background: color-mix(in srgb, var(--danger) 30%, transparent); color: var(--danger);">
        {error}
      </div>
    {/if}

    {#if isAnalyzing}
      <!-- Progress bar during analysis -->
      <div class="flex flex-col gap-2">
        <div class="text-sm" style="color: var(--text-secondary);">
          Analyzing positions... {progressPercent}%
        </div>
        <div class="h-2 w-full overflow-hidden rounded" style="background: var(--surface-secondary);">
          <div
            class="h-full rounded transition-all duration-300"
            style="width: {progressPercent}%; background: var(--accent-primary);"
          ></div>
        </div>
        {#if reviewStore.progress}
          <div class="text-xs" style="color: var(--text-dim);">
            {reviewStore.progress.analyzed_positions} / {reviewStore.progress.total_positions} positions
          </div>
        {/if}
      </div>
    {/if}

    {#if reviewStore.data}
      <!-- Navigation controls + move analysis (tightly related) -->
      <div class="flex flex-col gap-2">
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

        <!-- Alternative moves from SGF variations -->
        {#if reviewStore.variations.length > 0}
          <div class="rounded p-3 text-sm" style="background: var(--panel-bg);">
            <h3 class="mb-1 text-xs font-semibold" style="color: var(--text-secondary);">Alternative Moves</h3>
            {#each reviewStore.variations as v}
              <div class="flex items-center gap-2" style="color: var(--text-secondary);">
                <span class="font-mono" style="color: var(--accent-primary);">
                  {String.fromCharCode(65 + (v.col >= 8 ? v.col + 1 : v.col))}{boardState ? boardState.board_size - v.row : ""}
                </span>
                {#if v.comment}
                  <span class="truncate" style="color: var(--text-secondary);">{v.comment}</span>
                {/if}
                <span class="text-xs" style="color: var(--text-dim);">({v.continuation_length} moves)</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Win-rate chart (different section — extra breathing room above) -->
      <div class="flex flex-col gap-4 mt-1">
        <WinRateChart
          analyses={reviewStore.data.move_analyses}
          currentMove={reviewStore.currentMove}
          topMistakes={reviewStore.data.top_mistakes}
          onMoveSelect={handleMoveSelect}
        />
      </div>

      <!-- Action buttons (territory + generate) -->
      <div class="flex flex-col gap-4">
        <!-- Territory toggle -->
        <button
          onclick={() => reviewStore.toggleOwnership()}
          class="btn btn-sm {reviewStore.showOwnership ? 'btn-primary' : 'btn-secondary'}"
        >
          {reviewStore.showOwnership ? 'Hide' : 'Show'} Territory
        </button>

        <!-- Summary stats + generate -->
        {#if reviewStore.data.top_mistakes.length > 0}
          <div class="rounded p-2 text-xs" style="background: var(--panel-bg); color: var(--text-secondary);">
            Top mistakes at moves: {reviewStore.data.top_mistakes.join(", ")}
          </div>

          <!-- Generate problems from mistakes -->
          <button
            onclick={handleGenerateProblems}
            disabled={generatingProblems}
            class="btn btn-sm"
            style="background-color: var(--accent-secondary); color: white;"
          >
            {generatingProblems
              ? "Generating..."
              : generatedCount !== null
                ? `Generated ${generatedCount} problems`
                : "Generate Practice Problems"}
          </button>
        {/if}
      </div>
    {:else if !isAnalyzing && !error}
      <div class="text-sm" style="color: var(--text-dim);">Starting analysis...</div>
    {/if}
  </div>
</div>
