# 🛰️ Spotlight-Win: Elite Insight Engine

**v0.4.5 — Better, Faster, Smarter than Windows Search**

Spotlight-Win is a professional-grade, high-fidelity productivity tool designed for power users who demand instant results, native privacy, and "Elite" desktop aesthetics. More than just a search bar, it is an **Insight Engine** built to eliminate context switching and maximize deep work.

![Spotlight-Win Banner](public/logo_128.png)

## 💎 The "Elite" Experience

### 🚀 Performance Without Compromise

- **Silent Intelligence**: Optimized background indexing that consumes <1% CPU.
- **Zero-Click Workflow**: Designed for keyboard-first mastery with <150ms response times.
- **Batched Disk Commits**: Intelligent I/O management that protects your SSD while keeping the index fresh.
- **Narrowed Watcher**: Strategic directory monitoring focuses on high-value folders (Desktop, Start Menu, etc.), ignoring system noise.

### 🎨 Stunning Modern UI

- **Interactive Glassmorphism**: A sleek, translucent interface that feels like "Desktop Jewelry."
- **Smart Folding**: Interactive, collapsible category headers to manage complex search results.
- **High-Fidelity Branding**: Custom-designed assets and typography that outshine native OS utilities.

### 🛡️ Security & Privacy

- **Native Execution**: Leverages `ShellExecuteW` for secure, direct-to-OS application launching.
- **Smart Gaming Detection**: Automatically suppresses the global shortcut when a full-screen game or app is active, preventing key conflicts during gameplay.
- **"Anti-Bing" Philosophy**: Local-first intelligence. No data mining, no telemetry, and zero advertisements.
- **Sandboxed logic**: No arbitrary shell execution; every launch is validated against a secure internal cache.

### 🚀 Elite Features

- **Helper Manual**: Integrated in-app documentation via the `/help` command or mouse icon.
- **Ghost Action Bar**: Instant "Reveal" and "Forget" actions on every search result.
- **Breadcrumb Navigation**: Smart `Parent > Child` folder labeling providing instant directory context without cluttering the search.
- **Silent Intelligence**: High-performance Rust indexing with <1% CPU impact.
- **Size-Z Optimized**: Hardened release profile for a minimal binary footprint.
- **Zero-Bing Privacy**: 100% offline-first local search. No tracking. No ads.

## 🛠️ Development & Building

To build and package a release automatically:

```bash
npm run build:release
```

This command compiles the Rust engine, packages the UI, and generates a compressed `.zip` release in the root directory.

## 🚀 Getting Started

1. **Download**: Grab the latest `spotlight-win_0.4.5_x64-setup.zip` from the releases.
2. **Launch**: Use `Ctrl + Space` to bring the bar to life.
3. **Master**: Use `>` for commands or just start typing to see the magic.

---

_Spotlight-Win is an independent project dedicated to reclaiming the desktop for power users._
