import type { KataGoSetupProgress, KataGoStatus } from "../api/types";
import { getKataGoStatus, setupKataGo } from "../api/commands";
import { onKataGoSetupProgress } from "../api/events";

export const setupStore = createSetupStore();

function createSetupStore() {
  let status = $state<KataGoStatus>("not_installed");
  let progress = $state<KataGoSetupProgress | null>(null);
  let error = $state<string | null>(null);
  let unlisten: (() => void) | null = null;

  return {
    get status() {
      return status;
    },
    get progress() {
      return progress;
    },
    get error() {
      return error;
    },
    get downloadPercent() {
      if (!progress || progress.total === 0) return 0;
      return Math.round((progress.downloaded / progress.total) * 100);
    },
    get phaseLabel() {
      if (!progress) return "";
      if (progress.phase === "binary") return "Downloading KataGo engine...";
      if (progress.phase === "model") return "Downloading neural network...";
      return "Complete";
    },

    async refresh() {
      try {
        status = await getKataGoStatus();
        error = null;
      } catch (e) {
        error = String(e);
      }
    },

    async startSetup() {
      if (status === "ready") return;
      error = null;
      progress = null;

      if (!unlisten) {
        unlisten = await onKataGoSetupProgress((p) => {
          progress = p;
        });
      }

      try {
        const result = await setupKataGo();
        status = result === "ready" ? "ready" : "not_installed";
        progress = null;
      } catch (e) {
        status = "not_installed";
        error = String(e);
        progress = null;
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
