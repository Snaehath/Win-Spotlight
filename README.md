<div align="center">
  <img src="public/app_icon.png" width="88" height="88" alt="Spotlight-Win icon" />

  <h1>Spotlight-Win</h1>

  <p><strong>A fast, keyboard-first launcher for Windows.</strong><br />
  Press <kbd>Ctrl</kbd> + <kbd>Space</kbd>, type a few letters, press <kbd>Enter</kbd>.</p>

  <p>
    <a href="https://github.com/Snaehath/Win-Spotlight/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/Snaehath/Win-Spotlight?include_prereleases&label=release&color=2f6fe4" /></a>
    <img alt="Platform: Windows 10 and 11" src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-2f6fe4" />
    <img alt="Built with Rust and Tauri 2" src="https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri%202-2f6fe4" />
  </p>

  <p>
    <a href="https://github.com/Snaehath/Win-Spotlight/releases/latest"><strong>Download</strong></a> ·
    <a href="https://snaehath.github.io/Win-Spotlight/">Website</a> ·
    <a href="#what-you-can-type">What you can type</a> ·
    <a href="#keyboard-shortcuts">Shortcuts</a> ·
    <a href="#privacy">Privacy</a> ·
    <a href="#build-from-source">Build</a>
  </p>
</div>

---

Spotlight-Win is a small native launcher that lives in your system tray. It finds your apps, files and folders as you type, does quick maths and currency conversions, opens websites, and runs system actions, without leaving the keyboard. It's written in Rust on [Tauri 2](https://tauri.app/) and keeps its search index on your PC.

## Features

- **Instant app search.** Every installed app from the Start Menu, desktop shortcuts and Store app aliases. Duplicate shortcuts for the same app are merged into one result.
- **Files and folders.** Searches the names of files and folders anywhere in your Desktop, Downloads, Documents, Pictures, Videos and Music, including deep subfolders and OneDrive-redirected folders, plus the top levels of your other drives. Documents, images, slides, spreadsheets, audio, video, archives and code are all recognised. New files in your folders appear within seconds. Library folders such as `node_modules`, `site-packages` and `.git` are skipped to keep results clean.
- **Results that learn.** Apps you open often, and at this time of day, rise to the top. The best match is shown as a **Top Hit**, and the rest are grouped by type.
- **Typo-tolerant matching.** Prefix, acronym (`vsc` → Visual Studio Code) and fuzzy matching.
- **Calculator.** Type `12*(3+4)` or `sqrt(144)` and the answer appears instantly. Press <kbd>Enter</kbd> to copy it.
- **Currency conversion.** `100 usd to eur`, `€50 in inr`, `25 gbp`.
- **Web.** Type a URL to open it, save it as a short alias (`gh` → github.com), or search with `> g your query`.
- **System actions.** Lock, sleep, restart and shut down, each with a confirmation first. Type `system` to see CPU, memory and disk usage.
- **Quick filters.** `app:`, `file:`, `folder:` and `command:`, or click the round buttons next to the search bar.
- **Native look.** A frosted, rounded design that follows your Windows light or dark theme.

## What you can type

| Type this | You get |
| --- | --- |
| `code`, `vsc`, `chrome` | Apps, ranked by how often you use them |
| `invoice`, `folder:projects` | Matching files or folders |
| `45 * 12`, `2^10`, `sqrt(2)` | The result, ready to copy |
| `100 usd to eur`, `$20 in inr` | A live currency conversion |
| `github.com` | Open the site, or press <kbd>Ctrl</kbd> + <kbd>Enter</kbd> to save it as a shortcut |
| `> g rust lifetimes` | A Google search in your browser |
| `lock`, `sleep`, `restart`, `shutdown` | A system action (asks for confirmation first) |
| `system` | CPU, memory and disk usage |
| `/help` or <kbd>F1</kbd> | The built-in shortcut reference |

## Keyboard shortcuts

