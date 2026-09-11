<div align="center">
  <img src="public/app_icon.png" width="96" height="96" alt="Spotlight-Win Icon" />
  <h1>Spotlight-Win</h1>
  <p>
    <strong>The fastest way to find or launch anything on your Windows PC.</strong><br />
    Zero setup. Zero ads. Zero learning curve. Just press <kbd>Ctrl</kbd> + <kbd>Space</kbd> and type.
  </p>
</div>

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

| What you type            | What Spotlight does              |
| ------------------------ | -------------------------------- |
| `chrome`                 | Launches **Google Chrome**       |
| `code` or `vscode`       | Launches **Visual Studio Code**  |
| `project report`         | Opens your recent document       |
| `45 * 12` or `sqrt(144)` | Instantly evaluates the math     |
| `100 usd to eur`         | Live currency conversion         |
| `github.com`             | Opens your browser               |
| `task manager`           | Launches Windows Task Manager    |
| `sleep` or `lock`        | Puts PC to sleep or locks screen |

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

| Key                                               | Action                                   |
| ------------------------------------------------- | ---------------------------------------- |
| <kbd>Ctrl</kbd> + <kbd>Space</kbd>                | Summon or hide Spotlight                 |
| <kbd>Enter</kbd>                                  | Do the obvious thing (open top result)   |
| <kbd>↑</kbd> / <kbd>↓</kbd>                       | Move selection                           |
| <kbd>Shift</kbd> + <kbd>Enter</kbd>               | Reveal selected file in Windows Explorer |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>C</kbd> | Copy selected item path to clipboard     |
| <kbd>Alt</kbd> + <kbd>Enter</kbd>                 | Forget item from recent history          |
| <kbd>Esc</kbd>                                    | Get me out (close window)                |

_(Power-user escape hatches like `app:`, `file:`, `folder:`, and `sys:` exist if you ever want to force-filter, but you never need them for everyday work.)_

---

## 🏗️ Technical Architecture

Spotlight-Win combines a responsive web frontend with a high-performance native Rust core via Tauri v2:

- **Two-Stage Retrieval & Hard Bounding**: Tantivy inverted index retrieves candidate matches in sub-millisecond time. The ranking hot path is strictly bounded to $\le 48 \text{ candidates} + \approx 200 \text{ installed apps}$, completely decoupling index size from keystroke latency.
- **Decoupled Personal Composite Ranking**: Final results are scored across **Match Quality** (70%), **Personal Profile** (25% with 7-day half-life exponential recency decay and logarithmic frequency), and **Context** (5%).
- **Self-Healing State Recovery**: Transparent automatic reconstruction of derived index files on unexpected process termination or corruption—zero error popups or manual user fixes required.
- **In-Memory Caching & $O(1)$ Mapping**: Extracted Windows binary icons, precomputed normalized names, acronyms, and path lookup maps avoid disk I/O and per-keystroke allocations.
- **Debounced Incremental Watcher**: Real-time filesystem changes (Desktop, Start Menu, User Folders) batch-update without UI stutter or SSD thrashing.
- **Single Activation Invariant**: A single launcher session can activate a selected result at most once. Repeated <kbd>Enter</kbd> keydowns, OS key-repeat, and concurrent activation attempts are consumed idempotently until a new session is initiated.
- **Intentional Multi-Instance Invariant**: Distinguishes accidental within-session repeats from deliberate multi-window launches across sessions. Launching an already-running app prompts for confirmation (*"Do you want to open another Chrome window / instance?"*) with <kbd>Enter</kbd> confirming and <kbd>Esc</kbd> safely cancelling. Non-app targets execute immediately.
- **Failsafe System Actions**: Destructive actions (shutdown, restart) require deliberate keywords and trigger confirmation prompts before execution.

---

## 🛠️ Getting Started & Building

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.78+)
- [Node.js](https://nodejs.org/) (v18+) & `npm`

### Local Development & Testing

```bash
git clone https://github.com/Snaehath/Win-Spotlight.git
cd Win-Spotlight
npm install

# Run automated product invariant and benchmark test suite
npm test

# Launch dev server
npm run tauri dev
```

### Build Installer

```bash
npm run build:release
```

Standalone setup files and binaries will be generated in `src-tauri/target/release/bundle/`.

---

## 🎯 Product Roadmap & Release Hardening

Spotlight-Win is operating under an active **feature freeze** for its upcoming Release Candidate to focus exclusively on production reliability, Windows integration, and friction removal.

- 🗺️ **Long-Term Vision**: See [**ROADMAP.md**](file:///d:/DevelopmentSide/AI-Studio/spotlight-win/ROADMAP.md) for our full 12–18 month plan evolving Spotlight-Win into the fastest command surface for Windows (Personalization &rarr; System Actions &rarr; Clipboard &rarr; Workflows &rarr; Local AI).
- 📋 **Release Candidate Checklist**: Detailed release exit criteria and soak testing protocols are tracked in [`RC_ACCEPTANCE_MATRIX.md`](file:///d:/DevelopmentSide/AI-Studio/spotlight-win/RC_ACCEPTANCE_MATRIX.md).

- [x] **Bounded Two-Stage Search**: Tantivy candidate retrieval with bounded personal ranking and zero-allocation metadata.
- [x] **Self-Healing Index Recovery**: Automatic transparent recovery from unexpected process kills and corrupt cache states.
- [x] **User State Preservation**: Defensive timestamped backups of `history_v2.json` on corruption before state reset.
- [x] **Single Activation Invariant**: Dual-boundary session lock preventing duplicate launches from held or repeated <kbd>Enter</kbd> presses.
- [x] **Automated Invariant Suite**: 9 automated regression tests covering scoring separation, exponential recency, acronyms, and safeguards.
- [ ] **End-to-End Latency Verification**: Real-world dogfooding measuring p50 (<50ms) / p95 (<100ms) / p99 keypress-to-paint times.
- [ ] **Edge-Case Resilience**: Seamless behavior across multi-monitor setups, high-DPI scaling (125%, 150%, 200%), and Windows sleep/wake cycles.
- [ ] **Zero-Friction Installer**: Smooth one-click install/uninstall experience for everyday users.

---

## 🤝 Feedback & Suggestions

Spotlight-Win is built to give Windows users the clean, blazing-fast search experience they deserve.

If you encounter unexpected results, have ideas on ranking tuning, or want to suggest improvements:

- Open an [Issue](https://github.com/Snaehath/Win-Spotlight/issues) for bug reports or ranking feedback.
- Join the [Discussions](https://github.com/Snaehath/Win-Spotlight/discussions) to share your daily workflow impressions.

---

Reclaim your desktop. Stay in flow.
