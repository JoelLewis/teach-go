# GoSensei Design Critique Remediation — Design Spec

## Goal

Address the 5 priority issues from the design critique: transform the home screen from a parking lot into an atmospheric board-forward experience, compress onboarding from 7 steps to 3, add board toast coaching, replace the New Game modal with inline expansion, and fix spacing/hover issues across the app.

## Scope

**In scope:**
1. Board-forward home screen with ghosted board background
2. Animated ghost-to-real board transition on game/problem start
3. Inline new game options (replaces NewGameDialog modal)
4. Compressed 3-step onboarding (Welcome → Tutorial → Start Playing)
5. Board toast coaching overlay with auto-fade
6. Spacing hierarchy pass (varied gaps instead of uniform)
7. Button hover fix (use `--btn-hover` color instead of opacity)

**Deferred:**
- Dashboard radar chart redesign (needs category validation)
- Settings modal replacement (functional, low priority)
- On-board speech bubbles (scaling issues on busy boards)

## 1. Board-Forward Home Screen

### Layout

The home screen centers around a ghosted board grid that fills the background, creating atmosphere without competing with UI elements.

```
┌─────────────────────────────────────┐
│         (ghosted board grid)        │
│                                     │
│            GoSensei                 │  ← 4xl, display font, accent color
│         Your AI Go tutor            │  ← sm, italic, secondary
│             ~15 kyu                 │  ← sm, accent, bold
│                                     │
│        ┌──────────────────┐         │
│        │    New Game       │         │  ← btn-primary btn-lg, hero CTA
│        └──────────────────┘         │
│                                     │
│     [Problems]  [Progress]  [⚙]    │  ← btn-secondary btn-sm + btn-ghost
│                                     │
│   Last game: 9×9 · B+3.5 · 2d ago  │  ← text-dim, xs
│                                     │
└─────────────────────────────────────┘
```

### Ghosted Board

- Render a static 9x9 SVG grid (lines + star points only, no stones) behind all content
- Study theme: `--surface-board` color lines at ~12% opacity on `--surface-primary` background
- Grid theme: `--border-color` lines at ~10% opacity on `--surface-primary` background
- Grid is centered and sized to fill most of the viewport (e.g., 80% of viewport height)
- Lines use the same geometry as `BoardSvg.svelte`'s grid rendering but simplified (no click targets, no stones, no hover)

### Inline New Game Expansion

When user clicks "New Game":
1. Options slide open below the button (board size, color, AI strength)
2. The ghosted board grid animates to match the selected board size (9→13→19 lines)
3. A "Start" button appears at the bottom of the expanded options
4. Clicking "Start" triggers the ghost→real transition

The expansion replaces `NewGameDialog.svelte` modal. The component becomes an inline section within `HomeView.svelte`.

### Ghost-to-Real Board Transition

When user clicks "Start" (or "Practice Problems" → selects a problem):
1. The ghosted board grid scales and repositions to match the play view's board area
2. Opacity animates from ghost (~12%) to full (100%)
3. Board color fills in (wood texture or ink surface)
4. Once the transition completes, the view switches to PlayView/ProblemView
5. CSS transition using `transform`, `opacity`, and `background-color` (~400ms, ease-out-quart)

Implementation: Use Svelte's `{#key}` blocks or a shared layout animation. The simplest approach is a CSS transition on a shared board container that persists across the view switch, driven by a `transitioning` state flag in `App.svelte`.

### Player Rank

- Show estimated rank below the subtitle (e.g., "~15 kyu")
- Fetched from `api.getSkillProfile()` on mount
- If no games played yet, omit the rank line

### Recent Games

- Collapse from the current 10-game list to a single line: "Last game: 9×9 · Black by 3.5 pts · 2 days ago"
- Clickable to load that game
- If no recent games, omit entirely

## 2. Compressed Onboarding (3 Steps)

### Current Flow (7 steps)

Welcome1 → Welcome2 → Experience → Tutorial → Calibration download → Calibration game → Profile

### New Flow (3 steps)

**Step 1: Welcome** (single screen)
- Merge Welcome1 + Welcome2 content into one screen
- Title: "Welcome to GoSensei"
- 2-3 bullet points about what the app does (from Welcome2)
- Single "Let's Begin" CTA
- Ghosted board visible in background (same as home screen)

**Step 2: Tutorial**
- Keep existing tutorial exercises (these are valuable — interactive board teaching)
- Skip the experience questionnaire entirely
- Tutorial exercises serve double duty: teach basics AND gauge starting level
- Track performance silently: how many attempts per exercise, which hints used
- Infer approximate starting rank from tutorial performance

**Step 3: Start Playing**
- Brief "You're ready!" message
- Show inferred starting rank (from tutorial performance)
- "Start Playing" button → goes to home screen
- No separate calibration game step — calibration continues silently during first few real games

### Removed Steps

