import { sortByPriority } from "./utils.js";
import { showConfirm } from "./dialog.js";
import { renderResults, updateSelection } from "./ui.js";

const { invoke } = window.__TAURI__.core;

let searchInput;
let resultsList;
let selectedIndex = -1;
let currentResults = [];
let currentMode = "SEARCH"; // "SEARCH" or "NAMING"
let pendingShortcutUrl = "";
let filterTag;
let activeFilter = null;
let collapsedCategories = new Set();

const KEYWORD_MAP = {
  "app:": "Applications",
  "folder:": "Folders",
  "file:": "Files",
  "command:": "Commands",
};

window.addEventListener("DOMContentLoaded", () => {
  // Initialize Lucide icons
  if (window.lucide) {
    window.lucide.createIcons();
  }

  searchInput = document.querySelector("#search-input");
  resultsList = document.querySelector("#results-list");
  filterTag = document.querySelector("#filter-tag");

  // ── Debounced Search ──────────────────────────────────────────────────────
  let searchTimeout;

  searchInput.addEventListener("input", () => {
    if (currentMode === "NAMING") return; // Don't search while naming

    const query = searchInput.value;
    const activeKeyword = Object.keys(KEYWORD_MAP).find((key) =>
      query.startsWith(key),
    );

    if (activeKeyword) {
      activeFilter = activeKeyword;
      filterTag.innerText = KEYWORD_MAP[activeKeyword];
      filterTag.classList.remove("hidden");
      searchInput.value = "";
      searchInput.placeholder = `Search ${KEYWORD_MAP[activeKeyword]}...`;
    } else if (!activeFilter) {
      activeFilter = null;
      filterTag.classList.add("hidden");
      searchInput.placeholder = "Search...";
    }

    if (searchTimeout) clearTimeout(searchTimeout);
    const delay = query.startsWith(">") ? 250 : 150;

    searchTimeout = setTimeout(async () => {
      const currentVal = searchInput.value;
      const fullQuery = activeFilter ? activeFilter + currentVal : currentVal;
      const res = await invoke("search_items", { query: fullQuery });
      currentResults = sortByPriority(res);
      selectedIndex = -1;
      render();
    }, delay);
  });

  // ── Keyboard navigation ──────────────────────────────────────────────────
  window.addEventListener("keydown", async (e) => {
    if (e.key === "Enter" && e.altKey) {
      e.preventDefault();
      let targetIndex = selectedIndex >= 0 ? selectedIndex : 0;
      const item = currentResults[targetIndex];
      if (item && item.path) {
        await invoke("remove_from_history", { path: item.path });

        const res = await invoke("search_items", { query: searchInput.value });
        currentResults = sortByPriority(res);
        selectedIndex = Math.max(selectedIndex - 1, 0);
        render();
      }
    } else if (e.key === "ArrowDown") {
      let nextIndex = selectedIndex + 1;
      // Skip collapsed categories
      while (nextIndex < currentResults.length && collapsedCategories.has(currentResults[nextIndex].category)) {
        nextIndex++;
      }
      if (nextIndex < currentResults.length) {
        selectedIndex = nextIndex;
        updateSelection(resultsList, selectedIndex);
      }
      e.preventDefault();
    } else if (e.key === "ArrowUp") {
      let prevIndex = selectedIndex - 1;
      // Skip collapsed categories
      while (prevIndex >= 0 && collapsedCategories.has(currentResults[prevIndex].category)) {
        prevIndex--;
      }
      if (prevIndex >= 0) {
        selectedIndex = prevIndex;
        updateSelection(resultsList, selectedIndex);
      }
      e.preventDefault();
    } else if (
      e.key === "Backspace" &&
      searchInput.value === "" &&
      activeFilter
    ) {
      activeFilter = null;
      filterTag.classList.add("hidden");
      e.preventDefault();
    } else if (e.key === "Enter") {
      let targetIndex = selectedIndex;
      if (targetIndex === -1 && currentResults.length > 0) targetIndex = 0;

      if (currentMode === "NAMING") {
        saveShortcutAndReset();
        return;
      }

      if (targetIndex >= 0 && targetIndex < currentResults.length) {
        const item = currentResults[targetIndex];
        if (item.path) launchSelected(item.path, e);
      }
    } else if (e.key === "Escape") {
      invoke("hide_window");
    }
  });

  // ── Initial results ────────
  invoke("search_items", { query: "" }).then((res) => {
    currentResults = sortByPriority(res);
    render();
  });

  // ── Auto-hide on blur ────────────────────────────────────────────────────
  window.addEventListener("blur", () => invoke("hide_window"));

  // ── Auto-clear and focus on window show ──────────────────────────────────
  if (window.__TAURI__ && window.__TAURI__.event) {
    window.__TAURI__.event.listen("window-shown", () => {
      searchInput.value = "";
      activeFilter = null;
      if (filterTag) filterTag.classList.add("hidden");
      searchInput.focus();
      invoke("search_items", { query: "" }).then((res) => {
        currentResults = sortByPriority(res);
        selectedIndex = -1;
        render();
      });
    });
  }
});

