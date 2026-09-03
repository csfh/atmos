// Optional Omarchy installers. Prefs hides nothing in the catalog; the page
// picks Install or Remove from snapshot presence and the remove argv.

function catalog() {
  return [
    {
      id: "chrome",
      group: "browsers",
      label: "Chrome",
      hint: "omarchy install browser chrome",
      install: ["omarchy", "install", "browser", "chrome"],
      remove: ["omarchy", "remove", "browser", "chrome"],
      present: ["browsers", "chrome"],
    },
    {
      id: "brave",
      group: "browsers",
      label: "Brave",
      hint: "omarchy install browser brave",
      install: ["omarchy", "install", "browser", "brave"],
      remove: ["omarchy", "remove", "browser", "brave"],
      present: ["browsers", "brave"],
    },
    {
      id: "brave-origin",
      group: "browsers",
      label: "Brave Origin",
      hint: "omarchy install browser brave-origin",
      install: ["omarchy", "install", "browser", "brave-origin"],
      remove: ["omarchy", "remove", "browser", "brave-origin"],
      present: ["browsers", "brave-origin"],
    },
    {
      id: "edge",
      group: "browsers",
      label: "Edge",
      hint: "omarchy install browser edge",
      install: ["omarchy", "install", "browser", "edge"],
      remove: ["omarchy", "remove", "browser", "edge"],
      present: ["browsers", "edge"],
    },
    {
      id: "firefox",
      group: "browsers",
      label: "Firefox",
      hint: "omarchy install browser firefox",
      install: ["omarchy", "install", "browser", "firefox"],
      remove: ["omarchy", "remove", "browser", "firefox"],
      present: ["browsers", "firefox"],
    },
    {
      id: "zen",
      group: "browsers",
      label: "Zen",
      hint: "omarchy install browser zen",
      install: ["omarchy", "install", "browser", "zen"],
      remove: ["omarchy", "remove", "browser", "zen"],
      present: ["browsers", "zen"],
    },

    {
      id: "alacritty",
      group: "terminals",
      label: "Alacritty",
      hint: "omarchy install terminal alacritty",
      install: ["omarchy", "install", "terminal", "alacritty"],
      present: ["terminals", "alacritty"],
    },
    {
      id: "foot",
      group: "terminals",
      label: "foot",
      hint: "omarchy install terminal foot",
      install: ["omarchy", "install", "terminal", "foot"],
      present: ["terminals", "foot"],
    },
    {
      id: "ghostty",
      group: "terminals",
      label: "Ghostty",
      hint: "omarchy install terminal ghostty",
      install: ["omarchy", "install", "terminal", "ghostty"],
      present: ["terminals", "ghostty"],
    },
    {
      id: "kitty",
      group: "terminals",
      label: "Kitty",
      hint: "omarchy install terminal kitty",
      install: ["omarchy", "install", "terminal", "kitty"],
      present: ["terminals", "kitty"],
    },

    {
      id: "emacs",
      group: "editors",
      label: "Emacs",
      hint: "omarchy install editor emacs",
      install: ["omarchy", "install", "editor", "emacs"],
      present: ["editors", "emacs"],
    },
    {
      id: "helix",
      group: "editors",
      label: "Helix",
      hint: "omarchy install editor helix",
      install: ["omarchy", "install", "editor", "helix"],
      present: ["editors", "helix"],
    },
    {
      id: "vscode",
      group: "editors",
      label: "VS Code",
      hint: "omarchy install editor vscode",
      install: ["omarchy", "install", "editor", "vscode"],
      present: ["editors", "code"],
    },
    {
      id: "zed",
      group: "editors",
      label: "Zed",
      hint: "omarchy install editor zed",
      install: ["omarchy", "install", "editor", "zed"],
      present: ["editors", "zeditor"],
    },

    {
      id: "1password",
      group: "services",
      label: "1Password",
      hint: "omarchy install service 1password",
      install: ["omarchy", "install", "service", "1password"],
      remove: ["omarchy", "remove", "service", "1password"],
      present: ["services", "onepassword"],
    },
    {
      id: "dropbox",
      group: "services",
      label: "Dropbox",
      hint: "omarchy install service dropbox",
      install: ["omarchy", "install", "service", "dropbox"],
      remove: ["omarchy", "remove", "service", "dropbox"],
      present: ["services", "dropbox"],
    },
    {
      id: "nordvpn",
      group: "services",
      label: "NordVPN",
      hint: "omarchy install service nordvpn",
      install: ["omarchy", "install", "service", "nordvpn"],
      present: ["services", "nordvpn"],
    },
    {
      id: "signal",
      group: "services",
      label: "Signal",
      hint: "omarchy install service signal",
      install: ["omarchy", "install", "service", "signal"],
      present: ["services", "signal"],
    },
    {
      id: "spotify",
      group: "services",
      label: "Spotify",
      hint: "omarchy install service spotify",
      install: ["omarchy", "install", "service", "spotify"],
      present: ["services", "spotify"],
    },
    {
      id: "sunshine",
      group: "services",
      label: "Sunshine",
      hint: "omarchy install service sunshine",
      install: ["omarchy", "install", "service", "sunshine"],
      remove: ["omarchy", "remove", "service", "sunshine"],
      present: ["services", "sunshine"],
    },
    {
      id: "tailscale",
      group: "services",
      label: "Tailscale",
      hint: "omarchy install service tailscale",
      install: ["omarchy", "install", "service", "tailscale"],
      remove: ["omarchy", "remove", "service", "tailscale"],
      present: ["services", "tailscale"],
    },

    {
      id: "steam",
      group: "gaming",
      label: "Steam",
      hint: "omarchy install gaming steam",
      install: ["omarchy", "install", "gaming", "steam"],
      remove: ["omarchy", "remove", "gaming", "steam"],
      present: ["gaming", "steam"],
      wipe: true,
    },
    {
      id: "heroic",
      group: "gaming",
      label: "Heroic",
      hint: "omarchy install gaming heroic",
      install: ["omarchy", "install", "gaming", "heroic"],
      remove: ["omarchy", "remove", "gaming", "heroic"],
      present: ["gaming", "heroic"],
      wipe: true,
    },
    {
      id: "lutris",
      group: "gaming",
      label: "Lutris",
      hint: "omarchy install gaming lutris",
      install: ["omarchy", "install", "gaming", "lutris"],
      remove: ["omarchy", "remove", "gaming", "lutris"],
      present: ["gaming", "lutris"],
      wipe: true,
    },
    {
      id: "retroarch",
      group: "gaming",
      label: "RetroArch",
      hint: "omarchy install gaming retroarch",
      install: ["omarchy", "install", "gaming", "retroarch"],
      remove: ["omarchy", "remove", "gaming", "retroarch"],
      present: ["gaming", "retroarch"],
      wipe: true,
    },
    {
      id: "battlenet",
      group: "gaming",
      label: "Battle.net",
      hint: "omarchy install gaming battlenet",
      install: ["omarchy", "install", "gaming", "battlenet"],
      remove: ["omarchy", "remove", "gaming", "battlenet"],
      present: ["gaming", "battlenet"],
      wipe: true,
    },
    {
      id: "geforce-now",
      group: "gaming",
      label: "GeForce Now",
      hint: "omarchy install gaming geforce-now",
      install: ["omarchy", "install", "gaming", "geforce-now"],
      remove: ["omarchy", "remove", "gaming", "geforce-now"],
      present: ["gaming", "geforceNow"],
      wipe: true,
    },
    {
      id: "xbox-cloud",
      group: "gaming",
      label: "Xbox Cloud",
      hint: "omarchy install gaming xbox-cloud",
      install: ["omarchy", "install", "gaming", "xbox-cloud"],
      remove: ["omarchy", "remove", "gaming", "xbox-cloud"],
      present: ["gaming", "xboxCloud"],
    },
    {
      id: "xbox-controllers",
      group: "gaming",
      label: "Xbox controllers",
      hint: "omarchy install gaming xbox-controllers",
      install: ["omarchy", "install", "gaming", "xbox-controllers"],
      remove: ["omarchy", "remove", "gaming", "xbox-controllers"],
      present: ["gaming", "xboxControllers"],
    },

    {
      id: "chatgpt",
      group: "development",
      label: "ChatGPT app",
      hint: "omarchy install ai chatgpt",
      install: ["omarchy", "install", "ai", "chatgpt"],
      present: ["extras", "chatgpt"],
    },
  ];
}

