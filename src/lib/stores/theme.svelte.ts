import type { ThemeName } from "../api/types";

export const themeStore = createThemeStore();

function createThemeStore() {
  let active = $state<ThemeName>("study");

  return {
    get active() {
      return active;
    },
    set(theme: ThemeName) {
      active = theme;
      document.documentElement.dataset.theme = theme;
    },
  };
}
