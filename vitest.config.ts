import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [svelte({ hot: false })],
  // Component tests run in jsdom; without this Svelte resolves to its
  // server build and mount() is unavailable.
  resolve: { conditions: ["browser"] },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
    setupFiles: ["tests/setup.ts"],
  },
});
