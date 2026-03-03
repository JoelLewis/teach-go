# Phase 3 Design: P0 Features (Onboarding + Variation Support)

Date: 2026-03-03

## Overview

Phase 3 delivers the two remaining P0 feature gaps: **onboarding flow** for new users and **move tree / variation support** for game review. Together, these prepare GoSensei for closed beta testing.

---

## Feature A: Move Tree & Variation Support

### Approach: Dual Storage

Keep `Game` linear (zero refactor risk). Add a `VariationTree` alongside review sessions to expose alternative moves.

### Architecture

```
SGF File
  │
  ├── parse_sgf_tree() ──► SgfTreeRoot (full tree, preserved)
  │                           │
  │                           ├── Stored in ReviewSession.variation_tree
  │                           │
  │                           └── find_variations_at_move(n) ──► Vec<VariationMove>
  │
  └── Game::from_sgf() ──► Game (linear main line, unchanged)
                              │
                              └── ReviewSession.sgf (existing)
```

### Data Model

```rust
// In review.rs (new types)
pub struct VariationMove {
    pub row: u8,
    pub col: u8,
    pub color: String,         // "black" | "white"
    pub comment: Option<String>,
    pub variation_index: usize, // index into parent's children
    pub depth: u16,            // how many moves deep this branch goes
}
```

### Changes

**Backend (crates/gosensei-core):**
- New function in `sgf/tree.rs`: `find_variations_at_move(root: &SgfNode, move_num: usize) -> Vec<VariationMove>`
  - Walks the main line to move N, returns sibling nodes at that position
  - Each sibling represents an alternative move the original SGF explored
- New function: `replay_variation(root: &SgfNode, path: &[usize]) -> Vec<(Color, Move)>`
  - Given a path of child indices, returns the move sequence for that variation

**Backend (src-tauri):**
- Extend `ReviewSession` to store `variation_tree: Option<SgfNode>` (the parsed root)
- New IPC command: `get_review_variations(move_number: u16)` → `Vec<VariationMove>`
- New IPC command: `get_review_variation_line(move_number: u16, variation_index: usize)` → `Vec<MoveEntry>` + `BoardState`
  - Replays the selected variation branch and returns its moves + resulting board

**Frontend:**
- `review.svelte.ts`: add `variations` reactive state, fetch on move change
- `ReviewView.svelte`: render variation markers on board (semi-transparent stones with "V" label or distinct color)
- Click variation marker → fetch variation line → show in a "variation preview" panel (board + move list)
- Escape or "Back to main line" → dismiss preview

### Scope Boundaries
- Read-only variation viewing (no editing/creating variations)
- Only for ReviewView (not PlayView or ProblemView)
- SGF writer still outputs main line only (future: output explored variations)

### Tests
- `find_variations_at_move()`: 6 tests (no variations, one variation, multiple, nested, at start, at end)
- `replay_variation()`: 3 tests (simple branch, deep branch, invalid path)
- IPC integration: 2 tests (with/without variations in SGF)

---

## Feature B: Onboarding Flow

### Architecture

```
App.svelte mount
  │
  ├── getSettings()
  │     │
  │     └── settings.onboarding_completed?
  │           │
  │           ├── true ──► HomeView (normal)
  │           │
  │           └── false ──► OnboardingView
  │                           │
  │                           ├── Step 1: Welcome (2 screens)
  │                           ├── Step 2: Experience selector
  │                           ├── Step 3a: Tutorial (beginners)
  │                           ├── Step 3b: Calibration (experienced)
  │                           ├── Step 4: Profile results
  │                           └── Step 5: Done → HomeView
```

### Settings Schema Changes

```rust
pub struct Settings {
    // ... existing fields ...
    pub onboarding_completed: bool,  // default: false
    pub experience_level: String,    // default: "" (set during onboarding)
}
```

SQLite migration: `ALTER TABLE settings ADD COLUMN onboarding_completed INTEGER NOT NULL DEFAULT 0`
SQLite migration: `ALTER TABLE settings ADD COLUMN experience_level TEXT NOT NULL DEFAULT ''`

### Onboarding Steps

**Step 1: Welcome (2 screens)**
- Screen 1: "Welcome to GoSensei" — brief intro (Go is a 4000-year-old game, GoSensei is your personal tutor)
- Screen 2: "How GoSensei helps" — 3 bullet points (AI opponent adapts to your level, coaching explains your mistakes, practice problems build your skills)
- Navigation: "Next" button, progress dots

**Step 2: Experience Level Selector**
- 4 cards with icons:
  - "Never played Go" → routes to tutorial (Step 3a)
  - "I know the basic rules" → routes to calibration (Step 3b) at beginner strength
  - "I play casually" → routes to calibration (Step 3b) at intermediate strength
  - "I have a rank" → routes to calibration (Step 3b) at advanced strength + optional rank input
