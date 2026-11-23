// ── Category presentation ───────────────────────────────────────────────────
// `title` is the section header, `label` the short type shown on each row.

export const CATEGORY_CONFIG = {
  RECENT: { title: "Recently Used", label: "Recent" },
  COMMAND: { title: "Actions & Results", label: "Command" },
  "WEB SHORTCUT": { title: "Web Shortcuts", label: "Shortcut" },
  WEB: { title: "Web", label: "Website" },
  APP: { title: "Applications", label: "Application" },
  DOC: { title: "Documents", label: "Document" },
  CODE: { title: "Code & Config", label: "Code" },
  XLS: { title: "Spreadsheets", label: "Spreadsheet" },
  PPT: { title: "Presentations", label: "Presentation" },
  IMG: { title: "Images", label: "Image" },
  VID: { title: "Videos", label: "Video" },
  AUDIO: { title: "Audio", label: "Audio" },
  ARCHIVE: { title: "Archives", label: "Archive" },
  FOLDER: { title: "Folders", label: "Folder" },
  DOWNLOADS: { title: "Downloads", label: "Folder" },
  DOCUMENTS: { title: "Documents Folder", label: "Folder" },
  PICTURES: { title: "Pictures Folder", label: "Folder" },
  FILE: { title: "Other Files", label: "File" },
};

export function categoryTitle(category) {
  return CATEGORY_CONFIG[category]?.title ?? category;
}

// ── Item classification ─────────────────────────────────────────────────────
// The backend encodes row behaviour in `path`; keep that knowledge in one place.

export const ItemKind = Object.freeze({
  INFO: "info", // no path: inline result (math, stats) or a command hint
  SYSTEM: "system", // COMMAND:> sys …  (power actions — needs confirmation)
  URL: "url", // COMMAND:http(s)://…
  COMMAND: "command", // other COMMAND:… rows
  CLEAR_SHORTCUTS: "clear-shortcuts",
  APP: "app",
  FOLDER: "folder",
  FILE: "file",
});

export function itemKind(item) {
  const path = item.path || "";
  if (!path) return ItemKind.INFO;
  if (path === "CLEAR_SHORTCUTS") return ItemKind.CLEAR_SHORTCUTS;
  if (path.startsWith("COMMAND:")) {
    const body = path.slice("COMMAND:".length).toLowerCase();
    if (body.startsWith("http://") || body.startsWith("https://")) return ItemKind.URL;
    if (body.startsWith("> sys")) return ItemKind.SYSTEM;
    return ItemKind.COMMAND;
  }
  if (item.item_type === "App" || item.category === "APP") return ItemKind.APP;
  if (item.item_type === "Folder") return ItemKind.FOLDER;
  return ItemKind.FILE;
}

export function isFilesystemItem(item) {
  const kind = itemKind(item);
  return kind === ItemKind.APP || kind === ItemKind.FOLDER || kind === ItemKind.FILE;
}

/** Label for what Enter does on this row (shown in the footer). */
export function primaryActionLabel(item) {
  switch (itemKind(item)) {
    case ItemKind.APP:
      return "Open";
    case ItemKind.FOLDER:
      return "Open Folder";
    case ItemKind.FILE:
      return "Open File";
    case ItemKind.URL:
      return "Open in Browser";
    case ItemKind.SYSTEM:
    case ItemKind.CLEAR_SHORTCUTS:
      return "Run";
    case ItemKind.COMMAND:
      return "Run Command";
    case ItemKind.INFO:
      return isCommandHint(item) ? "Use Command" : "Copy";
    default:
      return "Open";
  }
}

/** Hint rows look like "> calc  — Evaluate a math expression…". */
export function isCommandHint(item) {
  return !item.path && !item.inline_display && /^>\s*\S+\s+—/.test(item.name || "");
}

export function commandHintPrefix(item) {
  const match = /^>\s*(\S+)/.exec(item.name || "");
  return match ? `> ${match[1]} ` : "";
}

/** For "45 * 2 = 90" return "90"; otherwise the whole text. */
export function copyableValue(item) {
  const text = item.inline_display || item.name || "";
  const idx = text.lastIndexOf(" = ");
  return idx >= 0 ? text.slice(idx + 3).trim() : text.trim();
}

// ── HTML helpers ────────────────────────────────────────────────────────────

const HTML_ESCAPES = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#039;" };

export function escapeHtml(value) {
  if (!value) return "";
  return String(value).replace(/[&<>"']/g, (ch) => HTML_ESCAPES[ch]);
}

/** Escape `text` and wrap the first case-insensitive occurrence of `term` in <mark>. */
export function highlightMatch(text, term) {
  const needle = (term || "").trim().toLowerCase();
  if (!text || !needle) return escapeHtml(text);
  const at = text.toLowerCase().indexOf(needle);
  if (at < 0) return escapeHtml(text);
  return (
    escapeHtml(text.slice(0, at)) +
    `<mark>${escapeHtml(text.slice(at, at + needle.length))}</mark>` +
    escapeHtml(text.slice(at + needle.length))
  );
}
