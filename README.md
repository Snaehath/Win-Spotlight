<div align="center">
  <img src="public/app_icon.png" width="80" height="80" alt="Spotlight-Win icon" />

  # Spotlight-Win

  A fast, keyboard-first launcher for Windows.

  Press <kbd>Ctrl</kbd> + <kbd>Space</kbd>, type what you need, and press <kbd>Enter</kbd>.
</div>

## Features

- Search and launch installed apps, files, folders, and system tools.
- Use quick filters for applications, folders, files, and commands.
- Calculate expressions and convert currencies from the search bar.
- Open URLs and create custom web shortcuts.
- Keep search and history data on your PC.

## Keyboard shortcuts

| Shortcut | Action |
| --- | --- |
| <kbd>Ctrl</kbd> + <kbd>Space</kbd> | Show or hide Spotlight-Win |
| <kbd>Enter</kbd> | Open the selected result |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Move through results |
| <kbd>Shift</kbd> + <kbd>Enter</kbd> | Reveal a file in Explorer |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>C</kbd> | Copy the selected path |
| <kbd>Ctrl</kbd> + <kbd>Enter</kbd> | Save a typed URL as a named web shortcut |
| <kbd>Alt</kbd> + <kbd>Enter</kbd> | Forget an item's launch history |
| <kbd>F1</kbd> | Show all shortcuts and commands |
| <kbd>Esc</kbd> | Hide the launcher (or cancel naming a shortcut) |

Filters include `app:`, `folder:`, `file:`, and `command:`. Type a URL to open it in your browser. Type `> sys` to see system actions. Pressing <kbd>Enter</kbd> on a calculation copies its result.

## Install

Download the Windows installer from [GitHub Releases](https://github.com/Snaehath/Win-Spotlight/releases) and run it. Press <kbd>Ctrl</kbd> + <kbd>Space</kbd> to open the launcher.

## Build from source

You need [Rust](https://www.rust-lang.org/tools/install), [Node.js](https://nodejs.org/) 18 or later, and npm.

```bash
git clone https://github.com/Snaehath/Win-Spotlight.git
cd Win-Spotlight
npm install
npm run tauri dev
```

To build the Windows installers:

```bash
npm run build:release
```

Installers are written to `src-tauri/target/release/bundle/`.

## Feedback

Report a bug or request a feature in [GitHub Issues](https://github.com/Snaehath/Win-Spotlight/issues).
