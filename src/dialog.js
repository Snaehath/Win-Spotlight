// ── Dialog Management ────────────────────────────────────────────────────────
export async function showConfirm(title, message, searchInput, options = {}) {
  const overlay = document.getElementById("confirm-dialog");
  const titleEl = document.getElementById("dialog-title");
  const msgEl = document.getElementById("dialog-message");
  const btnOk = document.getElementById("dialog-ok");
  const btnCancel = document.getElementById("dialog-cancel");

  const origOkText = btnOk.innerText;
  const origCancelText = btnCancel.innerText;

  if (options.okText) btnOk.innerText = options.okText;
  if (options.cancelText) btnCancel.innerText = options.cancelText;

  titleEl.innerText = title;
  msgEl.innerText = message;
  overlay.classList.remove("hidden");

  return new Promise((resolve) => {
    let handleKey;

    const cleanup = () => {
      btnOk.removeEventListener("click", handleOk);
      btnCancel.removeEventListener("click", handleCancel);
      if (handleKey) {
        window.removeEventListener("keydown", handleKey);
      }
      btnOk.innerText = origOkText;
      btnCancel.innerText = origCancelText;
      overlay.classList.add("hidden");
      // refocus search
      if (searchInput) searchInput.focus();
    };

    const handleOk = () => {
      cleanup();
      resolve(true);
    };

    const handleCancel = () => {
      cleanup();
      resolve(false);
    };

    btnOk.addEventListener("click", handleOk, { once: true });
    btnCancel.addEventListener("click", handleCancel, { once: true });
    
    handleKey = (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        handleOk();
      } else if (e.key === "Escape") {
        e.preventDefault();
        handleCancel();
      }
    };
    window.addEventListener("keydown", handleKey);
  });
}
