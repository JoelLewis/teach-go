import type { LlmStatus, LlmDownloadProgress } from "../api/types";
import { getLlmStatus, initLlmModel } from "../api/commands";
import { onLlmDownloadProgress } from "../api/events";

export const llmStore = createLlmStore();

function createLlmStore() {
  let status = $state<LlmStatus>("disabled");
  let downloadProgress = $state<LlmDownloadProgress | null>(null);
  let error = $state<string | null>(null);
  let unlisten: (() => void) | null = null;

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
