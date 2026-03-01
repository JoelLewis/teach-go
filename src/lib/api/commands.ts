import { invoke } from "@tauri-apps/api/core";
import type { GameResult, GameState, Settings } from "./types";

export async function newGame(
  boardSize: number,
  komi?: number,
): Promise<GameState> {
  return invoke("new_game", { boardSize, komi });
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

export async function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export async function updateSettings(settings: Settings): Promise<Settings> {
  return invoke("update_settings", { settings });
}
