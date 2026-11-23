import { showConfirm, cancelDialog, isDialogOpen } from "./dialog.js";
import { renderResults, updateSelection } from "./ui.js";
import {
  ItemKind,
  commandHintPrefix,
  copyableValue,
  isCommandHint,
  isFilesystemItem,
  itemKind,
  primaryActionLabel,
} from "./utils.js";

const { invoke } = window.__TAURI__.core;

const FILTERS = {
  "app:": "Applications",
  "folder:": "Folders",
  "file:": "Files",
  "command:": "Commands",
};

const DEFAULT_PLACEHOLDER = "Spotlight Search";
const MAX_RESULTS = 20;
const SEARCH_DELAY_MS = 45;
const COMMAND_SEARCH_DELAY_MS = 150;
const BLUR_HIDE_DELAY_MS = 150;

const SYSTEM_ACTIONS = {
  shutdown: { title: "Shut down this PC?", message: "Unsaved work in open apps may be lost.", ok: "Shut Down", danger: true },
  restart: { title: "Restart this PC?", message: "Unsaved work in open apps may be lost.", ok: "Restart", danger: true },
  sleep: { title: "Put this PC to sleep?", message: "Your session will be suspended.", ok: "Sleep" },
  lock: { title: "Lock the screen?", message: "You'll need to sign in again to continue.", ok: "Lock" },
  exit: { title: "Quit Spotlight?", message: "The launcher and Ctrl+Space shortcut will stop working until you start it again.", ok: "Quit", danger: true },
};

// ── State ───────────────────────────────────────────────────────────────────

const state = {
  results: [],
  selectedIndex: -1,
  filter: null, // one of FILTERS' keys
  mode: "search", // "search" | "naming"
  pendingShortcutUrl: "",
  searchedTerm: "", // the term the current results belong to
  isActivating: false, // single-activation lock; held until the next summon after a launch
  queryId: 0,
};

const el = {};
let searchTimer = null;
let toastTimer = null;
let lastPointer = { x: -1, y: -1 };

// ── Rendering ───────────────────────────────────────────────────────────────

function currentTerm() {
  return el.input.value.trim();
}

function fullQuery() {
  return (state.filter || "") + currentTerm();
}

function selectedItem() {
  return state.results[state.selectedIndex] ?? null;
}

function syncChrome() {
  const hasQuery = Boolean(el.input.value.trim() || state.filter || state.mode === "naming");
  el.container.classList.toggle("has-query", hasQuery);

  if (state.mode === "naming") {
    el.filterTag.textContent = "New shortcut";
    el.filterTag.classList.remove("hidden");
    el.input.placeholder = "Name this shortcut, e.g. gh";
  } else if (state.filter) {
    el.filterTag.textContent = FILTERS[state.filter];
    el.filterTag.classList.remove("hidden");
    el.input.placeholder = `Search ${FILTERS[state.filter].toLowerCase()}`;
  } else {
    el.filterTag.classList.add("hidden");
    el.input.placeholder = DEFAULT_PLACEHOLDER;
  }
}

function showNotice(title, text) {
  el.noticeTitle.textContent = title;
  el.noticeText.textContent = text;
  el.notice.classList.remove("hidden");
}

function render() {
  const hasResults = state.results.length > 0 && state.mode === "search";
  el.notice.classList.add("hidden");

  if (state.mode === "naming") {
    showNotice(`Save a shortcut for ${state.pendingShortcutUrl}`, "Type a name and press Enter  ·  Esc to cancel");
  } else if (!hasResults && state.searchedTerm) {
    showNotice(`No results for “${state.searchedTerm}”`, "Try a different term, or press F1 to see what you can search for");
  }

  const showPanel = hasResults || !el.notice.classList.contains("hidden");
  el.container.classList.toggle("empty-state", !showPanel);
  el.list.classList.toggle("hidden", !hasResults);
  el.input.setAttribute("aria-expanded", String(hasResults));

  renderResults(el.list, hasResults ? state.results : [], state.selectedIndex, state.searchedTerm);
  syncSelectionChrome();
}

function select(index, { scroll = true } = {}) {
  if (index === state.selectedIndex || index < 0 || index >= state.results.length) return;
  state.selectedIndex = index;
  updateSelection(el.list, index, { scroll });
  syncSelectionChrome();
}

function syncSelectionChrome() {
  const item = selectedItem();
  el.input.setAttribute("aria-activedescendant", item ? `result-${state.selectedIndex}` : "");
  const naming = state.mode === "naming";
  el.footerEnter.textContent = naming ? "Save" : item ? primaryActionLabel(item) : "Open";
  el.footerEnterItem.classList.toggle("hidden", !naming && !item);
  el.footerNav.classList.toggle("hidden", naming || state.results.length < 2);

  let secondary = "";
  if (item && isFilesystemItem(item)) secondary = "Shift+Enter to reveal  ·  Ctrl+Shift+C to copy path";
  else if (item && itemKind(item) === ItemKind.URL && item.category === "WEB") secondary = "Ctrl+Enter to save as shortcut";
  el.footerAction.textContent = secondary;
}

