# 🛰️ Spotlight-Win: Elite Insight Engine

> **A blazing-fast, privacy-first, desktop launcher and insight engine for Windows.**  
> Built with Rust, Tauri v2, and modern glassmorphic web aesthetics.

![Spotlight-Win Banner](public/logo_128.png)

---

## ⚡ What is Spotlight-Win?

**Spotlight-Win** is a lightweight, keyboard-first desktop productivity utility designed to replace slow, ad-cluttered OS search bars with instantaneous local results, ambient utilities, and native privacy.

- **Instant Results**: Sub-15ms search response powered by Rust and multi-tiered in-memory caching.
- **100% Private & Offline**: Zero web telemetry, zero background data mining, and zero Bing advertisements.
- **Ultra-Lightweight**: Minimal memory footprint (<30 MB idle) and <1% background CPU usage.
- **Desktop Jewelry**: Polished translucent glassmorphic interface with keyboard-first navigation.

---

## ✨ Features at a Glance

### 🔍 1. Smart Universal Search

- **Instant App & Tool Launch**: Quickly launch installed applications, Windows utilities (Task Manager, Registry Editor, PowerShell, Control Panel), and custom shortcuts.
- **Broad File & Document Indexing**: Search across Documents, Downloads, Desktop, Code, Archives, Media, and Folders with category-specific badges and icons.
- **Parent > Child Breadcrumbs**: Deep directory results clearly show parent folder context (e.g., `Projects > my-app`).
- **Keyword Filtering**: Narrow down results instantly by prefixing your query with `app:`, `file:`, `folder:`, or `command:`.

### 🧮 2. Ambient Math & Calculations

- Type any math expression directly into the search bar without special prefixes (e.g. `(45 * 12) / 4`, `sqrt(144)`, `2^8`, `sin(90)`).
- Instant inline calculation results ready to view or launch into Calculator.

### 💱 3. Live & Offline Currency Converter

- Natural language currency conversions (e.g. `100 usd to eur`, `5000 inr to usd`, `50 gbp to jpy`).
- Automatic background rate refresh with smart offline caching to keep conversions instant.

### 📊 4. System HUD & Power Controls

- **System Stats HUD**: Type `sys:` or `system` to view real-time CPU load, RAM usage, and available disk partition space.
- **Quick System Actions**: Fast command aliases like `> sys lock`, `> sys sleep`, `> sys restart`, and `> sys shutdown` with safety confirmation dialogs.

### 🌐 5. Custom Web Shortcuts & Aliases

- Open any URL directly by typing it (e.g. `github.com` or `news.ycombinator.com`).
- Save custom aliases on the fly (e.g. `yt` &rarr; `https://youtube.com`) to launch favorite tools in one keystroke.

### 👻 6. Ghost Actions & History Management

- **Reveal in Explorer**: Select any item and press <kbd>Shift</kbd> + <kbd>Enter</kbd> (or click the reveal button) to open its exact directory in Windows Explorer.
- **Forget from History**: Press <kbd>Alt</kbd> + <kbd>Enter</kbd> (or click the trash icon) to instantly remove unwanted items from recent history.

---

## ⌨️ Shortcuts & Command Reference

| Action                    | Input / Shortcut                    | Description                                      |
| ------------------------- | ----------------------------------- | ------------------------------------------------ |
| **Toggle Spotlight**      | <kbd>Ctrl</kbd> + <kbd>Space</kbd>  | Global hotkey to summon or hide the search bar   |
| **Open / Launch**         | <kbd>Enter</kbd>                    | Launch the selected application, file, or URL    |
| **Reveal in Explorer**    | <kbd>Shift</kbd> + <kbd>Enter</kbd> | Highlight file in its containing folder          |
| **Forget from History**   | <kbd>Alt</kbd> + <kbd>Enter</kbd>   | Remove selected item from recents                |
| **Dismiss / Close**       | <kbd>Esc</kbd>                      | Hide the launcher window                         |
| **Filter by Application** | `app: <query>`                      | Filter search to applications only               |
| **Filter by File**        | `file: <query>`                     | Filter search to documents and files             |
| **Filter by Folder**      | `folder: <query>`                   | Filter search to directory paths                 |
| **System HUD**            | `sys:` or `system`                  | Live CPU, RAM, and Disk space gauges             |
| **Web Search**            | `> g <query>`                       | Perform a quick Google search in default browser |
| **Help Manual**           | `/help`                             | Open the interactive in-app manual               |

---

## 🏗️ Architecture & Performance

```
┌─────────────────────────────────────────────────────────────┐
│                      Spotlight-Win                          │
│                                                             │
│   Frontend (Vanilla CSS + HTML5 + JS)                       │
│   ├── Glassmorphic Translucent Shell                        │
│   ├── Debounced Input Stream (150ms)                        │
│   └── Keyboard Event Traps & Selection                      │
│                                                             │
│   Backend (Tauri v2 + Rust)                                 │
│   ├── In-Memory Item Cache (Atomic Arc<Mutex>)              │
│   ├── Zero-Clone History & Adaptive Time Scoring            │
│   ├── In-Memory Icon Cache (RwLock + Disk Hash Map)         │
│   ├── Incremental File Watcher (notify-debouncer-mini)      │
│   └── Native Shell Execution (Win32 ShellExecuteW)          │
└─────────────────────────────────────────────────────────────┘
```

- **Zero-Clone Scoring**: Scoring loops operate directly against in-memory timestamps and launch counts without expensive serialization or disk queries.
- **Safe Native Execution**: Built around Windows `ShellExecuteW` to launch apps and files cleanly with their OS-registered associations.
- **Batched Watcher I/O**: The file watcher batches filesystem events with debouncing, avoiding disk thrashing while keeping indices fresh.

---

## 🛠️ Development & Building

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.78+ recommended)
- [Node.js](https://nodejs.org/) (v18+ recommended) & `npm`

### Local Development

Clone the repository and install frontend dependencies:

```bash
git clone https://github.com/Snaehath/Win-Spotlight.git
cd Win-Spotlight
npm install
npm run tauri dev
```

### Production Build

To create a standalone release executable and installer:

```bash
npm run build:release
```

Bundled binaries and setup files will be generated in `src-tauri/target/release/bundle/`.

---

## 💡 Suggestions, Feedback & Roadmap

We are continuously refining Spotlight-Win to make it the fastest and most elegant desktop companion for Windows power users. **Your suggestions, feature requests, and improvements are warmly welcomed!**

### 🔮 Ideas Under Consideration / Roadmap

- [ ] **Everything SDK / USN Journal Integration**: Instant whole-drive NTFS indexing for indexing millions of files in seconds.
- [ ] **Clipboard History & Snippets**: Seamless clipboard manager with quick paste support.
- [ ] **Extensible Plugin API**: Community plugins for Spotify playback control, window management, GitHub issues, and developer documentation search.
- [ ] **Configurable Keybindings & Custom Themes**: In-app settings to customize the summon shortcut, glassmorphism blur intensity, and theme accents.
- [ ] **Quick Notes & Scratchpad**: Instant floating markdown scratchpad summoned with a keystroke.

### 🤝 How to Share Suggestions

- Open an [Issue](https://github.com/Snaehath/Win-Spotlight/issues) for bug reports or feature proposals.
- Start a [Discussion](https://github.com/Snaehath/Win-Spotlight/discussions) to share ideas on UX, performance, or new plugins.
- Pull requests are always welcome!

---

Reclaim your desktop. Stay in flow.
