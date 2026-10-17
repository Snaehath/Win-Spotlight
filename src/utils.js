// ── Category Config ──────────────────────────────────────────────────────────

export const CATEGORY_CONFIG = {
  "RECENT":    { title: "Recently Used" },
  "COMMAND":   { title: "Actions & Results" },
  "WEB SHORTCUT": { title: "Web Shortcuts" },
  "WEB":       { title: "Web & Discovery" },
  "APP":       { title: "Applications" },
  "DOC":       { title: "Documents" },
  "CODE":      { title: "Code & Config" },
  "XLS":       { title: "Spreadsheets" },
  "PPT":       { title: "Presentations" },
  "IMG":       { title: "Images" },
  "VID":       { title: "Videos" },
  "AUDIO":     { title: "Audio & Music" },
  "ARCHIVE":   { title: "Archives & Compressed" },
  "FOLDER":    { title: "Folders" },
  "DOWNLOADS": { title: "Downloads" },
  "DOCUMENTS": { title: "Documents Folder" },
  "PICTURES":  { title: "Pictures Folder" },
  "FILE":      { title: "Other Files" },
};

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
