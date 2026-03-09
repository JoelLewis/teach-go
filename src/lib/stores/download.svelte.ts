import type { DownloadStatus, DownloadState } from "../api/types";
import { getDownloadStatus, retryDownloads } from "../api/commands";
import { onDownloadProgress } from "../api/events";

export const downloadStore = createDownloadStore();

function createDownloadStore() {
  let status = $state<DownloadStatus>({
    katago: { state: "not_installed" },
    llm: { state: "not_installed" },
  });
  let unlisten: (() => void) | null = null;

  return {
    get status() {
      return status;
    },
    get katagoReady() {
      return status.katago.state === "ready";
    },
    get llmReady() {
      return status.llm.state === "ready";
    },
    get katagoDownloading() {
      return status.katago.state === "downloading";
    },
    get llmDownloading() {
      return status.llm.state === "downloading";
    },
    get anyDownloading() {
      return status.katago.state === "downloading" || status.llm.state === "downloading";
    },
    get katagoProgress() {
      if (status.katago.state === "downloading") return status.katago.progress;
      if (status.katago.state === "ready") return 100;
      return 0;
    },
    get llmProgress() {
      if (status.llm.state === "downloading") return status.llm.progress;
      if (status.llm.state === "ready") return 100;
      return 0;
    },
    get katagoPhase() {
      if (status.katago.state === "downloading") return status.katago.phase;
      return "";
    },
    get katagoError(): string | null {
      if (status.katago.state === "error") return status.katago.message;
      return null;
    },
    get llmError(): string | null {
      if (status.llm.state === "error") return status.llm.message;
      return null;
    },

    async refresh() {
      try {
        status = await getDownloadStatus();
      } catch {
        // Backend may not be ready yet
      }
    },

    async startListening() {
      if (unlisten) return;
      unlisten = await onDownloadProgress((s) => {
        status = s;
      });
    },

    async retry() {
      await retryDownloads();
    },

    cleanup() {
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
    },
  };
}
