# GoSensei

A free, open-source desktop app that teaches Go (Weiqi/Baduk) to beginners through AI coaching. Built with Rust, Tauri v2, Svelte 5, and Pixi.js.

GoSensei combines KataGo's analysis engine with error classification and natural-language coaching to explain not just *what* went wrong, but *why* — in language calibrated to your level.

## Features

- **Free Play** against KataGo with real-time coaching (9x9, 13x13, 19x19)
- **Problem Training** with spaced repetition (FSRS) and adaptive difficulty
- **Post-Game Review** with win-rate charts, mistake navigation, and variation exploration
- **Skill Tracking** across six Go dimensions (Life & Death, Reading, Shape, Direction, Fighting, Endgame)
- **SGF import/export** for games and problem collections
- **Onboarding flow** with interactive tutorial and calibration game
- **Two visual themes** (Study and Grid) with animated stone placement

## Prerequisites

- [Rust](https://rustup.rs/) (stable, edition 2024)
- [Node.js](https://nodejs.org/) 22+
- Platform-specific system libraries (see below)

### Linux

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

### macOS

Xcode Command Line Tools (Tauri uses WebKit, which ships with macOS):

```bash
xcode-select --install
```

### Windows

No additional system dependencies required. WebView2 ships with Windows 10+.

## Getting Started

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

On first launch, GoSensei will prompt you to download KataGo and its neural network model (~300 MB total). This is handled automatically through the in-app setup flow.

## Building

```bash
# Production build (creates platform-specific installer)
npm run tauri build
```

## Project Structure

```
gosensei/
  crates/
    gosensei-core/       # Go rules engine, board, scoring, SGF parser
    gosensei-katago/     # KataGo process manager, Analysis Engine JSON protocol
    gosensei-coaching/   # Error classification, severity, coaching templates
    gosensei-llm/        # Local LLM coaching (Gemma 3 1B, optional)
  src-tauri/             # Tauri app: IPC commands, SQLite state, KataGo setup
  src/                   # Svelte 5 frontend: Pixi.js board, stores, views
```

## Testing

```bash
# Run all Rust tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Frontend type checking
npm run check
```

## KataGo Backend

GoSensei downloads platform-appropriate KataGo binaries automatically:

| Platform | Backend | Notes |
|----------|---------|-------|
| Linux | CUDA 12 | Requires NVIDIA GPU + drivers |
| macOS | Eigen (CPU) | Works on all Macs |
| Windows | OpenCL | Works with NVIDIA and AMD GPUs |

You can also provide your own KataGo binary via the `KATAGO_BINARY` environment variable, or place it on your system PATH.

## License

MIT