function toast(message, { error = false } = {}) {
  clearTimeout(toastTimer);
  el.toast.textContent = message;
  el.toast.classList.toggle("error", error);
  el.toast.classList.add("visible");
  toastTimer = setTimeout(() => el.toast.classList.remove("visible"), error ? 3500 : 1800);
}

function errorMessage(err) {
  return typeof err === "string" ? err : err?.message || "Something went wrong";
}

// ── Search ──────────────────────────────────────────────────────────────────

async function runSearch() {
  const queryId = ++state.queryId;
  const term = currentTerm();
  const query = fullQuery();

  // Nothing to search: a filter with no term (except command:) or an empty box.
  if (!query || (state.filter && state.filter !== "command:" && !term)) {
    state.results = [];
    state.selectedIndex = -1;
    state.searchedTerm = "";
    render();
    return;
  }

  let results;
  try {
    results = await invoke("search_items", { query });
  } catch (err) {
    console.error("search_items failed:", err);
    results = [];
  }
  if (queryId !== state.queryId || state.mode !== "search") return; // stale

  state.results = groupByCategory(results.slice(0, MAX_RESULTS));
  state.selectedIndex = state.results.length ? 0 : -1;
  state.searchedTerm = term;
  render();
}

/**
 * The backend returns rows in rank order, which can interleave categories
 * (Folders, Documents, Folders…). Keep the top hit first, then gather the rest
 * by category in order of each category's best-ranked row, so every section
 * header appears once. Order within a category is preserved.
 */
function groupByCategory(results) {
  if (results.length < 3) return results;
  const [top, ...rest] = results;
  const groups = new Map();
  for (const item of rest) {
    if (!groups.has(item.category)) groups.set(item.category, []);
    groups.get(item.category).push(item);
  }
  return [top, ...[...groups.values()].flat()];
}

function scheduleSearch() {
  clearTimeout(searchTimer);
  const delay = el.input.value.trimStart().startsWith(">") ? COMMAND_SEARCH_DELAY_MS : SEARCH_DELAY_MS;
  searchTimer = setTimeout(runSearch, delay);
}

function searchNow() {
  clearTimeout(searchTimer);
  return runSearch();
}

function setFilter(keyword) {
  state.filter = keyword;
  syncChrome();
}

function onInput() {
  if (state.mode === "naming") {
    syncChrome();
    return;
  }

  // Typed or pasted filter prefix ("app:chrome") becomes a chip + term.
  const value = el.input.value;
  const keyword = Object.keys(FILTERS).find((key) => value.toLowerCase().startsWith(key));
  if (keyword) {
    el.input.value = value.slice(keyword.length).trimStart();
    setFilter(keyword);
  }

  if (el.input.value.trim().toLowerCase() === "/help") {
    el.input.value = "";
    syncChrome();
    openHelp();
    return;
  }

  syncChrome();
  scheduleSearch();
}

// ── Overlays ────────────────────────────────────────────────────────────────

function isHelpOpen() {
  return !el.help.classList.contains("hidden");
}

function openHelp() {
  el.help.classList.remove("hidden");
  el.container.classList.add("overlay-open");
}

function closeHelp() {
  el.help.classList.add("hidden");
  if (!isDialogOpen()) el.container.classList.remove("overlay-open");
  el.input.focus();
}

// ── Session lifecycle ───────────────────────────────────────────────────────

function resetSession({ releaseLock }) {
  cancelDialog();
  clearTimeout(searchTimer);
  state.queryId++;
  state.mode = "search";
  state.pendingShortcutUrl = "";
  state.filter = null;
  state.results = [];
  state.selectedIndex = -1;
  state.searchedTerm = "";
  if (releaseLock) state.isActivating = false;
  el.input.value = "";
  el.help.classList.add("hidden");
  el.container.classList.remove("overlay-open");
  el.toast.classList.remove("visible");
  syncChrome();
  render();
}

async function hideWindow() {
  resetSession({ releaseLock: false });
  try {
    await invoke("hide_window");
  } catch (err) {
    console.error("hide_window failed:", err);
  }
}

// ── Shortcut naming ─────────────────────────────────────────────────────────

function enterNamingMode(url) {
  clearTimeout(searchTimer);
  state.queryId++;
  state.mode = "naming";
  state.pendingShortcutUrl = url;
  state.filter = null;
  state.results = [];
  state.selectedIndex = -1;
  el.input.value = "";
  syncChrome();
  render();
  el.input.focus();
}

