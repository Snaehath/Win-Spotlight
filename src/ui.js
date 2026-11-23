import {
  CATEGORY_CONFIG,
  ItemKind,
  categoryTitle,
  escapeHtml,
  highlightMatch,
  isCommandHint,
  isFilesystemItem,
  itemKind,
} from "./utils.js";
import { iconSvg } from "./icons.js";

const FALLBACK_GLYPH = {
  [ItemKind.URL]: "globe",
  [ItemKind.SYSTEM]: "power",
  [ItemKind.CLEAR_SHORTCUTS]: "trash-2",
  [ItemKind.COMMAND]: "terminal",
  [ItemKind.INFO]: "calculator",
};

function renderIcon(item, kind) {
  const icon = item.icon;
  if (icon && (icon.startsWith("data:") || icon.length > 50)) {
    const src = icon.startsWith("data:") ? icon : `data:image/png;base64,${icon}`;
    return `<img src="${escapeHtml(src)}" class="app-icon" alt="" draggable="false" />`;
  }
  const glyph = icon || FALLBACK_GLYPH[kind] || "terminal";
  const tone = kind === ItemKind.INFO && !isCommandHint(item) ? " accent" : "";
  return `<div class="app-icon glyph${tone}">${iconSvg(glyph)}</div>`;
}

/**
 * Inline results ("45 * 12 = 540", "100 USD = 92.13 EUR") read better with the
 * answer as the title and the question as the subtitle.
 */
function describe(item, kind, term) {
  const inline = item.inline_display;
  if (inline) {
    const eq = inline.lastIndexOf(" = ");
    const action = kind === ItemKind.INFO ? "Enter to copy" : "Enter to search the web";
    if (eq >= 0) {
      return {
        title: escapeHtml(inline.slice(eq + 3)),
        subtitle: `${inline.slice(0, eq)}  ·  ${action}`,
      };
    }
    return { title: escapeHtml(inline), subtitle: action };
  }

  if (isCommandHint(item)) {
    const [prefix, ...rest] = item.name.split("—");
    return { title: escapeHtml(prefix.trim()), subtitle: rest.join("—").trim() };
  }

  return { title: highlightMatch(item.name, term), subtitle: item.subtitle || "" };
}

function renderActions(item, kind) {
  const buttons = [];
  if (isFilesystemItem(item)) {
    buttons.push(["reveal", "folder-search", "Reveal in Explorer (Shift+Enter)"]);
    buttons.push(["copy", "copy", "Copy path (Ctrl+Shift+C)"]);
  } else if (kind === ItemKind.URL) {
    if (item.category === "WEB") {
      buttons.push(["save-shortcut", "bookmark-plus", "Save as shortcut (Ctrl+Enter)"]);
    }
    buttons.push(["copy", "copy", "Copy URL (Ctrl+Shift+C)"]);
  } else if (kind === ItemKind.INFO && !isCommandHint(item)) {
    buttons.push(["copy", "copy", "Copy (Enter)"]);
  }

  if (!buttons.length) return "";
  return `<div class="row-actions">${buttons
    .map(
      ([action, icon, label]) =>
        `<button class="row-action" type="button" tabindex="-1" data-action="${action}" title="${label}" aria-label="${label}">${iconSvg(icon)}</button>`,
    )
    .join("")}</div>`;
}

export function renderResults(list, results, selectedIndex, term) {
  let html = "";
  let lastCategory = null;

  results.forEach((item, index) => {
    // The best match is lifted into its own "Top Hit" group; the rest stay
    // grouped by category (so a second app still gets an "Applications" header).
    const group = index === 0 ? "Top Hit" : categoryTitle(item.category);
    if (group !== lastCategory) {
      lastCategory = group;
      html += `<li class="category-header" role="presentation">${escapeHtml(group)}</li>`;
    }

    const kind = itemKind(item);
    const { title, subtitle } = describe(item, kind, term);
    const actions = renderActions(item, kind);
    const label = CATEGORY_CONFIG[item.category]?.label ?? "";
    const classes = ["result-item"];
    if (index === 0) classes.push("top-hit");
    if (index === selectedIndex) classes.push("selected");
    if (item.inline_display) classes.push("inline-result");
    if (actions) classes.push("has-actions");

    html += `
      <li id="result-${index}" class="${classes.join(" ")}" role="option" aria-selected="${index === selectedIndex}" data-index="${index}">
        ${renderIcon(item, kind)}
        <div class="app-info">
          <span class="app-name">${title}</span>
          ${subtitle ? `<span class="app-subtitle">${escapeHtml(subtitle)}</span>` : ""}
        </div>
        ${label && !item.inline_display ? `<span class="row-hint">${escapeHtml(label)}</span>` : ""}
        ${actions}
      </li>`;
  });

  list.innerHTML = html;
  list.scrollTop = 0;
}

export function updateSelection(list, selectedIndex, { scroll = true } = {}) {
  const previous = list.querySelector(".result-item.selected");
  previous?.classList.remove("selected");
  previous?.setAttribute("aria-selected", "false");

  const row = list.querySelector(`.result-item[data-index="${selectedIndex}"]`);
  if (!row) return;
  row.classList.add("selected");
  row.setAttribute("aria-selected", "true");
  if (!scroll) return;

  // On the first row of a group, bring its section header into view too.
  const header = row.previousElementSibling;
  if (header?.classList.contains("category-header")) header.scrollIntoView({ block: "nearest" });
  row.scrollIntoView({ block: "nearest" });
}
