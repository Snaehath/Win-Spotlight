# 🚀 Spotlight-Win v0.4.0

**The Speed of Thought. The Power of Rust.**

Spotlight-Win is a high-performance, elite Windows command launcher built for users who demand sub-millisecond precision, stunning aesthetics, and habit-aware intelligence. Built with **Rust** and **Tauri**, it transforms your desktop into a unified, glassmorphic workstation.

---

### 📊 The Performance Gap

| Metric              | Windows Search          | Spotlight-Win                    |
| :------------------ | :---------------------- | :------------------------------- |
| **Query Latency**   | ~200ms - 500ms          | **< 1ms (Instant)**              |
| **Indexing Engine** | Native Windows Indexer  | **Tantivy (High-Perf Rust)**     |
| **Search Logic**    | Basic Prefix Matching   | **Adaptive Habit-Aware + Fuzzy** |
| **Resource Impact** | High Background CPU/RAM | **Ultra-Low Memory Footprint**   |
| **Privacy**         | Cloud/Web Integrated    | **100% Local / Zero Tracking**   |

---

### ⚡ Hero Features

- **🚀 Instant Everything**: Sub-1ms search and retrieval powered by the **Tantivy** full-text search engine. Launch apps, files, and commands at the speed of thought.
- **🔗 Custom Web Shortcuts**: Turn the web into your command line. Assign aliases (e.g., `gh` → `github.com`) to your favorite sites for instant, browser-bridged navigation.
- **🧠 Habit-Aware Intelligence**: Spotlight-Win learns your workflow. Our adaptive ranking engine prioritizes results based on your time-of-day habits and launch frequency.
- **💎 Elite Design System**: A stunning **Glassmorphism** interface featuring backdrop blur, vibrant design tokens, and micro-staggered entry animations.
- **🌍 Intelligent Ambient Search**:
  - **Math**: Instant evaluation (e.g., `5 * (10/2)`).
  - **Currency**: Live global conversion (e.g., `100 USD to EUR`).
  - **System Control**: Secure, confirmed execution for `shutdown`, `restart`, and `lock`.
- **🎨 High-Fidelity Iconography**: Integrated [Lucide Icons](https://lucide.dev/) for a consistent, professional vector aesthetic.

---

### 🛡️ Technical Pedigree

Spotlight-Win is engineered for the modern era, prioritizing security and efficiency:

- **Built with Rust**: Memory-safe performance that ensures zero-latency interaction.
- **Tauri 2.0**: The ultimate lightweight framework, resulting in a tiny memory footprint and native Windows integration.
- **Modular Architecture**: A robust, multi-module backend designed for infinite extensibility.

## 🛠️ Tech Stack

- **Backend:** [Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/)
- **Search Engine:** [Tantivy v0.22](https://github.com/quickwit-oss/tantivy)
- **Frontend:** Modular ES6 JavaScript + Vanilla CSS3 (Design Tokens)
- **Architecture:** Multi-module Rust backend (11+ crates) and Componentized Frontend.

## ⌨️ Shortcuts

| Action               | Shortcut                  |
| :------------------- | :------------------------ |
| **Toggle Launcher**  | `Ctrl + Space`            |
| **Navigate Results** | `Arrow Up` / `Arrow Down` |
| **Launch / Action**  | `Enter`                   |
| **Hide Launcher**    | `Escape`                  |

## 🏗️ Project Structure

### Backend (Rust)

- `src-tauri/src/index_engine/`: Persistent Tantivy search and ranking engine.
- `src-tauri/src/ranking.rs`: Composite adaptive scoring algorithm.
- `src-tauri/src/history.rs`: Habit-aware launch history (Time-of-day distribution).
- `src-tauri/src/commands/`: Extensible command plugin registry and system actions.
- `src-tauri/src/indexer/`: Core file indexing with PNG icon caching.
- `src-tauri/src/currency.rs`: Live currency conversion intent layer.

### Frontend (JS/CSS)

- `src/main.js`: Core event coordination and app state.
- `src/ui.js`: Modular result rendering and selection management.
- `src/dialog.js`: High-fidelity confirmation modal logic.
- `src/utils.js`: Shared design tokens and security helpers.
- `src/styles.css`: Glassmorphism design system.

---
