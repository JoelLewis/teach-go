import type { CoachingMessage } from "../api/types";

export const coachingStore = createCoachingStore();

function createCoachingStore() {
  let messages = $state<CoachingMessage[]>([]);

  return {
    get messages() {
      return messages;
    },
    add(message: CoachingMessage) {
      messages = [...messages, message];
    },
    clear() {
      messages = [];
    },
  };
}
