# Phase 3: P0 Features Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add variation support to game review and an onboarding flow for new users — the two remaining P0 features before beta.

**Architecture:** Dual-storage variation model (keep `Game` linear, cache `SgfTreeRoot` in `ReviewSession`). Onboarding uses existing view-switching pattern in `App.svelte` with a new `OnboardingView` gated on a `onboarding_completed` setting.

**Tech Stack:** Rust (gosensei-core, gosensei-app), Svelte 5, Pixi.js (BoardCanvas), SQLite, Tauri v2 IPC

---

## Part 1: Variation Support in Review Mode

### Task 1: Add `find_variations_at_move` to SGF tree module

**Files:**
- Modify: `crates/gosensei-core/src/sgf/tree.rs:413-452` (add new methods to `impl SgfNode`)

**Step 1: Write the failing tests**

Add these tests at the end of the `mod tests` block in `tree.rs` (before the closing `}`):

```rust
#[test]
fn find_variations_at_move_no_branches() {
    let sgf = "(;SZ[9];B[ee];W[cc];B[dd])";
    let tree = parse_sgf_tree(sgf).unwrap();
    let vars = tree.root.variations_at_move(1);
    assert!(vars.is_empty(), "Linear game has no variations");
}

#[test]
fn find_variations_at_move_with_branch() {
    // At move 2, White has two choices: W[cc] (main) and W[dd] (variation)
    let sgf = "(;SZ[9];B[ee](;W[cc])(;W[dd]))";
    let tree = parse_sgf_tree(sgf).unwrap();
    let vars = tree.root.variations_at_move(1);
    // Should return the non-main-line sibling(s): W[dd]
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].mv, Some((Color::White, Move::Play(Point::new(3, 3)))));
}

#[test]
fn find_variations_at_move_multiple_branches() {
    let sgf = "(;SZ[9];B[ee](;W[cc])(;W[dd])(;W[ff]))";
    let tree = parse_sgf_tree(sgf).unwrap();
    let vars = tree.root.variations_at_move(1);
    assert_eq!(vars.len(), 2); // W[dd] and W[ff], excluding main line W[cc]
}

#[test]
fn find_variations_at_move_nested() {
    // Move 1: B[ee], Move 2: W[cc] (main) with variation W[dd]
    // At move 2, inside W[cc] branch: B[ff] (main) with variation B[gg]
    let sgf = "(;SZ[9];B[ee](;W[cc](;B[ff])(;B[gg]))(;W[dd]))";
    let tree = parse_sgf_tree(sgf).unwrap();

    // Variations at move 1 (after B[ee]): W[dd] is the alternative
    let vars1 = tree.root.variations_at_move(1);
    assert_eq!(vars1.len(), 1);

    // Variations at move 2 (after W[cc]): B[gg] is the alternative
    let vars2 = tree.root.variations_at_move(2);
    assert_eq!(vars2.len(), 1);
    assert_eq!(vars2[0].mv, Some((Color::Black, Move::Play(Point::new(6, 6)))));
}

#[test]
fn find_variations_at_move_zero_returns_empty() {
    let sgf = "(;SZ[9];B[ee](;W[cc])(;W[dd]))";
    let tree = parse_sgf_tree(sgf).unwrap();
    let vars = tree.root.variations_at_move(0);
    assert!(vars.is_empty());
}

#[test]
fn find_variations_at_move_beyond_end() {
    let sgf = "(;SZ[9];B[ee];W[cc])";
    let tree = parse_sgf_tree(sgf).unwrap();
    let vars = tree.root.variations_at_move(99);
    assert!(vars.is_empty());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p gosensei-core -- sgf::tree::tests::find_variations 2>&1 | tail -20`
Expected: FAIL — `no method named variations_at_move found`

**Step 3: Implement `variations_at_move`**

Add this method to the existing `impl SgfNode` block in `tree.rs` (after the `depth` method, around line 451):

