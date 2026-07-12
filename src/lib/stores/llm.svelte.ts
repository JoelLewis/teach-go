import type { LlmStatus, LlmDownloadProgress } from "../api/types";
import { getLlmStatus, initLlmModel } from "../api/commands";
import { onLlmDownloadProgress } from "../api/events";

export const llmStore = createLlmStore();

export function createLlmStore() {
  let status = $state<LlmStatus>("not_installed");
  let downloadProgress = $state<LlmDownloadProgress | null>(null);
  let error = $state<string | null>(null);
  let unlisten: (() => void) | null = null;
  // Deliberately not $state: reactive callers (effects) must not re-fire
  // when the one-shot auto-load guard flips.
  let autoLoadAttempted = false;

  return {
    get status() {
      return status;
    },
    get downloadProgress() {
      return downloadProgress;
    },
    get error() {
      return error;
    },
    get downloadPercent() {
      if (!downloadProgress || downloadProgress.total === 0) return 0;
      return Math.round(
        (downloadProgress.downloaded / downloadProgress.total) * 100,
      );
    },

    async refresh() {
      try {
        status = await getLlmStatus();
        error = null;
      } catch (e) {
        status = "disabled";
        error = String(e);
      }
    },

    /**
     * One-shot session load of an already-downloaded model into memory.
     * At most one attempt per session, whatever the outcome — callers are
     * reactive effects, so an unconditional retry would loop forever when
     * the backend reports anything but "ready".
     */
    async ensureLoaded() {
      if (autoLoadAttempted || status === "ready" || status === "loading") return;
      autoLoadAttempted = true;
      await this.startDownload();
    },

    async startDownload() {
      if (status === "ready" || status === "loading") return;
      status = "loading";
      error = null;
      downloadProgress = null;

      // Listen for progress events
      if (!unlisten) {
        unlisten = await onLlmDownloadProgress((progress) => {
          downloadProgress = progress;
        });
      }

      try {
        const result = await initLlmModel();
        status = result === "ready" ? "ready" : "not_installed";
        downloadProgress = null;
      } catch (e) {
        status = "not_installed";
        error = String(e);
        downloadProgress = null;
      } finally {
        if (unlisten) {
          unlisten();
          unlisten = null;
        }
      }
    },

    cleanup() {
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
    },
  };
}
