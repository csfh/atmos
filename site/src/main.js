import "./style.css";

function copyText(text) {
  if (navigator.clipboard && window.isSecureContext) {
    return navigator.clipboard.writeText(text);
  }
  return new Promise((resolve, reject) => {
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.left = "-9999px";
    document.body.appendChild(area);
    area.select();
    try {
      document.execCommand("copy");
      resolve();
    } catch (err) {
      reject(err);
    } finally {
      area.remove();
    }
  });
}

document.querySelectorAll(".copy").forEach((button) => {
  const label = button.textContent;
  const target = document.querySelector(button.getAttribute("data-copy") || "");

  if (!target) return;

  button.addEventListener("click", async () => {
    const text = target.textContent.replace(/\s+/g, " ").trim();

    try {
      await copyText(text);
    } catch {
      return;
    }

    button.textContent = "Copied";
    button.classList.add("copied");
    window.setTimeout(() => {
      button.textContent = label;
      button.classList.remove("copied");
    }, 1600);
  });
});
