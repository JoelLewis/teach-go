import { listen } from "@tauri-apps/api/event";
import type { CoachingStreamChunk, KataGoSetupProgress, LlmDownloadProgress, ReviewProgress } from "./types";

export type EngineStatus = "starting" | "ready" | "error" | "stopped";

export function onEngineStatus(
  callback: (status: EngineStatus) => void,
): Promise<() => void> {
  return listen<EngineStatus>("engine-status", (event) => {
    callback(event.payload);
  }).then((unlisten) => unlisten);
}

export function onAiThinking(
  callback: (thinking: boolean) => void,
): Promise<() => void> {
  return listen<boolean>("ai-thinking", (event) => {
    callback(event.payload);
  }).then((unlisten) => unlisten);
}

export function onReviewProgress(
  callback: (progress: ReviewProgress) => void,
): Promise<() => void> {
  return listen<ReviewProgress>("review-progress", (event) => {
    callback(event.payload);
  }).then((unlisten) => unlisten);
}

export function onCoachingStream(
  callback: (chunk: CoachingStreamChunk) => void,
): Promise<() => void> {
  return listen<CoachingStreamChunk>("coaching-stream", (event) => {
    callback(event.payload);
  }).then((unlisten) => unlisten);
}

export function onKataGoSetupProgress(
  callback: (progress: KataGoSetupProgress) => void,
): Promise<() => void> {
  return listen<KataGoSetupProgress>("katago-setup-progress", (event) => {
    callback(event.payload);
  }).then((unlisten) => unlisten);
}

export function onLlmDownloadProgress(
  callback: (progress: LlmDownloadProgress) => void,
): Promise<() => void> {
  return listen<LlmDownloadProgress>("llm-download-progress", (event) => {
    callback(event.payload);
  }).then((unlisten) => unlisten);
}