function groupItems(group) {
  var name = String(group || "");
  var list = catalog();
  var out = [];
  for (var i = 0; i < list.length; i++) {
    if (list[i].group === name) out.push(list[i]);
  }
  return out;
}

function lookup(id) {
  var key = String(id || "");
  var list = catalog();
  for (var i = 0; i < list.length; i++) {
    if (list[i].id === key) return list[i];
  }
  return null;
}

function presentIn(item, bags) {
  if (!item || !(item.present instanceof Array) || item.present.length < 2) return false;
  var bag = bags && bags[item.present[0]];
  if (!bag || typeof bag !== "object") return false;
  return bag[item.present[1]] === true;
}

function devEnvs() {
  return [
    { value: "ruby", label: "Ruby" },
    { value: "node", label: "Node" },
    { value: "bun", label: "Bun" },
    { value: "deno", label: "Deno" },
    { value: "go", label: "Go" },
    { value: "php", label: "PHP" },
    { value: "laravel", label: "Laravel" },
    { value: "symfony", label: "Symfony" },
    { value: "python", label: "Python" },
    { value: "elixir", label: "Elixir" },
    { value: "phoenix", label: "Phoenix" },
    { value: "rust", label: "Rust" },
    { value: "java", label: "Java" },
    { value: "zig", label: "Zig" },
    { value: "ocaml", label: "OCaml" },
    { value: "dotnet", label: ".NET" },
    { value: "clojure", label: "Clojure" },
    { value: "scala", label: "Scala" },
  ];
}

function dockerDbs() {
  return [
    { value: "MySQL", label: "MySQL" },
    { value: "PostgreSQL", label: "PostgreSQL" },
    { value: "Redis", label: "Redis" },
    { value: "MongoDB", label: "MongoDB" },
    { value: "MariaDB", label: "MariaDB" },
    { value: "MSSQL", label: "MSSQL" },
  ];
}

function isDevEnv(id) {
  var name = String(id || "");
  var list = devEnvs();
  for (var i = 0; i < list.length; i++) {
    if (list[i].value === name) return true;
  }
  return false;
}

function isDockerDb(id) {
  var name = String(id || "");
  var list = dockerDbs();
  for (var i = 0; i < list.length; i++) {
    if (list[i].value === name) return true;
  }
  return false;
}
