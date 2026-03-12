import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  newGame,
  playMove,
  passTurn,
  startEngine,
  stopEngine,
  requestAiMove,
  getCoachingFeedback,
  undoMove,
  resign,
} from "../commands";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("commands", () => {
  it("newGame passes correct command and args", async () => {
    mockInvoke.mockResolvedValue({});
    await newGame(19, 6.5, "black");
    expect(mockInvoke).toHaveBeenCalledWith("new_game", {
      boardSize: 19,
      komi: 6.5,
      playerColor: "black",
    });
  });

  it("playMove passes row and col", async () => {
    mockInvoke.mockResolvedValue({});
    await playMove(3, 4);
    expect(mockInvoke).toHaveBeenCalledWith("play_move", { row: 3, col: 4 });
  });

  it("passTurn invokes with no args", async () => {
    mockInvoke.mockResolvedValue({});
    await passTurn();
    expect(mockInvoke).toHaveBeenCalledWith("pass_turn");
  });

  it("undoMove invokes correct command", async () => {
    mockInvoke.mockResolvedValue({});
    await undoMove();
    expect(mockInvoke).toHaveBeenCalledWith("undo_move");
  });

  it("resign invokes correct command", async () => {
    mockInvoke.mockResolvedValue([{}, {}]);
    await resign();
    expect(mockInvoke).toHaveBeenCalledWith("resign");
  });

  it("startEngine invokes correct command", async () => {
    mockInvoke.mockResolvedValue("ready");
    await startEngine();
    expect(mockInvoke).toHaveBeenCalledWith("start_engine");
  });

  it("stopEngine invokes correct command", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await stopEngine();
    expect(mockInvoke).toHaveBeenCalledWith("stop_engine");
  });

  it("requestAiMove invokes correct command", async () => {
    mockInvoke.mockResolvedValue({});
    await requestAiMove();
    expect(mockInvoke).toHaveBeenCalledWith("request_ai_move");
  });

  it("getCoachingFeedback invokes correct command", async () => {
    mockInvoke.mockResolvedValue(null);
    await getCoachingFeedback();
    expect(mockInvoke).toHaveBeenCalledWith("get_coaching_feedback");
  });
});
