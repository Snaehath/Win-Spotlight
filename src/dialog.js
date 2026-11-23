// ── Confirm dialog ──────────────────────────────────────────────────────────
// A single in-panel modal. While open it owns the keyboard (capture phase) so
// Enter/Escape never leak to the launcher underneath.

let activeResolve = null;

export function isDialogOpen() {
  return activeResolve !== null;
}

/** Resolve any open dialog as "cancelled" (e.g. when the window is re-summoned). */
export function cancelDialog() {
  activeResolve?.(false);
}

/**
 * @param {string} title
 * @param {string} message
 * @param {{ okText?: string, cancelText?: string, danger?: boolean, returnFocus?: HTMLElement }} [options]
 * @returns {Promise<boolean>}
 */
export function showConfirm(title, message, options = {}) {
  cancelDialog();

  const overlay = document.getElementById("confirm-dialog");
  const btnOk = document.getElementById("dialog-ok");
  const btnCancel = document.getElementById("dialog-cancel");
  const container = overlay.closest(".container");

  document.getElementById("dialog-title").textContent = title;
  document.getElementById("dialog-message").textContent = message;
  btnOk.textContent = options.okText || "Confirm";
  btnCancel.textContent = options.cancelText || "Cancel";
  overlay.classList.toggle("danger", Boolean(options.danger));
  overlay.classList.remove("hidden");
  container?.classList.add("overlay-open");
  btnOk.focus();

  return new Promise((resolve) => {
    const onKey = (e) => {
      e.stopImmediatePropagation();
      if (e.repeat) {
        e.preventDefault();
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        finish(false);
      } else if (e.key === "Enter") {
        e.preventDefault();
        finish(document.activeElement !== btnCancel);
      } else if (e.key === "Tab" || e.key === "ArrowLeft" || e.key === "ArrowRight") {
        e.preventDefault();
        (document.activeElement === btnOk ? btnCancel : btnOk).focus();
      }
    };
    const onOk = () => finish(true);
    const onCancel = () => finish(false);

    function finish(result) {
      window.removeEventListener("keydown", onKey, true);
      btnOk.removeEventListener("click", onOk);
      btnCancel.removeEventListener("click", onCancel);
      overlay.classList.add("hidden");
      if (!document.querySelector(".overlay:not(.hidden)")) {
        container?.classList.remove("overlay-open");
      }
      activeResolve = null;
      options.returnFocus?.focus();
      resolve(result);
    }

    activeResolve = finish;
    window.addEventListener("keydown", onKey, true);
    btnOk.addEventListener("click", onOk);
    btnCancel.addEventListener("click", onCancel);
  });
}