| Shortcut | Action |
| --- | --- |
| <kbd>Ctrl</kbd> + <kbd>Space</kbd> | Show or hide the launcher |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Move through results |
| <kbd>Enter</kbd> | Open the selected result (or copy a calculation) |
| <kbd>Shift</kbd> + <kbd>Enter</kbd> | Reveal the file or folder in Explorer |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>C</kbd> | Copy the selected path or URL |
| <kbd>Ctrl</kbd> + <kbd>Enter</kbd> | Save a typed URL as a named web shortcut |
| <kbd>Alt</kbd> + <kbd>Enter</kbd> | Forget an item's launch history |
| <kbd>Backspace</kbd> on an empty box | Remove the active filter |
| <kbd>F1</kbd> | Show all shortcuts and commands |
| <kbd>Esc</kbd> | Hide the launcher (or cancel naming a shortcut) |

Right-click the tray icon to **pause the global shortcut** (useful if a game or another app needs <kbd>Ctrl</kbd> + <kbd>Space</kbd>) or to quit.

## Install

1. Download the latest installer from [**GitHub Releases**](https://github.com/Snaehath/Win-Spotlight/releases/latest).
2. Run it. Spotlight-Win starts in the system tray and launches automatically when you sign in.
3. Press <kbd>Ctrl</kbd> + <kbd>Space</kbd>.

**Requirements:** Windows 10 or 11 (64-bit) with Microsoft Edge WebView2. WebView2 is built into Windows 11, and the installer adds it on Windows 10 if it's missing.

## Privacy

Spotlight-Win has no accounts, ads or telemetry, and it never sends your searches or file names anywhere.

- **The index stays on your PC.** File names, launch history and saved shortcuts are kept in `%APPDATA%\com.spotlight.launcher\`.
- **One network request, only when you ask for it.** When you type a currency conversion, the app fetches exchange rates from [frankfurter.app](https://www.frankfurter.app/), a free service publishing European Central Bank rates. Rates are cached for an hour, and no query text is sent.
- **Your browser handles the web.** Opening a URL or running `> g` hands the link to your default browser.

To reset everything, quit the app from the tray and delete `%APPDATA%\com.spotlight.launcher\` and `%APPDATA%\spotlight\launcher\`.

## Troubleshooting

| Problem | Fix |
| --- | --- |
| *"The Ctrl+Space shortcut is already in use"* on start | Another copy of Spotlight-Win, or another app, owns the shortcut. Quit the other copy from the tray, or close the app that uses <kbd>Ctrl</kbd> + <kbd>Space</kbd>. |
| A file doesn't show up | Files in your user folders appear within a few seconds. Folders elsewhere on your drives are scanned a few levels deep at start-up. Files inside hidden folders (starting with `.`) and library folders (`node_modules`, `site-packages`, `build`, …) are skipped on purpose. |
| An old item keeps ranking high | Select it and press <kbd>Alt</kbd> + <kbd>Enter</kbd> to forget its history. |

## Build from source

You need [Rust](https://www.rust-lang.org/tools/install) (stable), [Node.js](https://nodejs.org/) 18 or later, and the [Tauri prerequisites for Windows](https://tauri.app/start/prerequisites/) (Microsoft C++ Build Tools and WebView2).

```bash
git clone https://github.com/Snaehath/Win-Spotlight.git
cd Win-Spotlight
npm install
npm run tauri dev       # run with hot reload of the frontend
npm test                # run the Rust test suite
npm run build:release   # build the installers
```

Installers are written to `src-tauri/target/release/bundle/` (`nsis/` for the `.exe` setup and `msi/` for the MSI).

### Project layout

```text
src/                  Frontend: plain HTML, CSS and ES modules (no build step)
  main.js             State, keyboard handling, actions
  ui.js               Result rendering
  dialog.js           Confirmation dialog
  styles.css          Design tokens and light/dark themes
src-tauri/src/
  main.rs             App setup, tray, global shortcut, window commands
  search.rs           Query pipeline: filters, candidates, ranking, intents
  index_engine.rs     Tantivy full-text index of names
  indexer/            Filesystem scan, app identity merging, icon cache
  watcher.rs          Live filesystem updates
  ranking.rs          Frequency, recency and time-of-day scoring
  commands.rs         `>` commands: calc, web search, system actions
  currency.rs         Currency detection and conversion
  system_info.rs      CPU, memory and disk stats
  launcher.rs         Launching, reveal in Explorer, running-app check
```

## Feedback

Found a bug or have an idea? Open an issue on [GitHub Issues](https://github.com/Snaehath/Win-Spotlight/issues).
