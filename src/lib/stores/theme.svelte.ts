import type { ThemeName } from "../api/types";

const STORAGE_KEY = "gosensei-theme";

export const themeStore = createThemeStore();

function createThemeStore() {
  const stored = (typeof localStorage !== "undefined"
    ? localStorage.getItem(STORAGE_KEY)
    : null) as ThemeName | null;
  let active = $state<ThemeName>(stored ?? "study");

  return {
    get active() {
      return active;
    },
    set(theme: ThemeName) {
      active = theme;
      document.documentElement.dataset.theme = theme;
      localStorage.setItem(STORAGE_KEY, theme);
    },
  };
}
