export type StoneColor = "black" | "white";

export type GamePhase = "Playing" | "Finished";

export type StonePosition = {
  row: number;
  col: number;
  color: StoneColor;
};

export type GameResult =
  | { Score: { winner: StoneColor; margin: number } }
  | { Resignation: { winner: StoneColor } }
  | "Draw";

export type GameState = {
  board_size: number;
  stones: StonePosition[];
  current_color: StoneColor;
  move_number: number;
  captures_black: number;
  captures_white: number;
  phase: GamePhase;
  result: GameResult | null;
  last_move: [number, number] | null;
};

export type Severity = "Good" | "Inaccuracy" | "Mistake" | "Blunder";

export type CoachingMessage = {
  severity: Severity;
  error_class: string | null;
  message: string;
  suggested_move: string | null;
  score_loss: number;
  move_number: number;
};

export type SavedGame = {
  id: number;
  board_size: number;
  result: string;
  played_at: string;
};

export type Settings = {
  board_size: number;
  komi: number;
  show_coordinates: boolean;
  show_move_numbers: boolean;
  ai_strength: string;
  sound_enabled: boolean;
};
