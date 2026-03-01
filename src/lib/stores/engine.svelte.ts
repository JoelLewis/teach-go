import type { EngineStatus } from "../api/events";

export const engineStore = createEngineStore();

function createEngineStore() {
  let status = $state<EngineStatus>("stopped");
  let aiThinking = $state(false);

  return {
    get status() {
      return status;
    },
    get aiThinking() {
      return aiThinking;
    },
    setStatus(newStatus: EngineStatus) {
      status = newStatus;
    },
    setAiThinking(thinking: boolean) {
      aiThinking = thinking;
    },
  };
}