function render() {
  renderResults(
    resultsList, 
    currentResults, 
    selectedIndex, 
    launchSelected, 
    revealSelected, 
    forgetItem, 
    collapsedCategories, 
    toggleCategory
  );
  // Re-run Lucide to replace <i> with SVGs
  if (window.lucide) {
    window.lucide.createIcons();
  }
}

function toggleCategory(category) {
  if (collapsedCategories.has(category)) {
    collapsedCategories.delete(category);
  } else {
    collapsedCategories.add(category);
  }
  render();
}

// ── Launch Logic ─────────────────────────────────────────────────────────────

async function launchSelected(path, e) {
  if (!path) return;

  // ── Alt + Click: Remove from history ──
  if (e && e.altKey) {
    await invoke("remove_from_history", { path });
    // Refresh results immediately
    const currentVal = searchInput.value;
    const fullQuery = activeFilter ? activeFilter + currentVal : currentVal;
    const res = await invoke("search_items", { query: fullQuery });
    currentResults = sortByPriority(res);
    render();
    return;
  }

  // ── Shift + Click: Reveal in Explorer ──
  if (e && e.shiftKey && !path.startsWith("COMMAND:")) {
    revealSelected(path);
    return;
  }
  const lowerPath = path.toLowerCase();
  
  // Identify the specific item being launched (prioritizing the one that matches the path)
  const item = currentResults.find(i => i.path === path) || 
               (selectedIndex >= 0 ? currentResults[selectedIndex] : null);

  if (item && item.category === "FILTER") {
    searchInput.value = path;
    searchInput.dispatchEvent(new Event("input"));
    return;
  }

  // ── Shortcut Creation Flow ────────────────────────────────────────────────
  if (path.startsWith("CREATE_SHORTCUT:")) {
    pendingShortcutUrl = path.replace("CREATE_SHORTCUT:", "");
    currentMode = "NAMING";
    searchInput.value = "";
    searchInput.placeholder = "Enter alias name (e.g. 'yt')...";
    currentResults = [];
    render();
    return;
  }

  if (path === "CLEAR_SHORTCUTS") {
    const confirmed = await showConfirm(
      "Clear All Shortcuts?",
      "This will permanently delete all your saved web aliases. Are you sure?",
      searchInput,
    );
    if (confirmed) {
      await invoke("clear_shortcuts");
      // Refresh recents
      const res = await invoke("search_items", { query: "" });
      currentResults = sortByPriority(res);
      render();
    }
    return;
  }

  // ── Browser Confirmation ──
  if (
    lowerPath.startsWith("command:> g") ||
    lowerPath.includes("http://") ||
    lowerPath.includes("https://")
  ) {
    const confirmed = await showConfirm(
      "Open Browser?",
      "This will open your default web browser to perform a search or follow a link.",
      searchInput,
    );
    if (!confirmed) return;
  }

  // ── System Actions Confirmation ──
  if (lowerPath.startsWith("command:> sys")) {
    const parts = path.split(" ");
    const actionLabels = {
      shutdown: "Shut Down PC",
      restart: "Restart PC",
      sleep: "Sleep PC",
      lock: "Lock Screen",
      exit: "Exit Spotlight",
      quit: "Exit Spotlight",
    };

    const action = parts[parts.length - 1].toLowerCase();
    const isExit = action === "exit" || action === "quit";

    const confirmed = await showConfirm(
      actionLabels[action] || "System Action",
      isExit
        ? "Are you sure you want to close the app?"
        : `Are you sure you want to ${action} the computer now?`,
      searchInput,
    );
    if (!confirmed) return;
  }

  try {
    const shouldHide = await invoke("launch_app", { path });
    if (shouldHide) {
      await invoke("hide_window");
      searchInput.value = "";
      selectedIndex = -1;
    }
  } catch (err) {
    console.error("Launch error:", err);
  }
}

async function saveShortcutAndReset() {
  const alias = searchInput.value.trim();
  if (alias && pendingShortcutUrl) {
    await invoke("save_shortcut", { alias, url: pendingShortcutUrl });

    // Reset UI
    currentMode = "SEARCH";
    searchInput.placeholder = "Search...";
    searchInput.value = "";
    pendingShortcutUrl = "";

    // Refresh to show newly added shortcut if it matches empty query (recents)
    const res = await invoke("search_items", { query: "" });
    currentResults = sortByPriority(res);
    render();
  }
}

async function revealSelected(path) {
  if (!path || path.startsWith("COMMAND:")) return;
  await invoke("reveal_in_explorer", { path });
  await invoke("hide_window");
}

async function forgetItem(path) {
  if (!path) return;
  await invoke("remove_from_history", { path });
  
  // Refresh results immediately
  const currentVal = searchInput.value;
  const fullQuery = activeFilter ? activeFilter + currentVal : currentVal;
  const res = await invoke("search_items", { query: fullQuery });
  currentResults = sortByPriority(res);
  render();
}
