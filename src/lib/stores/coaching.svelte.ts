import type { CoachingMessage, Severity } from "../api/types";

export const coachingStore = createCoachingStore();

function createCoachingStore() {
  let messages = $state<CoachingMessage[]>([]);
  let lastMoveSeverity = $state<Severity | null>(null);
  let streamingMoveNumber = $state<number | null>(null);
  let streamingText = $state("");
  let flushPending = false;

  return {
    get messages() {
      return messages;
    },
    get lastMoveSeverity() {
      return lastMoveSeverity;
    },
    get streamingMoveNumber() {
      return streamingMoveNumber;
    },
    get streamingText() {
      return streamingText;
    },
    add(message: CoachingMessage) {
      messages = [...messages, message];
      lastMoveSeverity = message.severity;
    },
    setLastMoveSeverity(severity: Severity | null) {
      lastMoveSeverity = severity;
    },
    /** Begin streaming a coaching message — creates a placeholder entry. */
    startStream(severity: Severity, moveNumber: number, scoreLoss: number) {
      streamingMoveNumber = moveNumber;
      streamingText = "";
      // Add a placeholder message that will be updated as tokens arrive
      const placeholder: CoachingMessage = {
        severity,
        error_class: null,
        message: "",
        suggested_move: null,
        simplest_move: null,
        score_loss: scoreLoss,
        move_number: moveNumber,
      };
      messages = [...messages, placeholder];
      lastMoveSeverity = severity;
    },
    /** Append a text delta to the currently streaming message (batched via rAF). */
    appendStream(moveNumber: number, delta: string) {
      if (streamingMoveNumber !== moveNumber) return;
      streamingText += delta;
      if (!flushPending) {
        flushPending = true;
        requestAnimationFrame(() => {
          flushPending = false;
          const last = messages[messages.length - 1];
          if (last && last.move_number === streamingMoveNumber) {
            messages = [
              ...messages.slice(0, -1),
              { ...last, message: streamingText },
            ];
          }
        });
      }
    },
    /** Mark streaming as complete for a move. */
    completeStream(moveNumber: number) {
      if (streamingMoveNumber === moveNumber) {
        streamingMoveNumber = null;
        streamingText = "";
      }
    },
    clear() {
      messages = [];
      lastMoveSeverity = null;
      streamingMoveNumber = null;
      streamingText = "";
    },
  };
}
