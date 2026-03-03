import type { Settings } from "../api/types";

const DEFAULT_SETTINGS: Settings = {
  board_size: 9,
  komi: 6.5,
  show_coordinates: true,
  show_move_numbers: false,
  ai_strength: "beginner",
  sound_enabled: true,
  feedback_timing: "immediate",
};

export const settingsStore = createSettingsStore();

function createSettingsStore() {
  let settings = $state<Settings>(DEFAULT_SETTINGS);

  return {
    get value() {
      return settings;
    },
    update(newSettings: Settings) {
      settings = newSettings;
    },
  };
}
