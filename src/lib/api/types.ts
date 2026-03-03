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

export type MoveEntry = {
  move_number: number;
  color: StoneColor;
  row: number | null;
  col: number | null;
  is_pass: boolean;
};

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
  moves: MoveEntry[];
};

export type Severity = "Excellent" | "Good" | "Inaccuracy" | "Mistake" | "Blunder";

export type CoachingMessage = {
  severity: Severity;
  error_class: string | null;
  message: string;
  suggested_move: string | null;
  simplest_move: string | null;
  score_loss: number;
  move_number: number;
};

export type SavedGame = {
  id: number;
  board_size: number;
  result: string;
  played_at: string;
};

export type FeedbackTiming = "immediate" | "on_demand" | "post_game";

export type ThemeName = "study" | "grid";

export type Settings = {
  board_size: number;
  komi: number;
  show_coordinates: boolean;
  show_move_numbers: boolean;
  ai_strength: string;
  sound_enabled: boolean;
  feedback_timing: FeedbackTiming;
  theme: ThemeName;
  onboarding_completed: boolean;
  experience_level: string;
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

export type VariationMove = {
  row: number;
  col: number;
  color: StoneColor;
  comment: string | null;
  continuation_length: number;
};

export type DifficultySuggestion = {
  direction: "up" | "down";
  message: string;
};

export type NewGameConfig = {
  boardSize: number;
  playerColor: "black" | "white";
  aiStrength: string;
};

export type SkillDimension = {
  mu: number;
  sigma: number;
};

export type SkillProfile = {
  overall_rank: number;
  reading: SkillDimension;
  shape: SkillDimension;
  direction: SkillDimension;
  endgame: SkillDimension;
  life_death: SkillDimension;
  fighting: SkillDimension;
  games_played: number;
  last_updated: string;
};

export type SkillSnapshot = {
  recorded_at: string;
  source: string;
  overall_rank: number;
  reading_mu: number;
  shape_mu: number;
  direction_mu: number;
  endgame_mu: number;
  life_death_mu: number;
  fighting_mu: number;
};

// --- Problem Training ---

export type SolveStatus = "InProgress" | "Solved" | "Failed";

export type ProblemSummary = {
  id: number;
  category: string;
  difficulty: number;
  prompt: string;
  board_size: number;
};

export type ProblemState = {
  problem_id: number;
  board_state: GameState;
  prompt: string;
  category: string;
  status: SolveStatus;
  hints_used: number;
  attempts: number;
  elapsed_seconds: number;
};

export type MoveResultCorrect = {
  type: "Correct";
  opponent_response: [number, number] | null;
  solved: boolean;
};

export type MoveResultWrong = {
  type: "Wrong";
  message: string;
};

export type ProblemMoveResult = MoveResultCorrect | MoveResultWrong;

export type SolveMoveResult = {
  move_result: ProblemMoveResult;
  board_state: GameState;
  status: SolveStatus;
};

export type HintData =
  | { type: "None" }
  | { type: "Area"; min_row: number; max_row: number; min_col: number; max_col: number }
  | { type: "Candidates"; points: [number, number][] }
  | { type: "Answer"; point: [number, number] | null; message: string };

export type CategoryStat = {
  category: string;
  solved: number;
  attempted: number;
};

export type ProblemStats = {
  total_solved: number;
  total_attempted: number;
  accuracy_percent: number;
  per_category: CategoryStat[];
};

// --- LLM Coaching ---

export type CoachingStreamChunk = {
  move_number: number;
  text_delta: string;
  is_complete: boolean;
};

export type LlmDownloadProgress = {
  downloaded: number;
  total: number;
};

export type LlmStatus = "not_installed" | "ready" | "loading" | "disabled";
