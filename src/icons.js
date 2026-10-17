const ICON_PATHS = {
  activity: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>',
  "chevron-down": '<path d="m6 9 6 6 6-6"/>',
  cpu: '<rect x="5" y="5" width="14" height="14" rx="2"/><path d="M9 9h6v6H9zM9 1v4m6-4v4M9 19v4m6-4v4M1 9h4m-4 6h4m14-6h4m-4 6h4"/>',
  "folder-search": '<path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v3"/><path d="M3 7v11a2 2 0 0 0 2 2h6"/><circle cx="16" cy="16" r="3"/><path d="m18.2 18.2 2.3 2.3"/>',
  globe: '<circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>',
  "hard-drive": '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M7 15h.01M11 15h.01M3 10h18"/>',
  "log-out": '<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="m16 17 5-5-5-5m5 5H9"/>',
  moon: '<path d="M20.9 13A9 9 0 0 1 11 3.1 9 9 0 1 0 20.9 13z"/>',
  power: '<path d="M12 2v10"/><path d="M18.4 6.6a9 9 0 1 1-12.8 0"/>',
  "refresh-cw": '<path d="M20 7v5h-5M4 17v-5h5"/><path d="M5.6 9a7 7 0 0 1 11.6-2L20 12M4 12l2.8 5a7 7 0 0 0 11.6-2"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/>',
  "trash-2": '<path d="M3 6h18M8 6V4h8v2m3 0-1 14H6L5 6m4 4v6m6-6v6"/>',
  terminal: '<path d="m4 17 6-6-6-6m8 14h8"/>',
};

export function iconSvg(name, className = "") {
  const paths = ICON_PATHS[name] || ICON_PATHS.terminal;
  const classAttr = className ? ` class="${className}"` : "";
  return `<svg${classAttr} xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;
}