- **Welcome2**: Merged into Welcome (step 1)
- **Experience selection**: Removed — infer from tutorial behavior
- **Calibration download wait**: Move to background — download starts on app launch, tutorial gives it time
- **Calibration game**: Removed as explicit step — first real game serves this purpose
- **Profile display**: Folded into step 3 as a brief rank display

### Rank Inference from Tutorial

The tutorial has exercises for capturing, liberties, atari, ko. Track:
- Exercises completed without hints → higher starting rank
- Exercises needing multiple hints → lower starting rank
- Mapping: all exercises clean → ~15 kyu, some hints → ~20 kyu, struggled → ~25 kyu

This is a rough estimate. The real calibration happens over the first 3-5 games via KataGo analysis (existing system). The tutorial inference just sets a reasonable starting point.

## 3. Board Toast Coaching

### Behavior

After the AI coaching engine produces a coaching message for a mistake/blunder:
1. A toast overlay appears at the bottom of the board SVG container
2. Shows: severity badge + brief coaching text (1-2 sentences max)
3. Auto-fades after 5 seconds (opacity transition, 300ms)
4. Clicking the toast scrolls to the full message in the sidebar coaching panel
5. Toast does NOT appear for "Excellent" or "Good" moves (only Inaccuracy, Mistake, Blunder)

### Visual Design

```
┌──────────────────────────────────────┐
│                                      │
│            (Go board)                │
│                                      │
│  ┌────────────────────────────────┐  │
│  │ MISTAKE · −3.5 pts             │  │  ← severity color, bold, xs
│  │ Try connecting at E6 first.    │  │  ← text-on-card, sm
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

- Position: `absolute`, bottom 8px, left/right 8px within board container
- Background: semi-transparent surface-primary (90% opacity) with backdrop-blur
- Severity label colored with severity token
- Border-radius: 6px
- Font: body font, not display
- Max 2 lines of text — truncate with "..." if longer, full message in sidebar
- `pointer-events: auto` on the toast (clickable), `pointer-events: none` would block board interaction

### Implementation

- New component: `BoardToast.svelte`
- Positioned as a child of the board's `<div class="relative">` container in PlayView
- Receives the latest coaching message as a prop
- Internal timer auto-hides after 5 seconds
- Svelte transition: `fly` from bottom + `fade`

## 4. Spacing Hierarchy

### Current Problem

Nearly every section uses `gap-3` or `gap-4`. All panels use `p-4`. There's no visual rhythm.

### New Spacing Rules

| Relationship | Gap | Example |
|-------------|-----|---------|
| Tight group (related controls) | gap-1.5 to gap-2 | Game control buttons, score bar items |
| Section items | gap-3 | Coaching messages within panel, move history rows |
| Panel sections | gap-5 to gap-6 | Between coaching panel and game controls |
| Major divisions | gap-8 | Between board area and sidebar header |

### Specific Changes

**PlayView sidebar:**
- Game controls group: `gap-1.5` (buttons are related)
- Between score bar and game controls: `gap-2` (tightly related)
- Between game controls and coaching panel: `gap-6` (different mode: doing vs thinking)
- Between header and first content: `gap-4`

**HomeView:**
- Between title block and CTA: `gap-10` (generous, let the board breathe)
- Between CTA and secondary nav: `gap-4`
- Between secondary nav and recent game footnote: `gap-8`

**CoachingPanel:**
- Between coaching messages: `gap-2` (tight, scannable)
- Panel heading to first message: `gap-3`

## 5. Button Hover Fix

### Current Problem

All `.btn:hover` uses `opacity: 0.9` — barely perceptible, especially on the Study theme's gold buttons.

### Fix

Replace opacity hover with color-shift hover using the existing `--btn-hover` token:

```css
.btn-primary:hover {
  background-color: var(--btn-hover);
  opacity: 1; /* override base .btn:hover */
}
.btn-secondary:hover {
  background-color: var(--surface-card);
  opacity: 1;
}
.btn-danger:hover {
  filter: brightness(1.15);
  opacity: 1;
}
/* .btn-ghost:hover already works well — keep as-is */
```

Remove the base `.btn:hover { opacity: 0.9; }` rule. Each tier defines its own hover.

## File Impact Summary

| File | Change Type | Description |
|------|------------|-------------|
| `src/views/HomeView.svelte` | Major rewrite | Board-forward layout, ghosted board, inline new game, rank display |
| `src/views/OnboardingView.svelte` | Major rewrite | 3-step flow, remove experience/calibration steps, rank inference |
| `src/views/PlayView.svelte` | Moderate | Add BoardToast, spacing hierarchy changes |
| `src/components/NewGameDialog.svelte` | Delete | Replaced by inline expansion in HomeView |
| `src/components/BoardToast.svelte` | Create | Toast overlay component |
| `src/app.css` | Moderate | Button hover fix, remove opacity hover |
| `src/App.svelte` | Minor | Ghost-to-real transition coordination, remove NewGameDialog import |
| `src/views/ProblemView.svelte` | Minor | Spacing adjustments |
| `src/views/ReviewView.svelte` | Minor | Spacing adjustments |