function exitNamingMode() {
  state.mode = "search";
  state.pendingShortcutUrl = "";
  el.input.value = "";
  state.searchedTerm = "";
  syncChrome();
  render();
}

async function saveShortcut() {
  const alias = currentTerm();
  if (!alias) {
    toast("Type a name for the shortcut first", { error: true });
    return;
  }
  try {
    await invoke("save_shortcut", { alias, url: state.pendingShortcutUrl });
    exitNamingMode();
    toast(`Saved shortcut “${alias.toLowerCase()}”`);
  } catch (err) {
    toast(errorMessage(err), { error: true });
  }
}

// ── Item actions ────────────────────────────────────────────────────────────

function urlOf(item) {
  return item.path.replace(/^COMMAND:/, "");
}

async function copyText(text, label = text) {
  try {
    await navigator.clipboard.writeText(text);
    toast(`Copied ${label}`);
  } catch (err) {
    toast("Couldn't copy to the clipboard", { error: true });
  }
}

async function copyItem(item) {
  if (!item) return;
  const kind = itemKind(item);
  if (isFilesystemItem(item)) return copyText(item.path, "path");
  if (kind === ItemKind.URL) return copyText(urlOf(item), "URL");
  if (kind === ItemKind.INFO && !isCommandHint(item)) {
    const value = copyableValue(item);
    return copyText(value, value.length > 40 ? "result" : value);
  }
}

async function revealItem(item) {
  if (!item || !isFilesystemItem(item)) return;
  try {
    await invoke("reveal_in_explorer", { path: item.path });
    await hideWindow();
  } catch (err) {
    toast(errorMessage(err), { error: true });
  }
}

async function forgetItem(item) {
  if (!item?.path || !isFilesystemItem(item)) return;
  try {
    await invoke("remove_from_history", { path: item.path });
    toast(`Removed “${item.name}” from history`);
    await searchNow();
  } catch (err) {
    toast(errorMessage(err), { error: true });
  }
}

function saveAsShortcut(item) {
  if (item && itemKind(item) === ItemKind.URL) enterNamingMode(urlOf(item));
}

async function confirmSystemAction(item) {
  const action = item.path.trim().split(/\s+/).pop().toLowerCase();
  const copy = SYSTEM_ACTIONS[action === "quit" ? "exit" : action] ?? {
    title: `${item.name}?`,
    message: "Are you sure you want to continue?",
    ok: "Continue",
  };
  return showConfirm(copy.title, copy.message, { okText: copy.ok, danger: copy.danger, returnFocus: el.input });
}

async function confirmAnotherInstance(item) {
  try {
    const running = await invoke("check_app_running", { path: item.path });
    if (!running?.is_running) return true;
    const name = running.app_name || item.name || "This app";
    const message = running.kind === "window" ? `Open another ${name} window?` : "Launch another instance?";
    return showConfirm(`${name} is already running`, message, { okText: "Open Another", returnFocus: el.input });
  } catch (err) {
    console.warn("check_app_running failed:", err);
    return true;
  }
}

/** Run the primary action of the row at `index`, honouring modifier keys. */
async function activate(index, mods = {}) {
  const item = state.results[index];
  if (!item || state.isActivating) return;

  if (mods.altKey) return forgetItem(item);
  if (mods.shiftKey && isFilesystemItem(item)) return revealItem(item);
  if (mods.ctrlKey && itemKind(item) === ItemKind.URL) return saveAsShortcut(item);

  const kind = itemKind(item);

  if (kind === ItemKind.INFO) {
    if (isCommandHint(item)) {
      el.input.value = commandHintPrefix(item);
      onInput();
    } else {
      await copyItem(item);
    }
    return;
  }

  state.isActivating = true;
  let keepLock = false;
  try {
    if (kind === ItemKind.CLEAR_SHORTCUTS) {
      const ok = await showConfirm("Delete all web shortcuts?", "Every saved shortcut will be removed. This can't be undone.", {
        okText: "Delete All",
        danger: true,
        returnFocus: el.input,
      });
      if (ok) {
        await invoke("clear_shortcuts");
        toast("All shortcuts deleted");
        await searchNow();
      }
      return;
    }

    if (kind === ItemKind.SYSTEM && !(await confirmSystemAction(item))) return;
    if (kind === ItemKind.APP && !(await confirmAnotherInstance(item))) return;

    const shouldHide = await invoke("launch_app", { path: item.path });
    if (shouldHide) {
      // Hold the lock: OS key-repeat or a stray click must not launch twice.
      // `window-shown` releases it at the start of the next session.
      keepLock = true;
      await hideWindow();
    }
  } catch (err) {
    console.error("launch failed:", err);
    toast(errorMessage(err), { error: true });
  } finally {
    if (!keepLock) state.isActivating = false;
  }
}

