// ── Category Config ──────────────────────────────────────────────────────────

export const CATEGORY_CONFIG = {
  "FILTER":    { title: "Quick Filters", isNew: true },
  "RECENT":    { title: "Recently Used" },
  "COMMAND":   { title: "Actions & Results" },
  "WEB SHORTCUT": { title: "Web Shortcuts" },
  "WEB":       { title: "Web & Discovery" },
  "APP":       { title: "Applications" },
  "DOC":       { title: "Documents" },
  "XLS":       { title: "Spreadsheets" },
  "PPT":       { title: "Presentations" },
  "IMG":       { title: "Images" },
  "VID":       { title: "Videos" },
  "FILE":      { title: "Other Files" },
  "DOWNLOADS": { title: "Downloads" },
  "DOCUMENTS": { title: "Documents Folder" },
  "PICTURES":  { title: "Pictures Folder" },
  "FOLDER":    { title: "Folders" },
};

// Legacy support if needed, but we'll migrate to CATEGORY_CONFIG
export const CATEGORY_TITLES = Object.fromEntries(
  Object.entries(CATEGORY_CONFIG).map(([k, v]) => [k, v.title])
);

export const CATEGORY_PRIORITY = [
  "FILTER", "RECENT", "COMMAND", "WEB SHORTCUT", "WEB",
  "APP", "DOC", "XLS", "PPT", "IMG", "VID",
  "DOWNLOADS", "DOCUMENTS", "PICTURES",
  "FOLDER", "FILE",
];

// Helper: Escape HTML to prevent XSS from raw filenames
export function escapeHtml(unsafe) {
  if (!unsafe) return "";
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

// Helper: sort results by category priority
export function sortByPriority(res) {
  return res.sort((a, b) => {
    const pA = CATEGORY_PRIORITY.indexOf(a.category);
    const pB = CATEGORY_PRIORITY.indexOf(b.category);
    return (pA === -1 ? 99 : pA) - (pB === -1 ? 99 : pB);
  });
}
