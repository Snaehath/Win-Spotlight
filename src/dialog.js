// ── Dialog Management ────────────────────────────────────────────────────────
export async function showConfirm(title, message, searchInput) {
  const overlay = document.getElementById("confirm-dialog");
  const titleEl = document.getElementById("dialog-title");
  const msgEl = document.getElementById("dialog-message");
  const btnOk = document.getElementById("dialog-ok");
  const btnCancel = document.getElementById("dialog-cancel");

  titleEl.innerText = title;
  msgEl.innerText = message;
  overlay.classList.remove("hidden");

  return new Promise((resolve) => {
    const handleOk = () => {
      cleanup();
      resolve(true);
    };
    const handleCancel = () => {
      cleanup();
      resolve(false);
    };
    const cleanup = () => {
      btnOk.removeEventListener("click", handleOk);
      btnCancel.removeEventListener("click", handleCancel);
      overlay.classList.add("hidden");
      // refocus search
      if (searchInput) searchInput.focus();
    };
    btnOk.addEventListener("click", handleOk, { once: true });
    btnCancel.addEventListener("click", handleCancel, { once: true });
    
    const handleKey = (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        window.removeEventListener("keydown", handleKey);
        handleOk();
      } else if (e.key === "Escape") {
        e.preventDefault();
        window.removeEventListener("keydown", handleKey);
        handleCancel();
      }
    };
    window.addEventListener("keydown", handleKey);
  });
}
