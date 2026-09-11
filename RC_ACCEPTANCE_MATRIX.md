# Spotlight-Win Release Candidate (RC) Acceptance Matrix

> **Feature Freeze Policy Active:**
> *No new features unless they resolve a demonstrated usability or reliability defect.*

This document defines the formal exit criteria for **Spotlight-Win v1.0 Release Candidate**. A build may only be deemed production-ready once all acceptance criteria are verified on clean Windows installations.

---

## 📋 Acceptance Criteria Matrix

### 1. Runtime & Latency
- [x] **Hotkey Reliability**: <kbd>Ctrl</kbd> + <kbd>Space</kbd> reliably summons the launcher window in < 50ms from warm state.
- [x] **First Keystroke Acceptance**: The first character typed immediately following hotkey invocation is never dropped or delayed.
- [ ] **Warm Latency SLA**:
  - [ ] p50 < 50 ms (keypress to UI paint)
  - [ ] p95 < 100 ms
  - [ ] p99 < 150 ms
- [x] **Burst Typing Concurrency**: Rapid continuous typing (`c → ch → chr → chrom → chrome`) discards obsolete asynchronous responses via query ID tracking without UI flicker or stale results.
- [x] **Single Activation Invariant**:
  - [x] Holding <kbd>Enter</kbd> launches the selected target at most once (OS `e.repeat` discarded).
  - [x] Rapid physical double-tapping of <kbd>Enter</kbd> launches at most once (`isActivating` session lock).
  - [x] Opening a new launcher session (<kbd>Ctrl</kbd> + <kbd>Space</kbd> / `window-shown`) cleanly resets the lock.
- [x] **Intentional Multi-Instance Invariant**:
  - [x] First launch of an application opens immediately with zero prompt.
  - [x] Subsequent launch of an already-running application displays a confirmation prompt: *"Do you want to open another [App] window / instance?"* with `[Cancel]` and `[Open another]`.
  - [x] <kbd>Enter</kbd> confirms "Open another"; <kbd>Esc</kbd> cancels and refocuses search input without spawning duplicate instances.
  - [x] Non-app targets (URLs, files, folders, calculators, system commands) bypass running-process checks entirely.

---

### 2. Search Correctness & Relevance
- [x] **Exact Match Priority**: Exact application name match strictly tops results over partial, prefix, or fuzzy matches.
- [x] **Prefix over Fuzzy**: Substring prefix matches always score above mid-string or fuzzy matches.
- [x] **Acronym Matching**: Short acronym queries (`vs` $\to$ Visual Studio Code, `cmd` $\to$ Command Prompt) score deterministically.
- [x] **Folder Search Cleanliness**: Paths containing `.`, `_`, or standard directory separators display clean, natural folder names.
- [x] **Bounded Candidate Invariant**: Single-letter queries (`a`, `c`, `s`, `m`) remain bounded ($\le 48$ Tantivy candidates) without unbounded allocations or latency spikes.
- [x] **Focused Web & URL Action**: Entering full or partial URLs (`https://`, `.com`) surfaces a single clear browser navigation action.
- [x] **Math & Unit Intent**: Valid mathematical expressions (`128 * 4`, `sqrt(144)`) evaluate inline without external subprocess overhead.
- [x] **Destructive Safeguards**: Critical system actions (`shutdown`, `restart`) require explicit keywords and prompt confirmation dialogs before execution.

---

### 3. Resilience & Crash Recovery
- [ ] **Missing Index**: Wiping `%APPDATA%/com.spotlight.launcher/spotlight_index` prompts transparent, silent background index regeneration.
- [ ] **Corrupt Index Recovery**: Writing invalid bytes into `meta.json` or segments recovers automatically without crashing or showing error dialogs.
- [ ] **Stale Process Locks**: Dead process `.lock` files in Tantivy directory are cleaned and healed on launch.
- [x] **User State Preservation**: Corrupt `history_v2.json` creates a timestamped `history_v2.corrupt.<timestamp>.json` backup before resetting in-memory records.
- [ ] **Hard Process Kill During Indexing**: Terminating the process mid-write (`taskkill /f /im spotlight-win.exe`) results in automatic self-healing on next start.
- [ ] **Hard Process Kill During Launch**: Process termination during application launch leaves no corrupt lock states.

---

### 4. Windows OS Integration & UX
- [ ] **Multi-Monitor Layout**: The launcher centers cleanly on the active monitor where the mouse cursor resides.
- [ ] **DPI & Scaling**: Renders crisbly across 100%, 125%, 150%, and 200% Windows display scaling settings.
- [ ] **Focus Restoration**: Dismissing the launcher (<kbd>Escape</kbd> or blur) restores keyboard focus to the previously active foreground window.
- [ ] **Explorer Reveal**: <kbd>Shift</kbd> + Click / Reveal button opens File Explorer with the targeted file or folder selected.
- [ ] **System Tray & Hotkeys**: Tray icon menu (Show, Settings, Exit) functions reliably; global hotkey survives explorer.exe restarts.
- [ ] **Sleep / Wake Cycles**: Launcher remains responsive and hotkey remains registered after PC resumes from sleep or hibernation.
- [ ] **Lock / Unlock Cycles**: Launcher state is maintained cleanly across Windows lock (<kbd>Win</kbd> + <kbd>L</kbd>) and unlock events.
- [ ] **Path Edge Cases**: Correctly opens and reveals paths with spaces (`C:\Program Files\My App\app.exe`) and Unicode characters.

---

### 5. Packaging & Installer Lifecycle
- [ ] **Clean Installation**: Both NSIS Setup (`.exe`) and Windows Installer (`.msi`) install silently or via wizard without antivirus false-positives.
- [ ] **Startup Registration**: Toggleable "Launch on Windows Startup" creates standard registry or startup folder shortcut.
- [ ] **Clean Upgrade**: Installing a newer version over an existing version preserves user configuration and launch history.
- [ ] **Clean Uninstall**: Uninstaller cleanly removes binary files, Start Menu shortcuts, and registry keys.
- [ ] **Release Profile Verification**: All RC tests run against optimized production builds (`npm run build:release`).

---

## ⏱️ 24–72 Hour Windows Soak Test Protocol

To validate background stability under realistic Windows desktop workloads:

### Operational Scenario
Leave Spotlight-Win running continuously in the system tray for **24 to 72 hours** while conducting regular daily desktop workflows:
1. Launch applications, open nested folders, execute system commands.
2. Sleep and wake the PC across overnight cycles.
3. Install, update, and remove desktop software.
4. Create, rename, edit, and delete files on Desktop, Documents, and Downloads.

### Inspection Metrics Checklist
- [ ] **Memory Stability**: Steady-state RAM remains stable (< 85 MB idle RSS; no monotonic growth).
- [ ] **Idle CPU Usage**: 0.0% CPU consumption while idle in system tray.
- [ ] **Watcher Efficiency**: Filesystem events batch-debounce cleanly without triggering CPU spikes or disk thrashing.
- [ ] **Index Stability**: Tantivy index size remains compact and does not bloat uncontrollably.
- [ ] **History Stability**: `history_v2.json` stays cleanly serialized and within size bounds (< 500 KB).
- [ ] **Zero Crash Invariant**: Zero uncaught exceptions, panics, or silent process terminations.
