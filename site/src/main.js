import "./style.css";

const command = document.querySelector("#install-cmd");
const button = document.querySelector("#copy-btn");

if (command && button) {
  const label = button.textContent;

  button.addEventListener("click", async () => {
    const text = command.textContent.trim();

    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const range = document.createRange();
      range.selectNodeContents(command);
      const selection = window.getSelection();
      selection?.removeAllRanges();
      selection?.addRange(range);
      document.execCommand("copy");
      selection?.removeAllRanges();
    }

    button.textContent = "Copied";
    button.classList.add("copied");
    window.setTimeout(() => {
      button.textContent = label;
      button.classList.remove("copied");
    }, 1600);
  });
}
