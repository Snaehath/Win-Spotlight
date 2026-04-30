// ── Category Config ──────────────────────────────────────────────────────────

export const CATEGORY_TITLES = {
  "FILTER":    "Quick Filters",
  "RECENT":    "Recently Used",
  "COMMAND":   "Actions & Results",
  "WEB SHORTCUT": "Web Shortcuts",
  "WEB":       "Web & Discovery",
  "APP":       "Applications",
  "DOC":       "Documents",
  "XLS":       "Spreadsheets",
  "PPT":       "Presentations",
  "IMG":       "Images",
  "VID":       "Videos",
  "FILE":      "Other Files",
  "DOWNLOADS": "Downloads",
  "DOCUMENTS": "Documents Folder",
  "PICTURES":  "Pictures Folder",
  "FOLDER":    "Folders",
};

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