```rust
/// Walk the main line to `move_num` and return the non-main-line sibling
/// nodes at that position. These represent alternative moves the SGF explores.
/// Move numbering: move 1 = first move (child of root), move 2 = second, etc.
pub fn variations_at_move(&self, move_num: usize) -> Vec<&SgfNode> {
    if move_num == 0 {
        return Vec::new();
    }

    // Walk the main line (first child at each level) counting moves
    let mut current = self;
    let mut moves_seen = 0;

    loop {
        // If this node has a move, count it
        if current.mv.is_some() {
            moves_seen += 1;
        }

        if moves_seen == move_num {
            // We're at the target move's node.
            // The variations are this node's siblings — but we only have
            // the parent's children list. We need to return the parent's
            // non-first children. However, we've already descended into
            // the first child. We need a different traversal.
            break;
        }

        // Descend main line
        if let Some(first_child) = current.children.first() {
            current = first_child;
        } else {
            return Vec::new(); // Past the end
        }
    }

    Vec::new() // Placeholder — see corrected version below
}
```

Actually, the traversal needs parent context. Let me use a different approach — walk from root, tracking parent at each step:

```rust
/// Walk the main line to `move_num` and return the non-main-line sibling
/// nodes at that position (i.e., alternative moves explored by the SGF).
/// Move 1 = first played move. Returns empty vec if no variations exist.
pub fn variations_at_move(&self, move_num: usize) -> Vec<&SgfNode> {
    if move_num == 0 {
        return Vec::new();
    }

    let mut current = self;
    let mut moves_seen = 0;

    loop {
        if current.mv.is_some() {
            moves_seen += 1;
            if moves_seen == move_num {
                // current IS the main-line node at move_num.
                // Its siblings are the other children of its parent.
                // But we don't have a parent pointer — we need to
                // check the parent (which called us) from the level above.
                // Restructure: track parent's children list instead.
                break;
            }
        }

        if let Some(first_child) = current.children.first() {
            current = first_child;
        } else {
            return Vec::new();
        }
    }

    Vec::new()
}
```