// ── Keyboard ────────────────────────────────────────────────────────────────

function moveSelection(delta) {
  const count = state.results.length;
  if (!count) return;
  const from = state.selectedIndex < 0 ? (delta > 0 ? -1 : 0) : state.selectedIndex;
  select((from + delta + count) % count);
}

function onKeyDown(e) {
  if (isDialogOpen()) return; // the dialog handles its own keys

  if (e.repeat && (e.key === "Enter" || e.key === "Escape")) {
    e.preventDefault();
    return;
  }

  if (e.key === "F1") {
    e.preventDefault();
    isHelpOpen() ? closeHelp() : openHelp();
    return;
  }

  if (isHelpOpen()) {
    e.preventDefault();
    if (e.key === "Escape") closeHelp();
    return;
  }

  switch (e.key) {
    case "Escape":
      e.preventDefault();
      if (state.mode === "naming") exitNamingMode();
      else hideWindow();
      return;

    case "ArrowDown":
    case "ArrowUp":
      e.preventDefault();
      if (state.mode === "search") moveSelection(e.key === "ArrowDown" ? 1 : -1);
      return;

    case "Enter":
      e.preventDefault();
      if (state.mode === "naming") saveShortcut();
      else activate(state.selectedIndex, e);
      return;

    case "Backspace":
      if (!el.input.value && el.input.selectionStart === 0) {
        if (state.mode === "naming") {
          e.preventDefault();
          exitNamingMode();
        } else if (state.filter) {
          e.preventDefault();
          setFilter(null);
          searchNow();
        }
      }
      return;
  }

  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "c") {
    e.preventDefault();
    copyItem(selectedItem());
  }
}

// ── Wiring ──────────────────────────────────────────────────────────────────

function bindPointer() {
  el.list.addEventListener("mousemove", (e) => {
    if (e.clientX === lastPointer.x && e.clientY === lastPointer.y) return; // ignore scroll-induced moves
    lastPointer = { x: e.clientX, y: e.clientY };
    const row = e.target.closest(".result-item");
    if (row) select(Number(row.dataset.index), { scroll: false });
  });

  el.list.addEventListener("click", (e) => {
    const row = e.target.closest(".result-item");
    if (!row) return;
    const index = Number(row.dataset.index);
    const item = state.results[index];
    const action = e.target.closest(".row-action")?.dataset.action;

    if (action === "reveal") revealItem(item);
    else if (action === "copy") copyItem(item);
    else if (action === "save-shortcut") saveAsShortcut(item);
    else activate(index, e);

    el.input.focus();
  });

  // Keep focus in the search box when clicking chrome.
  el.container.addEventListener("mousedown", (e) => {
    if (e.target !== el.input && !e.target.closest(".dialog-box")) e.preventDefault();
  });

  document.querySelectorAll(".idle-shortcut").forEach((button) => {
    button.addEventListener("click", () => {
      setFilter(button.dataset.query);
      el.input.focus();
      searchNow();
    });
  });

  el.filterTag.addEventListener("click", () => {
    if (state.mode === "naming") exitNamingMode();
    else {
      setFilter(null);
      searchNow();
    }
    el.input.focus();
  });
}

function bindWindow() {
  let blurTimer = null;
  window.addEventListener("blur", () => {
    blurTimer = setTimeout(hideWindow, BLUR_HIDE_DELAY_MS);
  });
  window.addEventListener("focus", () => clearTimeout(blurTimer));

  window.__TAURI__?.event?.listen("window-shown", () => {
    resetSession({ releaseLock: true });
    el.input.focus();
  });
}

window.addEventListener("DOMContentLoaded", () => {
  el.container = document.getElementById("container");
  el.input = document.getElementById("search-input");
  el.list = document.getElementById("results-list");
  el.filterTag = document.getElementById("filter-tag");
  el.help = document.getElementById("help-overlay");
  el.notice = document.getElementById("notice");
  el.noticeTitle = document.getElementById("notice-title");
  el.noticeText = document.getElementById("notice-text");
  el.footerAction = document.getElementById("footer-action");
  el.footerEnter = document.getElementById("footer-enter");
  el.footerEnterItem = document.getElementById("footer-enter-item");
  el.footerNav = document.getElementById("footer-nav");
  el.toast = document.getElementById("toast");

  el.input.addEventListener("input", onInput);
  window.addEventListener("keydown", onKeyDown);
  bindPointer();
  bindWindow();

  syncChrome();
  render();
  el.input.focus();
});
