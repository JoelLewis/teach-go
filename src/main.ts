import { mount } from "svelte";
import App from "./App.svelte";

window.addEventListener("error", (e) => {
  document.body.innerHTML += `<pre style="color:red;padding:20px;font-size:14px;">JS ERROR: ${e.message}\n${e.filename}:${e.lineno}</pre>`;
});
window.addEventListener("unhandledrejection", (e) => {
  document.body.innerHTML += `<pre style="color:orange;padding:20px;font-size:14px;">UNHANDLED PROMISE: ${e.reason}</pre>`;
});

if (import.meta.env.DEV) {
  // Guest half of tauri-plugin-mcp: answers execute-js / get-page-map /
  // get-element-position / get-dom events emitted by the Rust side (only
  // present in `pnpm playtest` builds). Statically eliminated from
  // production bundles by Vite's DEV branch removal.
  import("tauri-plugin-mcp")
    .then((mcp) => mcp.setupPluginListeners())
    .catch((err: unknown) => {
      console.warn("tauri-plugin-mcp guest bindings not installed:", err);
    });
  // window.__playtest driving hooks (getView / getGameState / clickPoint).
  import("./lib/dev/playtest").then((p) => p.installPlaytestHooks());
}

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
