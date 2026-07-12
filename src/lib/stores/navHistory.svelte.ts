const MAX_HISTORY = 50;

export type AppView = "home" | "play" | "review" | "dashboard" | "problem" | "onboarding";

/** A history entry — onboarding is a one-time flow and never enters history. */
export type NavEntry = {
  view: Exclude<AppView, "onboarding">;
  reviewGameId?: number;
};

export function createNavHistory(cap: number = MAX_HISTORY) {
  let back = $state<NavEntry[]>([]);
  let forward = $state<NavEntry[]>([]);

  function pushCapped(stack: NavEntry[], entry: NavEntry) {
    stack.push(entry);
    if (stack.length > cap) stack.shift();
  }

  return {
    get canGoBack() {
      return back.length > 0;
    },
    get canGoForward() {
      return forward.length > 0;
    },
    /** Record the view being left when navigating somewhere new. Clears the forward stack. */
    push(from: NavEntry) {
      pushCapped(back, from);
      forward = [];
    },
    /** Step back; `current` becomes a forward entry. Returns null at the start of history. */
    goBack(current: NavEntry): NavEntry | null {
      const entry = back.pop();
      if (!entry) return null;
      pushCapped(forward, current);
      return entry;
    },
    /** Step forward; `current` becomes a back entry. Returns null at the end of history. */
    goForward(current: NavEntry): NavEntry | null {
      const entry = forward.pop();
      if (!entry) return null;
      pushCapped(back, current);
      return entry;
    },
    clear() {
      back = [];
      forward = [];
    },
  };
}

export const navHistory = createNavHistory();