The clean implementation (walk and track the parent's children at each move boundary):

```rust
pub fn variations_at_move(&self, move_num: usize) -> Vec<&SgfNode> {
    if move_num == 0 {
        return Vec::new();
    }

    let mut current = self;
    let mut moves_seen = 0;

    loop {
        // Check if current has a move
        if current.mv.is_some() {
            moves_seen += 1;
        }

        if current.children.is_empty() {
            return Vec::new();
        }

        // Before descending, check: does the NEXT level have the target move?
        let next = &current.children[0]; // main line child
        let next_is_move = next.mv.is_some();
        if next_is_move && moves_seen + 1 == move_num {
            // The children of `current` represent the choices at move_num.
            // children[0] is the main line; children[1..] are variations.
            return current.children.iter().skip(1).collect();
        }

        current = next;
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p gosensei-core -- sgf::tree::tests::find_variations -v`
Expected: All 6 new tests PASS

**Step 5: Commit**

```bash
git add crates/gosensei-core/src/sgf/tree.rs
git commit -m "feat(core): add variations_at_move to SgfNode for review variation support"
```

---

### Task 2: Add `replay_variation` helper

**Files:**
- Modify: `crates/gosensei-core/src/sgf/tree.rs` (add method + tests)

**Step 1: Write the failing test**

```rust
#[test]
fn replay_variation_branch() {
    // B[ee] then (W[cc];B[dd]) or (W[ff];B[gg])
    let sgf = "(;SZ[9];B[ee](;W[cc];B[dd])(;W[ff];B[gg]))";
    let tree = parse_sgf_tree(sgf).unwrap();
    // Get variation at move 1 (after B[ee]): should be W[ff]
    let vars = tree.root.variations_at_move(1);
    assert_eq!(vars.len(), 1);
    // Replay that variation's continuation
    let line = vars[0].main_line_moves();
    assert_eq!(line.len(), 2); // W[ff], B[gg]
    assert_eq!(line[0], (Color::White, Move::Play(Point::new(5, 5))));
    assert_eq!(line[1], (Color::Black, Move::Play(Point::new(6, 6))));
}
```

**Step 2: Run test to verify it passes** (it should already pass — `main_line_moves()` already exists on `SgfNode`)

Run: `cargo test -p gosensei-core -- sgf::tree::tests::replay_variation -v`
Expected: PASS — `main_line_moves()` already walks the first-child chain from any node

**Step 3: Commit** (this test documents the pattern; no new code needed)

```bash
git add crates/gosensei-core/src/sgf/tree.rs
git commit -m "test(core): add replay_variation_branch test documenting variation replay pattern"
```

---

### Task 3: Extend `ReviewSession` to store the variation tree

**Files:**
- Modify: `src-tauri/src/review.rs:36-46` (add field to `ReviewSession`)
- Modify: `src-tauri/src/commands/review.rs:48-69` (store tree in session)

**Step 1: Add `variation_tree` field to `ReviewSession`**

In `src-tauri/src/review.rs`, add the import and field:

```rust
// At top of file, add import:
use gosensei_core::sgf::tree::SgfNode;

// In ReviewSession struct, add after `is_complete`:
    /// Parsed SGF variation tree (for showing alternative moves in review).
    pub variation_tree: Option<SgfNode>,
```

**Step 2: Store the tree during `start_review`**

In `src-tauri/src/commands/review.rs`, after `let game = Game::from_sgf(&sgf)` (line 49), also parse the tree:

```rust
// After line 49, add:
use gosensei_core::sgf::tree::parse_sgf_tree;

let variation_tree = parse_sgf_tree(&sgf).ok().map(|t| t.root);
```

Then in the `ReviewSession` initialization (line 60-68), add the field:

```rust
*review = Some(ReviewSession {
    game_sgf: sgf,
    board_size,
    komi,
    total_positions,
    results: vec![None; total_positions as usize],
    ownership: vec![None; total_positions as usize],
    is_complete: false,
    variation_tree,
});
```

**Step 3: Run existing tests**

Run: `cargo test -p gosensei-app -- review::tests -v`
Expected: compilation error — need to add `variation_tree` to test's `ReviewSession` too

**Step 4: Fix test compilation — add `variation_tree: None` to test's `ReviewSession`**

In `src-tauri/src/commands/review.rs`, find the `score_loss_computation` test (line ~435) and add `variation_tree: None` to the `ReviewSession`:

```rust
let mut session = ReviewSession {
    // ... existing fields ...
    is_complete: false,
    variation_tree: None,  // ADD THIS
};
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p gosensei-app -- review -v`
Expected: PASS

**Step 6: Commit**

```bash
git add src-tauri/src/review.rs src-tauri/src/commands/review.rs
git commit -m "feat(app): store SGF variation tree in ReviewSession"
```

---

### Task 4: Add `get_review_variations` IPC command

**Files:**
- Modify: `src-tauri/src/commands/review.rs` (new command)
- Modify: `src-tauri/src/lib.rs:33-69` (register command)

**Step 1: Define the `VariationMove` response type**

In `src-tauri/src/review.rs`, add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariationMove {
    pub row: u8,
    pub col: u8,
    pub color: String,
    pub comment: Option<String>,
    /// Number of moves in this variation's continuation
    pub continuation_length: usize,
}
```

**Step 2: Add the IPC command**

In `src-tauri/src/commands/review.rs`, add:

```rust
#[tauri::command]
pub async fn get_review_variations(
    state: State<'_, AppState>,
    move_number: u16,
) -> Result<Vec<crate::review::VariationMove>, AppError> {
    use gosensei_core::types::Move;

    let review = state.review.lock().await;
    let session = review
        .as_ref()
        .ok_or(AppError::Other("No review in progress".into()))?;

    let tree = match &session.variation_tree {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let alt_nodes = tree.variations_at_move(move_number as usize);
    let mut variations = Vec::new();

    for node in alt_nodes {
        if let Some((color, Move::Play(point))) = node.mv {
            variations.push(crate::review::VariationMove {
                row: point.row,
                col: point.col,
                color: color.as_str().to_string(),
                comment: node.comment.clone(),
                continuation_length: node.main_line_moves().len(),
            });
        }
    }

    Ok(variations)
}
```

**Step 3: Register the command in `lib.rs`**

In `src-tauri/src/lib.rs`, add after line 55 (`commands::review::get_ownership_at,`):

```rust
            commands::review::get_review_variations,
```

**Step 4: Run tests**

Run: `cargo test -p gosensei-app -v 2>&1 | tail -5`
Expected: PASS (compilation + existing tests)

**Step 5: Commit**

```bash
git add src-tauri/src/review.rs src-tauri/src/commands/review.rs src-tauri/src/lib.rs
git commit -m "feat(app): add get_review_variations IPC command"
```

---

### Task 5: Add frontend types and API wrappers for variations

**Files:**
- Modify: `src/lib/api/types.ts` (add `VariationMove` type)
- Modify: `src/lib/api/commands.ts` (add wrapper function)

**Step 1: Add `VariationMove` type**

In `src/lib/api/types.ts`, add after the `ReviewProgress` type (line ~96):

```typescript
export type VariationMove = {
  row: number;
  col: number;
  color: StoneColor;
  comment: string | null;
  continuation_length: number;
};
```

**Step 2: Add the API wrapper**

In `src/lib/api/commands.ts`, add after `getOwnershipAt` (line ~110):

```typescript
export async function getReviewVariations(moveNumber: number): Promise<VariationMove[]> {
  return invoke("get_review_variations", { moveNumber });
}
```

Add `VariationMove` to the import list at the top of `commands.ts`.

**Step 3: Run type check**

Run: `npx svelte-check 2>&1 | tail -3`
Expected: 0 ERRORS

**Step 4: Commit**

```bash
git add src/lib/api/types.ts src/lib/api/commands.ts
git commit -m "feat(frontend): add VariationMove type and getReviewVariations API wrapper"
```

---

### Task 6: Show variations in ReviewView

**Files:**
- Modify: `src/lib/stores/review.svelte.ts` (add `variations` state)
- Modify: `src/views/ReviewView.svelte` (fetch + display variations)

**Step 1: Add `variations` to the review store**

In `src/lib/stores/review.svelte.ts`, add to the store:

```typescript
// After line 10 (let showOwnership = ...):
let variations = $state<import("../api/types").VariationMove[]>([]);

// Add getter:
get variations() {
  return variations;
},

// Add setter:
setVariations(v: import("../api/types").VariationMove[]) {
  variations = v;
},

// In clear():
variations = [];
```

**Step 2: Fetch variations on move change in ReviewView**

In `src/views/ReviewView.svelte`, in the `$effect` that reacts to `reviewStore.currentMove` (the one that calls `getReviewPosition`), add after the board state update:

```typescript
// After updating boardState:
try {
  const vars = await api.getReviewVariations(reviewStore.currentMove);
  reviewStore.setVariations(vars);
} catch {
  reviewStore.setVariations([]);
}
```

**Step 3: Display variation markers on the board**

In `ReviewView.svelte`, compute `highlights` from variations:

```typescript
let variationHighlights = $derived(
  reviewStore.variations.map((v) => ({
    type: "candidates" as const,
    points: [[v.row, v.col]] as [number, number][],
  }))
);
```

Then pass these highlights to `BoardCanvas`:

```svelte
<BoardCanvas
  ...existing props...
  highlights={variationHighlights.length > 0 ? variationHighlights : []}
  onIntersectionClick={handleVariationClick}
/>
```

**Step 4: Add variation click handler**

```typescript
function handleVariationClick(row: number, col: number) {
  const match = reviewStore.variations.find((v) => v.row === row && v.col === col);
  if (match) {
    // TODO: In a future task, this could open a variation explorer panel.
    // For now, show the variation's comment as a tooltip/message.
    console.log("Variation clicked:", match);
  }
}
```

**Step 5: Show variation info in ReviewMovePanel**

Below the `ReviewMovePanel` in `ReviewView.svelte`, add a variations section:

```svelte
{#if reviewStore.variations.length > 0}
  <div class="rounded bg-stone-800 p-3 text-sm">
    <h3 class="mb-1 text-xs font-semibold text-stone-400">Alternative Moves</h3>
    {#each reviewStore.variations as v}
      <div class="flex items-center gap-2 text-stone-300">
        <span class="font-mono text-amber-400">
          {String.fromCharCode(65 + (v.col >= 8 ? v.col + 1 : v.col))}{boardState?.board_size ? boardState.board_size - v.row : v.row}
        </span>
        {#if v.comment}
          <span class="text-stone-400">{v.comment}</span>
        {/if}
        <span class="text-xs text-stone-500">({v.continuation_length} moves)</span>
      </div>
    {/each}
  </div>
{/if}
```

**Step 6: Run type check**

Run: `npx svelte-check 2>&1 | tail -3`
Expected: 0 ERRORS

**Step 7: Commit**

```bash
git add src/lib/stores/review.svelte.ts src/views/ReviewView.svelte
git commit -m "feat(frontend): show SGF variations as alternative moves in review mode"
```

---

## Part 2: Onboarding Flow

### Task 7: Add onboarding settings to backend

**Files:**
- Modify: `src-tauri/src/commands/settings.rs:7-32` (add fields + read/write)
- Modify: `src/lib/api/types.ts:60-69` (update Settings type)

**Step 1: Add fields to Rust `Settings` struct**

In `settings.rs`, add after `theme: String` (line 16):

```rust
    pub onboarding_completed: bool,
    pub experience_level: String,
```

In `Default` impl, add:

```rust
            onboarding_completed: false,
            experience_level: String::new(),
```

**Step 2: Read the new fields in `get_settings`**

In the `match` block (line 46-63), add:

```rust
            "onboarding_completed" => settings.onboarding_completed = value == "true",
            "experience_level" => settings.experience_level = value,
```

**Step 3: Write the new fields in `update_settings`**

In the `pairs` vec (line 77-86), add:

```rust
        ("onboarding_completed", settings.onboarding_completed.to_string()),
        ("experience_level", settings.experience_level.clone()),
```

**Step 4: Update TypeScript `Settings` type**

In `src/lib/api/types.ts`, add to the `Settings` type:

```typescript
  onboarding_completed: boolean;
  experience_level: string;
```

**Step 5: Run tests**

Run: `cargo test -p gosensei-app -v 2>&1 | tail -5` and `npx svelte-check 2>&1 | tail -3`
Expected: PASS (may need to update Settings defaults in frontend stores)

**Step 6: Update settings store default**

Check `src/lib/stores/settings.svelte.ts` — if it has a default Settings value, add the new fields with defaults (`onboarding_completed: false`, `experience_level: ""`).

**Step 7: Commit**

```bash
git add src-tauri/src/commands/settings.rs src/lib/api/types.ts src/lib/stores/settings.svelte.ts
git commit -m "feat: add onboarding_completed and experience_level to Settings"
```

---

### Task 8: Create tutorial exercise data

**Files:**
- Create: `src/lib/onboarding/exercises.ts`

**Step 1: Create the file with 4 hardcoded exercises**

```typescript
export type TutorialExercise = {
  id: string;
  title: string;
  instruction: string;
  boardSize: number;
  setupBlack: [number, number][];
  setupWhite: [number, number][];
  playerColor: "black" | "white";
  correctMove: [number, number];
  successMessage: string;
};

export const tutorialExercises: TutorialExercise[] = [
  {
    id: "capture",
    title: "Capturing Stones",
    instruction: "The white stone has only one liberty left. Place a black stone to capture it!",
    boardSize: 5,
    setupBlack: [[1, 2], [2, 1], [2, 3]],
    setupWhite: [[2, 2]],
    playerColor: "black",
    correctMove: [3, 2],
    successMessage: "You captured the white stone by filling its last liberty.",
  },
  {
    id: "ko",
    title: "The Ko Rule",
    instruction: "Black can capture the white stone, but the position would repeat. This is called ko — you can't immediately recapture. Capture the white stone!",
    boardSize: 5,
    setupBlack: [[1, 2], [2, 1], [3, 2], [2, 3]],
    setupWhite: [[1, 3], [2, 2], [3, 3]],
    playerColor: "black",
    correctMove: [2, 4],
    successMessage: "After capturing, White cannot immediately take back — that's the ko rule. Players must play elsewhere first.",
  },
  {
    id: "territory",
    title: "Counting Territory",
    instruction: "Black has surrounded territory in the corner. Tap the key intersection to secure it!",
    boardSize: 5,
    setupBlack: [[0, 2], [1, 2], [2, 0], [2, 1], [2, 2]],
    setupWhite: [[3, 0], [3, 1], [3, 2], [3, 3]],
    playerColor: "black",
    correctMove: [0, 0],
    successMessage: "Black's corner is now secured. The empty points surrounded by your stones are your territory.",
  },
  {
    id: "life_death",
    title: "Life and Death",
    instruction: "White's group needs two eyes to live. Find the vital point to kill it!",
    boardSize: 5,
    setupBlack: [[0, 0], [0, 1], [0, 3], [0, 4], [1, 0], [1, 4], [2, 0], [2, 1], [2, 2], [2, 3], [2, 4]],
    setupWhite: [[0, 2], [1, 1], [1, 2], [1, 3]],
    playerColor: "black",
    correctMove: [0, 2],
    successMessage: "By playing at the vital point, White cannot make two eyes. The group is dead!",
  },
];
```

**Step 2: Run type check**

Run: `npx svelte-check 2>&1 | tail -3`
Expected: 0 ERRORS

**Step 3: Commit**

```bash
git add src/lib/onboarding/exercises.ts
git commit -m "feat: add 4 tutorial exercises for beginner onboarding"
```

---

### Task 9: Create OnboardingView — Welcome + Experience steps

**Files:**
- Create: `src/views/OnboardingView.svelte`

**Step 1: Create the multi-step onboarding view**

```svelte
<script lang="ts">
  import BoardCanvas from "../lib/board/BoardCanvas.svelte";
  import { boardThemeForName } from "../lib/board/themes";
  import { themeStore } from "../lib/stores/theme.svelte";
  import { settingsStore } from "../lib/stores/settings.svelte";
  import { tutorialExercises, type TutorialExercise } from "../lib/onboarding/exercises";
  import * as api from "../lib/api/commands";
  import type { StoneColor, StonePosition, GameState } from "../lib/api/types";

  type Props = {
    onComplete: () => void;
  };

  let { onComplete }: Props = $props();

  type Step = "welcome1" | "welcome2" | "experience" | "tutorial" | "calibration" | "profile" | "done";
  let step = $state<Step>("welcome1");
  let experienceLevel = $state("");

  // Tutorial state
  let tutorialIndex = $state(0);
  let tutorialFeedback = $state<string | null>(null);

  // Calibration state
  let calibrationState = $state<GameState | null>(null);
  let calibrationMoveCount = $state(0);

  // Profile state
  let profileRank = $state(25);

  const currentExercise = $derived(tutorialExercises[tutorialIndex] ?? null);

  const tutorialStones = $derived<StonePosition[]>(
    currentExercise
      ? [
          ...currentExercise.setupBlack.map(([row, col]) => ({ row, col, color: "black" as StoneColor })),
          ...currentExercise.setupWhite.map(([row, col]) => ({ row, col, color: "white" as StoneColor })),
        ]
      : [],
  );

  function selectExperience(level: string) {
    experienceLevel = level;
    if (level === "never") {
      step = "tutorial";
    } else {
      startCalibration(level);
    }
  }

  function handleTutorialClick(row: number, col: number) {
    if (!currentExercise) return;
    const [cr, cc] = currentExercise.correctMove;
    if (row === cr && col === cc) {
      tutorialFeedback = currentExercise.successMessage;
      setTimeout(() => {
        tutorialFeedback = null;
        if (tutorialIndex < tutorialExercises.length - 1) {
          tutorialIndex++;
        } else {
          // Tutorial complete — start calibration at beginner level
          startCalibration("never");
        }
      }, 2500);
    } else {
      tutorialFeedback = "Not quite — try again!";
      setTimeout(() => (tutorialFeedback = null), 1500);
    }
  }

  async function startCalibration(level: string) {
    step = "calibration";
    const strength = level === "never" || level === "rules" ? "beginner" : level === "casual" ? "intermediate" : "advanced";
    try {
      calibrationState = await api.newGame(9, 6.5, "black");
      calibrationMoveCount = 0;
      // Set AI strength for the calibration
      const settings = { ...settingsStore.value, ai_strength: strength };
      await api.updateSettings(settings);
      await api.startEngine();
    } catch (e) {
      console.error("Failed to start calibration:", e);
      // Skip calibration on error
      finishOnboarding();
    }
  }

  async function handleCalibrationMove(row: number, col: number) {
    if (!calibrationState || calibrationState.phase !== "Playing") return;
    try {
      calibrationState = await api.playMove(row, col);
      calibrationMoveCount++;
      if (calibrationState.phase === "Playing") {
        // AI responds
        calibrationState = await api.requestAiMove();
        calibrationMoveCount++;
      }
    } catch (e) {
      console.error("Calibration move failed:", e);
    }
  }

  async function endCalibration() {
    try {
      const profile = await api.getSkillProfile();
      profileRank = Math.round(profile.overall_rank);
    } catch {
      profileRank = 25;
    }
    step = "profile";
  }

  async function finishOnboarding() {
    try {
      const settings = {
        ...settingsStore.value,
        onboarding_completed: true,
        experience_level: experienceLevel,
      };
      await api.updateSettings(settings);
      settingsStore.update(settings);
    } catch (e) {
      console.error("Failed to save onboarding:", e);
    }
    onComplete();
  }
</script>

<div class="flex h-full items-center justify-center" style="background-color: var(--surface-primary, #1c1917); color: var(--text-primary, #f5f5f4);">
  {#if step === "welcome1"}
    <div class="flex max-w-md flex-col items-center gap-6 text-center">
      <h1 class="text-4xl font-bold text-amber-500">Welcome to GoSensei</h1>
      <p class="text-lg text-stone-300">
        Go is one of the oldest and deepest strategy games in the world.
        GoSensei is your personal AI tutor — here to help you learn, improve, and enjoy the game.
      </p>
      <button onclick={() => (step = "welcome2")} class="rounded-lg bg-amber-700 px-8 py-3 text-lg font-semibold text-white hover:bg-amber-600">
        Next
      </button>
      <div class="flex gap-2">
        <div class="h-2 w-8 rounded bg-amber-500"></div>
        <div class="h-2 w-8 rounded bg-stone-600"></div>
      </div>
    </div>

  {:else if step === "welcome2"}
    <div class="flex max-w-md flex-col items-center gap-6 text-center">
      <h2 class="text-2xl font-bold text-stone-200">How GoSensei Helps</h2>
      <div class="flex flex-col gap-4 text-left text-stone-300">
        <div class="flex items-start gap-3">
          <span class="text-2xl">🎯</span>
          <p><strong class="text-stone-100">Adaptive AI opponent</strong> that matches your level and grows with you</p>
        </div>
        <div class="flex items-start gap-3">
          <span class="text-2xl">💡</span>
          <p><strong class="text-stone-100">Coaching after each move</strong> — explains mistakes and suggests better plays</p>
        </div>
        <div class="flex items-start gap-3">
          <span class="text-2xl">📚</span>
          <p><strong class="text-stone-100">Practice problems</strong> that target your weaknesses with spaced repetition</p>
        </div>
      </div>
      <button onclick={() => (step = "experience")} class="rounded-lg bg-amber-700 px-8 py-3 text-lg font-semibold text-white hover:bg-amber-600">
        Let's Begin
      </button>
      <div class="flex gap-2">
        <div class="h-2 w-8 rounded bg-stone-600"></div>
        <div class="h-2 w-8 rounded bg-amber-500"></div>
      </div>
    </div>

  {:else if step === "experience"}
    <div class="flex max-w-lg flex-col items-center gap-6 text-center">
      <h2 class="text-2xl font-bold text-stone-200">What's your experience with Go?</h2>
      <div class="grid grid-cols-2 gap-3">
        {#each [
          { level: "never", label: "Never played", desc: "I'm completely new" },
          { level: "rules", label: "Know the rules", desc: "I understand the basics" },
          { level: "casual", label: "Play casually", desc: "I've played some games" },
          { level: "ranked", label: "I have a rank", desc: "I play regularly" },
        ] as option}
          <button
            onclick={() => selectExperience(option.level)}
            class="flex flex-col items-center gap-1 rounded-lg border border-stone-600 bg-stone-800 p-4 text-center hover:border-amber-500 hover:bg-stone-700"
          >
            <span class="font-semibold text-stone-200">{option.label}</span>
            <span class="text-xs text-stone-400">{option.desc}</span>
          </button>
        {/each}
      </div>
    </div>

  {:else if step === "tutorial" && currentExercise}
    <div class="flex gap-6">
      <div class="flex flex-col items-center gap-3">
        <h2 class="text-xl font-bold text-stone-200">{currentExercise.title}</h2>
        <p class="max-w-xs text-sm text-stone-300">{currentExercise.instruction}</p>
        <BoardCanvas
          boardSize={currentExercise.boardSize}
          stones={tutorialStones}
          currentColor={currentExercise.playerColor}
          lastMove={null}
          theme={boardThemeForName(themeStore.active)}
          onIntersectionClick={handleTutorialClick}
        />
        {#if tutorialFeedback}
          <div class="rounded bg-stone-800 px-4 py-2 text-sm font-medium {tutorialFeedback.startsWith('Not') ? 'text-red-400' : 'text-emerald-400'}">
            {tutorialFeedback}
          </div>
        {/if}
        <div class="text-xs text-stone-500">Exercise {tutorialIndex + 1} of {tutorialExercises.length}</div>
      </div>
    </div>

  {:else if step === "calibration" && calibrationState}
    <div class="flex gap-6">
      <div class="flex flex-col items-center gap-3">
        <h2 class="text-xl font-bold text-stone-200">Calibration Game</h2>
        <p class="max-w-xs text-sm text-stone-300">
          Play a few moves so GoSensei can estimate your level. No pressure — just play naturally!
        </p>
        <BoardCanvas
          boardSize={calibrationState.board_size}
          stones={calibrationState.stones}
          currentColor={calibrationState.current_color}
          lastMove={calibrationState.last_move}
          animate
          theme={boardThemeForName(themeStore.active)}
          onIntersectionClick={handleCalibrationMove}
        />
        <div class="flex items-center gap-3">
          <span class="text-xs text-stone-500">Move {calibrationMoveCount}</span>
          {#if calibrationMoveCount >= 10}
            <button onclick={endCalibration} class="rounded bg-amber-700 px-4 py-2 text-sm font-semibold text-white hover:bg-amber-600">
              That's enough — show my profile
            </button>
          {:else}
            <span class="text-xs text-stone-500">(play at least 10 moves)</span>
          {/if}
        </div>
      </div>
    </div>

  {:else if step === "profile"}
    <div class="flex max-w-md flex-col items-center gap-6 text-center">
      <h2 class="text-2xl font-bold text-stone-200">Your Starting Profile</h2>
      <div class="rounded-lg border border-stone-600 bg-stone-800 p-6">
        <div class="text-4xl font-bold text-amber-500">{profileRank} kyu</div>
        <p class="mt-1 text-sm text-stone-400">Estimated starting level</p>
      </div>
      <p class="text-sm text-stone-300">
        We recommend starting with <strong class="text-stone-100">9×9 games</strong> at
        <strong class="text-stone-100">{experienceLevel === "ranked" ? "advanced" : "beginner"} difficulty</strong>.
        GoSensei will adjust as you improve.
      </p>
      <button onclick={finishOnboarding} class="rounded-lg bg-amber-700 px-8 py-3 text-lg font-semibold text-white hover:bg-amber-600">
        Start Playing
      </button>
    </div>
  {/if}
</div>
```

**Step 2: Run type check**

Run: `npx svelte-check 2>&1 | tail -3`
Expected: 0 ERRORS (may need adjustments for exact types)

**Step 3: Commit**

```bash
git add src/views/OnboardingView.svelte
git commit -m "feat: create OnboardingView with welcome, experience, tutorial, calibration, and profile steps"
```

---

### Task 10: Wire OnboardingView into App.svelte

**Files:**
- Modify: `src/App.svelte`

**Step 1: Import and gate on onboarding_completed**

At the top of `App.svelte`:
- Add `import OnboardingView from "./views/OnboardingView.svelte";`
- Change the `currentView` type to include `"onboarding"`:
  ```typescript
  let currentView = $state<"home" | "play" | "review" | "dashboard" | "problem" | "onboarding">("home");
  ```

**Step 2: Check onboarding in the `onMount`**

In the `onMount` callback, after loading settings, add:

```typescript
if (!settings.onboarding_completed) {
  currentView = "onboarding";
}
```

**Step 3: Add the view case**

In the template, add before the `{:else if currentView === "play"}` branch:

```svelte
  {:else if currentView === "onboarding"}
    <OnboardingView onComplete={() => { currentView = "home"; }} />
```

**Step 4: Run type check**

Run: `npx svelte-check 2>&1 | tail -3`
Expected: 0 ERRORS

**Step 5: Commit**

```bash
git add src/App.svelte
git commit -m "feat: gate app on onboarding — show OnboardingView on first launch"
```

---

### Task 11: Final integration test

**Step 1: Run full test suite**

```bash
cargo test --workspace 2>&1 | grep -E "^test result|^running"
npx svelte-check 2>&1 | tail -3
```

Expected:
- All 182+ Rust tests pass (plus ~7 new variation tests)
- 0 TypeScript errors

**Step 2: Commit any fixes**

**Step 3: Final commit message for the phase**

```bash
git log --oneline -12
```

Review the commit history to ensure all Phase 3 work is captured.
