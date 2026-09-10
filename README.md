# 🛰️ Spotlight-Win

> **The fastest way to find or launch anything on your Windows PC.**  
> Zero setup. Zero ads. Zero learning curve. Just press <kbd>Ctrl</kbd> + <kbd>Space</kbd> and type.

![Spotlight-Win Banner](public/logo_128.png)

---

## ⚡ The Flow

```text
       Ctrl + Space
            ↓
    Type what you need
            ↓
       Press Enter
            ↓
          Done.
```

No complicated syntax to memorize. No dashboards to navigate. No waiting.  
Spotlight-Win stays out of your way until you summon it, does the obvious thing, and disappears.

---

## 🎯 Just Start Typing

Spotlight-Win understands what you mean without requiring command prefixes:

| What you type | What Spotlight does |
|---|---|
| `chrome` | Launches **Google Chrome** |
| `code` or `vscode` | Launches **Visual Studio Code** |
| `project report` | Opens your recent document |
| `45 * 12` or `sqrt(144)` | Instantly evaluates the math |
| `100 usd to eur` | Live currency conversion |
| `github.com` | Opens your browser |
| `task manager` | Launches Windows Task Manager |
| `sleep` or `lock` | Puts PC to sleep or locks screen |

---

## ✨ The Core Experience

### 🚀 1. Instant App & File Launching
Search across installed applications, system tools, documents, downloads, code files, and folders with sub-15ms latency. Deep directory results clearly show parent folder context (e.g., `Projects > my-app`).

### 🧮 2. Ambient Math & Calculations
Evaluate math expressions directly inline—supporting arithmetic, powers (`2^8`), and standard functions (`sqrt`, `sin`, `cos`, `abs`, `log`).

### 💱 3. Currency Conversion
Type conversions naturally (e.g. `250 usd to inr`, `50 gbp to jpy`). Rates update automatically in the background and are cached offline for instant response.

### 🌐 4. Web Shortcuts & Aliases
Type URLs directly or create custom short aliases (e.g. `yt` &rarr; `https://youtube.com`, `gh` &rarr; `https://github.com`) to launch favorite destinations with a few keystrokes.

### 🛡️ 5. Zero-Telemetry Local Privacy
- **100% Offline-First**: No web telemetry, no tracking, and zero Bing scraper ads.
- **Ultra-Lightweight**: Idles at <30 MB RAM with <1% background CPU impact.
- **Safe Execution**: Uses native Windows `ShellExecuteW` to launch apps and files with their default OS-registered associations.

---

## ⌨️ Minimalist Keyboard Interaction

We follow a strict interaction philosophy: **One search box. One obvious answer.**

| Key | Action |
|---|---|
| <kbd>Ctrl</kbd> + <kbd>Space</kbd> | Summon or hide Spotlight |
| <kbd>Enter</kbd> | Do the obvious thing (open top result) |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Move selection |
| <kbd>Shift</kbd> + <kbd>Enter</kbd> | Reveal selected file in Windows Explorer |
| <kbd>Alt</kbd> + <kbd>Enter</kbd> | Forget item from recent history |
| <kbd>Esc</kbd> | Get me out (close window) |

*(Power-user escape hatches like `app:`, `file:`, `folder:`, and `sys:` exist if you ever want to force-filter, but you never need them for everyday work.)*

---

## 🏗️ Technical Architecture

Spotlight-Win combines a responsive web frontend with a high-performance native Rust core via Tauri v2:

- **Sub-Microsecond Scoring**: Ranking combines fuzzy text matching, direct prefix bonuses, app prioritization, launch frequency, and time-of-day relevance in memory.
- **In-Memory Caching**: Extracted Windows binary icons and file records are kept in memory with disk persistence, avoiding disk I/O during keystrokes.
- **Debounced Incremental Watcher**: Real-time filesystem changes (Desktop, Start Menu, User Folders) update the in-memory cache dynamically without SSD thrashing.
- **Failsafe System Actions**: Destructive actions (shutdown, restart) require deliberate keywords and trigger confirmation prompts before execution.

---

## 🛠️ Getting Started & Building

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.78+)
- [Node.js](https://nodejs.org/) (v18+) & `npm`

### Local Development
```bash
git clone https://github.com/Snaehath/Win-Spotlight.git
cd Win-Spotlight
npm install
npm run tauri dev
```

### Build Installer
```bash
npm run build:release
```
Standalone setup files and binaries will be generated in `src-tauri/target/release/bundle/`.

---

## 🎯 v1.0 Production Hardening Roadmap

Rather than adding endless features, our current milestone is focused entirely on **production hardening, reliability, and friction removal**:

- [ ] **Search Ranking Perfection**: Continuous tuning so the top result is undeniably what you meant on the first keystroke.
- [ ] **Edge-Case Resilience**: Seamless behavior across multi-monitor setups, high-DPI scaling (125%, 150%, 200%), and Windows sleep/wake cycles.
- [ ] **Zero-Friction Installer**: Smooth one-click install/uninstall experience for everyday users.
- [ ] **High-Scale Indexing**: Everything SDK / USN Journal integration to scale across 1M+ files with zero UI latency.
- [ ] **Clipboard History**: Clean, unobtrusive search-and-paste clipboard workflow.

---

## 🤝 Feedback & Suggestions

Spotlight-Win is built to give Windows users the clean, blazing-fast search experience they deserve.

If you encounter unexpected results, have ideas on ranking tuning, or want to suggest improvements:
- Open an [Issue](https://github.com/Snaehath/Win-Spotlight/issues) for bug reports or ranking feedback.
- Join the [Discussions](https://github.com/Snaehath/Win-Spotlight/discussions) to share your daily workflow impressions.

---

Reclaim your desktop. Stay in flow.
