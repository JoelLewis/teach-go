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

export type MoveAnalysis = {
  move_number: number;
  color: "black" | "white" | null;
  player_move: string | null;
  winrate_black: number;
  score_lead: number;
  best_move: string | null;
  score_loss: number;
  severity: Severity;
  coaching_message: string | null;
  best_variation: string[];
};

export type ReviewData = {
  board_size: number;
  total_moves: number;
  komi: number;
  move_analyses: MoveAnalysis[];
  top_mistakes: number[];
};

export type ReviewProgress = {
  total_positions: number;
  analyzed_positions: number;
  is_complete: boolean;
};
