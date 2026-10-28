// The Defaults hub's pickers: the programs each one offers, and the list a
// picker shows. The Rust window's Defaults page is held to this by
// frontend/tests/defaults_parity.rs.

function browserOptions() {
  return [
    { value: "chromium", label: "Chromium" },
    { value: "chrome", label: "Chrome" },
    { value: "brave", label: "Brave" },
    { value: "brave-origin", label: "Brave Origin" },
    { value: "edge", label: "Edge" },
    { value: "firefox", label: "Firefox" },
    { value: "zen", label: "Zen" },
  ];
}

function terminalOptions() {
  return [
    { value: "alacritty", label: "Alacritty" },
    { value: "foot", label: "Foot" },
    { value: "ghostty", label: "Ghostty" },
    { value: "kitty", label: "Kitty" },
  ];
}

function editorOptions() {
  return [
    { value: "nvim", label: "Neovim" },
    { value: "code", label: "VS Code" },
    { value: "cursor", label: "Cursor" },
    { value: "zed", label: "Zed" },
    { value: "sublime_text", label: "Sublime Text" },
    { value: "helix", label: "Helix" },
    { value: "vim", label: "Vim" },
    { value: "emacs", label: "Emacs" },
  ];
}

// Omarchy calls Zed's program zeditor; the picker calls it zed.
function editorValue(editor) {
  return editor === "zeditor" ? "zed" : editor;
}

// Which editors are installed, by the picker's values.
function editorsInstalled(editors) {
  editors = editors || {};
  return {
    nvim: editors.nvim,
    code: editors.code,
    cursor: editors.cursor,
    zed: editors.zeditor,
    sublime_text: editors.sublime_text,
    helix: editors.helix,
    vim: editors.vim,
    emacs: editors.emacs,
  };
}

// A file-type picker: the installed handlers, and the current one when it is
// not among them.
function mimeOptions(current, list) {
  var out = [];
  if (list instanceof Array) {
    for (var i = 0; i < list.length; i++) out.push(list[i]);
  }
  var id = String(current || "");
  if (!id) return out;
  for (var j = 0; j < out.length; j++) {
    if (out[j] && out[j].value === id) return out;
  }
  out.push({ value: id, label: id });
  return out;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    browserOptions: browserOptions,
    terminalOptions: terminalOptions,
    editorOptions: editorOptions,
    editorValue: editorValue,
    editorsInstalled: editorsInstalled,
    mimeOptions: mimeOptions,
  };
}
