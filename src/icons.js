// Inline SVG glyphs. Names match the `icon` values the backend emits for
// synthetic rows (system actions, shortcuts, stats) plus a few UI-only icons.
const ICON_PATHS = {
  activity: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>',
  "bookmark-plus": '<path d="M19 21l-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/><path d="M12 7v6m-3-3h6"/>',
  calculator: '<rect x="4" y="2" width="16" height="20" rx="2"/><path d="M8 6h8M8 11h.01M12 11h.01M16 11h.01M8 15h.01M12 15h.01M16 15h.01M8 19h.01M12 19h4"/>',
  copy: '<rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1"/>',
  cpu: '<rect x="5" y="5" width="14" height="14" rx="2"/><path d="M9 9h6v6H9zM9 1v4m6-4v4M9 19v4m6-4v4M1 9h4m-4 6h4m14-6h4m-4 6h4"/>',
  "folder-search": '<path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v3"/><path d="M3 7v11a2 2 0 0 0 2 2h6"/><circle cx="16" cy="16" r="3"/><path d="m18.2 18.2 2.3 2.3"/>',
  globe: '<circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>',
  "hard-drive": '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M7 15h.01M11 15h.01M3 10h18"/>',
  "link-2": '<path d="M9 17H7A5 5 0 0 1 7 7h2m6 0h2a5 5 0 0 1 0 10h-2M8 12h8"/>',
  lock: '<rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>',
  "log-out": '<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="m16 17 5-5-5-5m5 5H9"/>',
  moon: '<path d="M20.9 13A9 9 0 0 1 11 3.1 9 9 0 1 0 20.9 13z"/>',
  power: '<path d="M12 2v10"/><path d="M18.4 6.6a9 9 0 1 1-12.8 0"/>',
  "refresh-cw": '<path d="M20 7v5h-5M4 17v-5h5"/><path d="M5.6 9a7 7 0 0 1 11.6-2L20 12M4 12l2.8 5a7 7 0 0 0 11.6-2"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/>',
  terminal: '<path d="m4 17 6-6-6-6m8 14h8"/>',
  "trash-2": '<path d="M3 6h18M8 6V4h8v2m3 0-1 14H6L5 6m4 4v6m6-6v6"/>',
};

export function iconSvg(name) {
  const paths = ICON_PATHS[name] || ICON_PATHS.terminal;
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${paths}</svg>`;
}
