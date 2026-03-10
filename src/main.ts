import { mount } from "svelte";
import App from "./App.svelte";

window.addEventListener("error", (e) => {
  document.body.innerHTML += `<pre style="color:red;padding:20px;font-size:14px;">JS ERROR: ${e.message}\n${e.filename}:${e.lineno}</pre>`;
});
window.addEventListener("unhandledrejection", (e) => {
  document.body.innerHTML += `<pre style="color:orange;padding:20px;font-size:14px;">UNHANDLED PROMISE: ${e.reason}</pre>`;
});

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
