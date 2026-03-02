type SoundName = "stone" | "capture" | "pass" | "correct" | "wrong";

const SOUND_PATHS: Record<SoundName, string> = {
  stone: "/sounds/stone.wav",
  capture: "/sounds/capture.wav",
  pass: "/sounds/pass.wav",
  correct: "/sounds/correct.wav",
  wrong: "/sounds/wrong.wav",
};

let enabled = true;
const cache = new Map<SoundName, HTMLAudioElement>();

function getAudio(name: SoundName): HTMLAudioElement {
  let audio = cache.get(name);
  if (!audio) {
    audio = new Audio(SOUND_PATHS[name]);
    cache.set(name, audio);
  }
  return audio;
}

export function setEnabled(value: boolean): void {
  enabled = value;
}

export function play(name: SoundName): void {
  if (!enabled) return;
  const audio = getAudio(name);
  audio.currentTime = 0;
  audio.play().catch(() => {
    // Ignore play errors (e.g., user hasn't interacted yet)
  });
}
