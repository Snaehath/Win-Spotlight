import { escapeHtml, CATEGORY_TITLES } from './utils.js';

export function renderResults(resultsList, currentResults, selectedIndex, onLaunch) {
  resultsList.innerHTML = "";
  let lastCategory = null;

  currentResults.forEach((item, index) => {
    // Category header
    if (item.category !== lastCategory) {
      const header = document.createElement("div");
      header.className = "category-header";
      header.innerText = CATEGORY_TITLES[item.category] || item.category;
      resultsList.appendChild(header);
      lastCategory = item.category;
    }

    const li = document.createElement("li");
    li.className = `result-item ${index === selectedIndex ? "selected" : ""}`;
    li.style.setProperty("--item-index", index);

    // Icon logic: handles base64 data URIs or named Lucide icons
    let iconHTML = "";
    if (item.icon) {
      if (item.icon.startsWith("data:") || item.icon.length > 50) {
        // Base64 image
        const iconSrc = item.icon.startsWith("data:") ? item.icon : `data:image/png;base64,${item.icon}`;
        iconHTML = `<img src="${escapeHtml(iconSrc)}" class="app-icon" alt="" />`;
      } else {
        // Lucide icon name
        iconHTML = `<div class="app-icon lucide-wrapper"><i data-lucide="${escapeHtml(item.icon)}"></i></div>`;
      }
    } else {
      // Default placeholder
      const isCmd = item.category === 'COMMAND' || item.category === 'WEB' || item.category === 'WEB SHORTCUT';
      iconHTML = `<div class="app-icon placeholder ${isCmd ? 'cmd-icon' : ''}">
                   ${isCmd ? '<i data-lucide="terminal"></i>' : ''}
                 </div>`;
    }

    // Sub-label: escape safely
    const subLabel = item.inline_display
      ? `<span class="app-path inline-result">${escapeHtml(item.inline_display)}</span>`
      : (item.path ? `<span class="app-path">${escapeHtml(item.path)}</span>` : "");

    li.innerHTML = `
      ${iconHTML}
      <div class="app-info">
        <div class="app-header">
          <span class="app-name">${escapeHtml(item.name)}</span>
        </div>
        ${subLabel}
      </div>
    `;

    if (item.path) {
      li.onclick = () => onLaunch(item.path);
    }

    resultsList.appendChild(li);

    if (index === selectedIndex) {
      li.scrollIntoView({ block: "nearest" });
    }
  });
}

export function updateSelection(resultsList, selectedIndex) {
  const items = resultsList.querySelectorAll(".result-item");
  items.forEach((li, index) => {
    if (index === selectedIndex) {
      li.classList.add("selected");
      li.scrollIntoView({ block: "nearest", behavior: "smooth" });
    } else {
      li.classList.remove("selected");
    }
  });
}
