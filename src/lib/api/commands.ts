// Thin wrappers around the generated tauri-specta bindings (./bindings.ts).
// Unwraps the Result envelope so callers keep plain promise semantics:
// resolved with data, rejected with the serialized backend error.
import { commands } from "./bindings";
import type {
  AiEngine,
  CoachingMessage,
  DifficultySuggestion,
  DownloadStatus,
  GameResult,
  GameState,
  HintData,
  ImportProblemResult,
  KataGoStatus,
  ProblemState,
  ProblemStats,
  ProblemSummary,
  ReviewData,
  ReviewProgress,
  SavedGame,
  Settings,
  SkillProfile,
  SkillSnapshot,
  SolveMoveResult,
  VariationMove,
} from "./bindings";
import type { LlmStatus } from "./types";

type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: string };

async function unwrap<T>(result: Promise<CommandResult<T>>): Promise<T> {
  const r = await result;
  if (r.status === "error") throw r.error;
  return r.data;
}

export async function newGame(
  boardSize: number,
  komi?: number,
  playerColor?: "black" | "white",
): Promise<GameState> {
  return unwrap(commands.newGame(boardSize, komi ?? null, playerColor ?? null));
}

export async function playMove(row: number, col: number): Promise<GameState> {
  return unwrap(commands.playMove(row, col));
}

export async function passTurn(): Promise<GameState> {
  return unwrap(commands.passTurn());
}

export async function resign(): Promise<[GameState, GameResult]> {
  return unwrap(commands.resign());
}

export async function undoMove(): Promise<GameState> {
  return unwrap(commands.undoMove());
}

export async function checkDifficultySuggestion(): Promise<DifficultySuggestion | null> {
  return unwrap(commands.checkDifficultySuggestion());
}

export async function getGamePosition(moveNumber: number): Promise<GameState> {
  return unwrap(commands.getGamePosition(moveNumber));
}

export async function getAiEngine(): Promise<AiEngine> {
  return commands.getAiEngine();
}

export async function startEngine(): Promise<KataGoStatus> {
  return unwrap(commands.startEngine());
}

export async function stopEngine(): Promise<void> {
  await unwrap(commands.stopEngine());
}

export async function requestAiMove(): Promise<GameState> {
  return unwrap(commands.requestAiMove());
}

export async function getCoachingFeedback(): Promise<CoachingMessage | null> {
  return unwrap(commands.getCoachingFeedback());
}

export async function saveGameSgf(): Promise<string | null> {
  return unwrap(commands.saveGameSgf());
}

export async function loadGameSgf(): Promise<GameState | null> {
  return unwrap(commands.loadGameSgf());
}

export async function listGames(): Promise<SavedGame[]> {
  return unwrap(commands.listGames());
}

export async function loadSavedGame(gameId: number): Promise<GameState> {
  return unwrap(commands.loadSavedGame(gameId));
}

export async function getSettings(): Promise<Settings> {
  return unwrap(commands.getSettings());
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  return unwrap(commands.updateSettings(settings));
}

export async function startReview(gameId?: number): Promise<void> {
  await unwrap(commands.startReview(gameId ?? null));
}

export async function getReviewProgress(): Promise<ReviewProgress> {
  return unwrap(commands.getReviewProgress());
}

export async function getReviewData(): Promise<ReviewData> {
  return unwrap(commands.getReviewData());
}

export async function getReviewPosition(moveNumber: number): Promise<GameState> {
  return unwrap(commands.getReviewPosition(moveNumber));
}

export async function getOwnershipAt(moveNumber: number): Promise<number[] | null> {
  return unwrap(commands.getOwnershipAt(moveNumber));
}

export async function getReviewVariations(moveNumber: number): Promise<VariationMove[]> {
  return unwrap(commands.getReviewVariations(moveNumber));
}

export async function getSkillProfile(): Promise<SkillProfile> {
  return unwrap(commands.getSkillProfile());
}

export async function getSkillHistory(windowDays?: number): Promise<SkillSnapshot[]> {
  return unwrap(commands.getSkillHistory(windowDays ?? null));
}

// --- Problem Training ---

export async function listProblems(
  category?: string,
  limit?: number,
): Promise<ProblemSummary[]> {
  return unwrap(commands.listProblems(category ?? null, limit ?? null));
}

export async function startProblem(problemId: number): Promise<ProblemState> {
  return unwrap(commands.startProblem(problemId));
}

export async function solveMove(row: number, col: number): Promise<SolveMoveResult> {
  return unwrap(commands.solveMove(row, col));
}

export async function getHint(level: string): Promise<HintData> {
  return unwrap(commands.getHint(level));
}

export async function skipProblem(): Promise<void> {
  await unwrap(commands.skipProblem());
}

export async function getProblemState(): Promise<ProblemState | null> {
  return unwrap(commands.getProblemState());
}

export async function getRecommendedProblem(): Promise<ProblemState> {
  return unwrap(commands.getRecommendedProblem());
}

export async function getProblemStats(): Promise<ProblemStats> {
  return unwrap(commands.getProblemStats());
}

export async function generateProblemsFromGame(threshold?: number): Promise<number> {
  return unwrap(commands.generateProblemsFromGame(threshold ?? null));
}

export async function importProblemsFromSgf(): Promise<ImportProblemResult | null> {
  return unwrap(commands.importProblemsFromSgf());
}

// --- LLM Coaching ---

export async function initLlmModel(): Promise<string> {
  return unwrap(commands.initLlmModel());
}

export async function getLlmStatus(): Promise<LlmStatus> {
  return (await unwrap(commands.getLlmStatus())) as LlmStatus;
}

// --- Download Manager ---

export async function getDownloadStatus(): Promise<DownloadStatus> {
  return commands.getDownloadStatus();
}

export async function retryDownloads(): Promise<void> {
  return commands.retryDownloads();
}
