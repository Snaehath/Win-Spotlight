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
let helpOverlay;
let isActivating = false;

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
  helpOverlay = document.querySelector("#help-overlay");

  document.querySelector("#help-btn").addEventListener("click", (e) => {
    e.stopPropagation();
    helpOverlay.classList.toggle("hidden");
  });

  // ── Debounced Search ──────────────────────────────────────────────────────
  let searchTimeout;
  let currentQueryId = 0;

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
    const delay = query.startsWith(">") ? 150 : 45;

    searchTimeout = setTimeout(async () => {
      const currentVal = searchInput.value;

      // ── Elite Internal Command: /help ──
      if (currentVal.toLowerCase() === "/help") {
        helpOverlay.classList.remove("hidden");
        searchInput.value = "";
        return;
      }

      const queryId = ++currentQueryId;
      const fullQuery = activeFilter ? activeFilter + currentVal : currentVal;
      const res = await invoke("search_items", { query: fullQuery });
      if (queryId !== currentQueryId) return; // Drop stale out-of-order response

      currentResults = sortByPriority(res).slice(0, 10);
      selectedIndex = -1;
      render();
    }, delay);
  });

  // ── Keyboard navigation ──────────────────────────────────────────────────
  window.addEventListener("keydown", async (e) => {
    // Single Activation Invariant: Ignore OS key-repeat for activation/dismiss keys
    if (e.repeat && (e.key === "Enter" || e.key === "Escape")) {
      e.preventDefault();
      return;
    }

    if (e.key === "Enter" && e.altKey) {
      e.preventDefault();
      let targetIndex = selectedIndex >= 0 ? selectedIndex : 0;
      const item = currentResults[targetIndex];
      if (item && item.path) {
        await invoke("remove_from_history", { path: item.path });

        const res = await invoke("search_items", { query: searchInput.value });
        currentResults = sortByPriority(res).slice(0, 10);
        selectedIndex = Math.max(selectedIndex - 1, 0);
        render();
      }
    } else if ((e.key === "C" || e.key === "c") && e.ctrlKey && e.shiftKey) {
      e.preventDefault();
      let targetIndex = selectedIndex >= 0 ? selectedIndex : 0;
      const item = currentResults[targetIndex];
      if (item && item.path) {
        navigator.clipboard.writeText(item.path);
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
      // Single Activation Invariant: Prevent multiple launches within one session
      if (isActivating) {
        e.preventDefault();
        return;
      }

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
      if (!helpOverlay.classList.contains("hidden")) {
        helpOverlay.classList.add("hidden");
      } else {
        invoke("hide_window");
      }
    }
  });

  // ── Initial results ────────
  invoke("search_items", { query: "" }).then((res) => {
    currentResults = sortByPriority(res).slice(0, 10);
    render();
  });

  // ── Auto-hide on blur with debounce ──────────────────────────────────────
  let blurTimeout;
  window.addEventListener("blur", () => {
    blurTimeout = setTimeout(() => {
      invoke("hide_window");
    }, 150);
  });

  window.addEventListener("focus", () => {
    if (blurTimeout) {
      clearTimeout(blurTimeout);
    }
  });

  // ── Auto-clear and focus on window show ──────────────────────────────────
  if (window.__TAURI__ && window.__TAURI__.event) {
    window.__TAURI__.event.listen("window-shown", () => {
      isActivating = false; // Reset session lock for fresh launcher session
      searchInput.value = "";
      activeFilter = null;
      if (filterTag) filterTag.classList.add("hidden");
      searchInput.focus();
      invoke("search_items", { query: "" }).then((res) => {
        currentResults = sortByPriority(res).slice(0, 10);
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
  if (!path || isActivating) return;
  isActivating = true;

  const releaseLock = () => {
    isActivating = false;
  };

  // ── Alt + Click: Remove from history ──
  if (e && e.altKey) {
    try {
      await invoke("remove_from_history", { path });
      // Refresh results immediately
      const currentVal = searchInput.value;
      const fullQuery = activeFilter ? activeFilter + currentVal : currentVal;
      const res = await invoke("search_items", { query: fullQuery });
      currentResults = sortByPriority(res);
      render();
    } finally {
      releaseLock();
    }
    return;
  }

  // ── Shift + Click: Reveal in Explorer ──
  if (e && e.shiftKey && !path.startsWith("COMMAND:")) {
    releaseLock();
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
    releaseLock();
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
    releaseLock();
    return;
  }

  if (path === "CLEAR_SHORTCUTS") {
    try {
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
    } finally {
      releaseLock();
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
    if (!confirmed) {
      releaseLock();
      return;
    }
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
    if (!confirmed) {
      releaseLock();
      return;
    }
  }

  // ── Intentional Multi-Instance Confirmation ──
  const isApp = (item && item.category === "APP") || lowerPath.endsWith(".exe") || lowerPath.endsWith(".lnk");
  if (isApp && !path.startsWith("COMMAND:")) {
    try {
      const runningInfo = await invoke("check_app_running", { path });
      if (runningInfo && runningInfo.is_running) {
        const appTitle = runningInfo.app_name || (item ? item.name : "Application");
        const isWindowed = runningInfo.kind === "window";
        const message = isWindowed
          ? `Do you want to open another ${appTitle} window?`
          : `Do you want to launch another instance?`;

        const confirmed = await showConfirm(
          `${appTitle} is already running`,
          message,
          searchInput,
          { okText: "Open another", cancelText: "Cancel" }
        );
        if (!confirmed) {
          releaseLock();
          return;
        }
      }
    } catch (err) {
      console.warn("Could not check running app status:", err);
    }
  }

  // ── Actual Launch Execution ──
  try {
    const shouldHide = await invoke("launch_app", { path });
    if (shouldHide) {
      await invoke("hide_window");
      searchInput.value = "";
      selectedIndex = -1;
      // Single Activation Invariant:
      // The session lock (isActivating) deliberately remains TRUE.
      // Subsequent Enter keydowns, OS key-repeats, or clicks are completely ignored
      // until window-shown resets isActivating for a fresh launcher session.
    } else {
      releaseLock();
    }
  } catch (err) {
    console.error("Launch error:", err);
    releaseLock();
  }
}

async function saveShortcutAndReset() {
  const alias = searchInput.value.trim();
  if (alias && pendingShortcutUrl) {
    try {
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
    } catch (err) {
      alert("Failed to save shortcut: " + err);
    }
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
