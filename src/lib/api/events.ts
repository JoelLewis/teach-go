import { listen } from "@tauri-apps/api/event";

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
