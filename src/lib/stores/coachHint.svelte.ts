/**
 * Session-scoped hint for the coaching panel: while coaching messages are
 * template-based because the AI coach model is still downloading, show one
 * unobtrusive note that explanations will improve once the model is ready.
 *
 * The hint appears at most once per app session and disappears permanently
 * as soon as the model download is no longer running.
 */

export type ModelDownloadState = "not_installed" | "downloading" | "ready" | "error";

export function createCoachHint() {
  let visible = $state(false);
  let shownThisSession = false;

  return {
    get visible() {
      return visible;
    },
    /**
     * Call when a coaching message lands. Shows the hint (once per session)
     * only when the model is still downloading — the one case where the
     * message is guaranteed to be template-based because the model is absent.
     */
    noteCoachingMessage(modelDownloading: boolean) {
      if (!modelDownloading || shownThisSession) return;
      visible = true;
      shownThisSession = true;
    },
    /**
     * Call when the model download state changes. Hides the hint as soon as
     * the download is no longer running (ready, failed, or not started) so
     * the copy never outlives its truth.
     */
    noteModelState(state: ModelDownloadState) {
      if (state !== "downloading") visible = false;
    },
  };
}

export const coachHintStore = createCoachHint();
