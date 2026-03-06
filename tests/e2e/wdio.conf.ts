import os from "os";
import path from "path";
import { spawn, type ChildProcess } from "child_process";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const projectRoot = path.resolve(__dirname, "../..");

let tauriDriver: ChildProcess | null = null;
let shuttingDown = false;

// Platform-specific binary name
const appBinary = process.platform === "win32"
  ? path.join(projectRoot, "src-tauri", "target", "release", "gosensei-app.exe")
  : path.join(projectRoot, "src-tauri", "target", "release", "gosensei-app");

export const config: WebdriverIO.Config = {
  runner: "local",
  host: "127.0.0.1",
  port: 4444,
  specs: ["./specs/**/*.ts"],
  maxInstances: 1,

  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: appBinary,
      },
    },
  ],

  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },

  // Log level: trace | debug | info | warn | error | silent
  logLevel: "warn",

  // Wait for elements up to 10s
  waitforTimeout: 10000,

  beforeSession() {
    const driverPath = path.resolve(
      os.homedir(),
      ".cargo",
      "bin",
      process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver",
    );

    tauriDriver = spawn(driverPath, [], {
      stdio: [null, process.stdout, process.stderr],
    });

    tauriDriver.on("error", (error) => {
      console.error("tauri-driver failed to start:", error);
      process.exit(1);
    });

    tauriDriver.on("exit", (code) => {
      if (!shuttingDown) {
        console.error("tauri-driver exited unexpectedly with code:", code);
        process.exit(1);
      }
    });
  },

  afterSession() {
    closeTauriDriver();
  },
};

function closeTauriDriver() {
  shuttingDown = true;
  tauriDriver?.kill();
  tauriDriver = null;
}

// Ensure cleanup on unexpected exit
for (const signal of ["exit", "SIGINT", "SIGTERM", "SIGHUP"] as const) {
  process.on(signal, () => {
    closeTauriDriver();
    if (signal !== "exit") process.exit();
  });
}
