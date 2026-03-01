import type { GameState } from "../api/types";

export const gameStore = createGameStore();

function createGameStore() {
  let state = $state<GameState | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  return {
    get state() {
      return state;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    set(newState: GameState) {
      state = newState;
      error = null;
    },
    setLoading(value: boolean) {
      loading = value;
    },
    setError(msg: string) {
      error = msg;
      loading = false;
    },
    clear() {
      state = null;
      error = null;
      loading = false;
    },
  };
}
