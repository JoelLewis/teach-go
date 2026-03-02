import { invoke } from "@tauri-apps/api/core";
import type {
  CoachingMessage,
  GameResult,
  GameState,
  HintData,
  ProblemState,
  ProblemStats,
  ProblemSummary,
  ReviewData,
  ReviewProgress,
  SavedGame,
  Settings,
  SkillProfile,
  SolveMoveResult,
} from "./types";

export async function newGame(
  boardSize: number,
  komi?: number,
  playerColor?: "black" | "white",
): Promise<GameState> {
  return invoke("new_game", { boardSize, komi, playerColor });
}

export async function playMove(row: number, col: number): Promise<GameState> {
  return invoke("play_move", { row, col });
}

export async function passTurn(): Promise<GameState> {
  return invoke("pass_turn");
}

export async function resign(): Promise<[GameState, GameResult]> {
  return invoke("resign");
}

export async function undoMove(): Promise<GameState> {
  return invoke("undo_move");
}

export async function startEngine(): Promise<string> {
  return invoke("start_engine");
}

export async function stopEngine(): Promise<void> {
  return invoke("stop_engine");
}

export async function requestAiMove(): Promise<GameState> {
  return invoke("request_ai_move");
}

export async function getCoachingFeedback(): Promise<CoachingMessage | null> {
  return invoke("get_coaching_feedback");
}

export async function saveGameSgf(): Promise<string | null> {
  return invoke("save_game_sgf");
}

export async function loadGameSgf(): Promise<GameState | null> {
  return invoke("load_game_sgf");
}

export async function listGames(): Promise<SavedGame[]> {
  return invoke("list_games");
}

export async function loadSavedGame(gameId: number): Promise<GameState> {
  return invoke("load_saved_game", { gameId });
}

export async function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  return invoke("update_settings", { settings });
}

export async function startReview(gameId?: number): Promise<void> {
  return invoke("start_review", { gameId: gameId ?? null });
}

export async function getReviewProgress(): Promise<ReviewProgress> {
  return invoke("get_review_progress");
}

export async function getReviewData(): Promise<ReviewData> {
  return invoke("get_review_data");
}

export async function getReviewPosition(moveNumber: number): Promise<GameState> {
  return invoke("get_review_position", { moveNumber });
}

export async function getOwnershipAt(moveNumber: number): Promise<number[] | null> {
  return invoke("get_ownership_at", { moveNumber });
}

export async function getSkillProfile(): Promise<SkillProfile> {
  return invoke("get_skill_profile");
}

// --- Problem Training ---

export async function listProblems(
  category?: string,
  limit?: number,
): Promise<ProblemSummary[]> {
  return invoke("list_problems", { category: category ?? null, limit: limit ?? null });
}

export async function startProblem(problemId: number): Promise<ProblemState> {
  return invoke("start_problem", { problemId });
}

export async function solveMove(row: number, col: number): Promise<SolveMoveResult> {
  return invoke("solve_move", { row, col });
}

export async function getHint(level: string): Promise<HintData> {
  return invoke("get_hint", { level });
}

export async function skipProblem(): Promise<void> {
  return invoke("skip_problem");
}

export async function getProblemState(): Promise<ProblemState | null> {
  return invoke("get_problem_state");
}

export async function getRecommendedProblem(): Promise<ProblemState> {
  return invoke("get_recommended_problem");
}

export async function getProblemStats(): Promise<ProblemStats> {
  return invoke("get_problem_stats");
}

export async function generateProblemsFromGame(threshold?: number): Promise<number> {
  return invoke("generate_problems_from_game", { threshold: threshold ?? null });
}
