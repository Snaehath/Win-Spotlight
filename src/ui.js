import { escapeHtml, CATEGORY_CONFIG } from "./utils.js";

export function renderResults(
  resultsList,
  currentResults,
  selectedIndex,
  onLaunch,
  onReveal,
  onForget,
  collapsedCategories,
  onToggleCategory,
) {
  resultsList.innerHTML = "";
  let lastCategory = null;

  currentResults.forEach((item, index) => {
    // ── Skip Filters ──
    if (item.category === "FILTER") return;

    // Category header
    if (item.category !== lastCategory) {
      const isCollapsed =
        collapsedCategories && collapsedCategories.has(item.category);
      const header = document.createElement("div");
      header.className = `category-header ${isCollapsed ? "collapsed" : ""}`;

      const config = CATEGORY_CONFIG[item.category] || { title: item.category };

      // Toggle icon
      const toggle = document.createElement("div");
      toggle.className = "category-toggle";
      toggle.innerHTML = '<i data-lucide="chevron-down"></i>';
      header.appendChild(toggle);

      const titleSpan = document.createElement("span");
      titleSpan.innerText = config.title;
      header.appendChild(titleSpan);

      if (config.isNew) {
        const badge = document.createElement("span");
        badge.className = "badge-new";
        badge.innerText = "NEW";
        header.appendChild(badge);
      }

      if (onToggleCategory) {
        header.onclick = () => onToggleCategory(item.category);
      }

      resultsList.appendChild(header);
      lastCategory = item.category;
    }

    // Skip rendering items if category is collapsed
    if (collapsedCategories && collapsedCategories.has(item.category)) {
      return;
    }

    const li = document.createElement("li");
    li.className = `result-item ${index === selectedIndex ? "selected" : ""}`;
    li.dataset.index = index;
    li.style.setProperty("--item-index", index);

    // Icon logic
    let iconHTML = "";
    if (item.icon) {
      if (item.icon.startsWith("data:") || item.icon.length > 50) {
        const iconSrc = item.icon.startsWith("data:")
          ? item.icon
          : `data:image/png;base64,${item.icon}`;
        iconHTML = `<img src="${escapeHtml(iconSrc)}" class="app-icon" alt="" />`;
      } else {
        iconHTML = `<div class="app-icon lucide-wrapper"><i data-lucide="${escapeHtml(item.icon)}"></i></div>`;
      }
    } else {
      const isCmd =
        item.category === "COMMAND" ||
        item.category === "WEB" ||
        item.category === "WEB SHORTCUT";
      iconHTML = `<div class="app-icon placeholder ${isCmd ? "cmd-icon" : ""}">
                   ${isCmd ? '<i data-lucide="terminal"></i>' : ""}
                 </div>`;
    }

    // Sub-label
    const subLabel = item.inline_display
      ? `<span class="app-path inline-result">${escapeHtml(item.inline_display)}</span>`
      : item.path
        ? `<span class="app-path">${escapeHtml(item.path)}</span>`
        : "";

    li.innerHTML = `
      ${iconHTML}
      <div class="app-info">
        <div class="app-header">
          <span class="app-name">${escapeHtml(item.name)}</span>
        </div>
        ${subLabel}
      </div>
      <div class="action-bar">
        <button class="action-icon reveal-btn" title="Reveal in Explorer">
          <i data-lucide="folder-search"></i>
        </button>
        <button class="action-icon forget-btn" title="Remove from History">
          <i data-lucide="trash-2"></i>
        </button>
      </div>
    `;

    // Click handlers
    if (item.path) {
      li.onclick = (e) => onLaunch(item.path, e);

      const revealBtn = li.querySelector(".reveal-btn");
      const forgetBtn = li.querySelector(".forget-btn");

      revealBtn.onclick = (e) => {
        e.stopPropagation();
        onReveal(item.path);
      };

      forgetBtn.onclick = (e) => {
        e.stopPropagation();
        onForget(item.path);
      };

      // Hide reveal for commands
      if (item.category === "COMMAND") {
        revealBtn.style.display = "none";
      }

      // Hide forget if not RECENT
      if (item.category !== "RECENT") {
        forgetBtn.style.display = "none";
      }
    }

    resultsList.appendChild(li);

    if (index === selectedIndex) {
      li.scrollIntoView({ block: "nearest" });
    }
  });
}

export function updateSelection(resultsList, selectedIndex) {
  const items = resultsList.querySelectorAll(".result-item");
  items.forEach((li) => {
    if (parseInt(li.dataset.index) === selectedIndex) {
      li.classList.add("selected");
      li.scrollIntoView({ block: "nearest", behavior: "smooth" });
    } else {
      li.classList.remove("selected");
    }
  });
}
