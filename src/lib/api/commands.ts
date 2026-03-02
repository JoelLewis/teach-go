import { invoke } from "@tauri-apps/api/core";
import type {
  CoachingMessage,
  GameResult,
  GameState,
  ReviewData,
  ReviewProgress,
  SavedGame,
  Settings,
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
