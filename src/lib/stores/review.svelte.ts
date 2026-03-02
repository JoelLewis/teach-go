import type { MoveAnalysis, ReviewData, ReviewProgress } from "../api/types";

export const reviewStore = createReviewStore();

function createReviewStore() {
  let data = $state<ReviewData | null>(null);
  let progress = $state<ReviewProgress | null>(null);
  let currentMove = $state(0);

  return {
    get data() {
      return data;
    },
    get progress() {
      return progress;
    },
    get currentMove() {
      return currentMove;
    },
    get currentAnalysis(): MoveAnalysis | null {
      if (!data) return null;
      return data.move_analyses.find((a) => a.move_number === currentMove) ?? null;
    },
    setData(reviewData: ReviewData) {
      data = reviewData;
      currentMove = 0;
    },
    setProgress(p: ReviewProgress) {
      progress = p;
    },
    goToMove(n: number) {
      if (!data) return;
      currentMove = Math.max(0, Math.min(n, data.total_moves));
    },
    nextMove() {
      if (!data) return;
      currentMove = Math.min(currentMove + 1, data.total_moves);
    },
    prevMove() {
      currentMove = Math.max(currentMove - 1, 0);
    },
    nextMistake() {
      if (!data || data.top_mistakes.length === 0) return;
      const next = data.top_mistakes.find((m) => m > currentMove);
      if (next !== undefined) {
        currentMove = next;
      }
    },
    prevMistake() {
      if (!data || data.top_mistakes.length === 0) return;
      const prev = [...data.top_mistakes].reverse().find((m) => m < currentMove);
      if (prev !== undefined) {
        currentMove = prev;
      }
    },
    clear() {
      data = null;
      progress = null;
      currentMove = 0;
    },
  };
}