- Saved to `settings.experience_level`

**Step 3a: Tutorial (Beginners)**
- 4 interactive exercises on a 5×5 board using the existing BoardCanvas component:
  1. **Capture**: Pre-placed stones, player captures a group (1 liberty left)
  2. **Ko**: Demonstrate ko rule — place stone, see error, learn the rule
  3. **Territory**: Count territory on a simple endgame position
  4. **Life & Death**: Kill a corner group (2 moves)
- Each exercise: setup position (AB/AW stones), single correct move, success feedback
- Data: 4 hardcoded problem definitions (not in DB — just static TypeScript data)
- Uses existing `BoardCanvas` with `highlights` for hints

**Step 3b: Calibration Game**
- Standard 9×9 game against AI using the existing `new_game` + `request_ai_move` pipeline
- AI strength set based on experience level selection
- After 10-15 moves, coaching pipeline has enough data to estimate skill
- "End calibration" button becomes available after move 10
- On end: call `get_skill_profile()` to establish initial skill vector

**Step 4: Profile Results**
- Show the established skill profile (radar chart from DashboardView, reusable component)
- "Your estimated level: ~20 kyu"
- Recommend board size and difficulty: "We suggest starting with 9×9 at beginner difficulty"
- "Start playing" button

**Step 5: Completion**
- Set `onboarding_completed = true` via `update_settings()`
- Navigate to HomeView
- HomeView shows a "Suggested: Play your first full game" card (conditional on game count = 0)

### Frontend Components

| Component | Type | Purpose |
|-----------|------|---------|
| `OnboardingView.svelte` | View | Multi-step flow container with step state |
| `WelcomeStep.svelte` | Component | Welcome screens with illustrations |
| `ExperienceStep.svelte` | Component | 4-card experience selector |
| `TutorialStep.svelte` | Component | Interactive exercises with BoardCanvas |
| `CalibrationStep.svelte` | Component | Abbreviated game with early-exit |
| `ProfileStep.svelte` | Component | Skill profile display + recommendations |

### Tutorial Exercise Data

```typescript
type TutorialExercise = {
  title: string;
  instruction: string;
  boardSize: number;          // 5 for tutorials
  setupBlack: [number, number][];
  setupWhite: [number, number][];
  correctMove: [number, number];
  explanation: string;
};
```

4 exercises, hardcoded in `src/lib/onboarding/exercises.ts`. No backend needed — purely frontend logic using `Board::new()` + manual stone placement via the existing renderer.

### Backend Commands (new)

None required beyond existing commands. The onboarding flow uses:
- `get_settings()` / `update_settings()` — for onboarding_completed flag
- `new_game()` / `play_move()` / `request_ai_move()` — for calibration game
- `get_skill_profile()` — for profile results

The only backend change is the Settings struct + SQLite schema.

### Scope Boundaries
- No account creation (single-player app)
- No skip button on tutorial exercises (each takes ~10 seconds)
- Calibration game uses existing AI pipeline — no special diagnostic mode
- Tutorial exercises are frontend-only (no Go rules engine validation — just check if move matches correct answer)
- No animations in tutorial beyond existing stone placement animation

### Tests
- Backend: 2 tests (settings migration, onboarding_completed persistence)
- Frontend: manual testing (onboarding is primarily UI flow)

---

## Implementation Order

1. **Variation support first** (1-2 days) — smaller scope, backend-heavy, less UI
2. **Onboarding flow second** (3-4 days) — larger scope, mostly frontend, uses existing backend

Total estimate: 5-6 days of implementation.

---

## Files Modified

### Variation Support
- `crates/gosensei-core/src/sgf/tree.rs` — new helper functions
- `src-tauri/src/commands/review.rs` — extend ReviewSession, new IPC commands
- `src-tauri/src/lib.rs` — register new commands
- `src/lib/api/commands.ts` — new command wrappers
- `src/lib/api/types.ts` — VariationMove type
- `src/lib/stores/review.svelte.ts` — variations state
- `src/views/ReviewView.svelte` — variation markers + preview panel

### Onboarding Flow
- `src-tauri/src/commands/settings.rs` — Settings struct + migration
- `src-tauri/src/db.rs` — schema migration
- `src/lib/api/types.ts` — ExperienceLevel type
- `src/App.svelte` — onboarding gate + new view
- `src/views/OnboardingView.svelte` — new view (multi-step)
- `src/components/WelcomeStep.svelte` — new component
- `src/components/ExperienceStep.svelte` — new component
- `src/components/TutorialStep.svelte` — new component
- `src/components/CalibrationStep.svelte` — new component
- `src/components/ProfileStep.svelte` — new component
- `src/lib/onboarding/exercises.ts` — tutorial exercise data
- `src/views/HomeView.svelte` — suggested first activity card
