// Pure parsers for passwd/group and account names. QML imports this; Node tests eval it.

var RESERVED_USERNAMES = {
  root: true,
  bin: true,
  daemon: true,
  mail: true,
  ftp: true,
  http: true,
  nobody: true,
  dbus: true,
  "systemd-coredump": true,
  "systemd-network": true,
  "systemd-oom": true,
  "systemd-journal-remote": true,
  "systemd-resolve": true,
  "systemd-timesync": true,
  tss: true,
  uuidd: true,
  alpm: true,
  git: true,
  avahi: true,
  cups: true,
  "cups-browsed": true,
  lp: true,
  _talkd: true,
  polkitd: true,
  rtkit: true,
  qemu: true,
  brltty: true,
  gluster: true,
  rpc: true,
  "libvirt-qemu": true,
  pcscd: true,
  "nvidia-persistenced": true,
  sddm: true,
};

var ALWAYS_GROUPS = { wheel: true, docker: true };

function reservedUsername(name) {
  return RESERVED_USERNAMES[String(name || "")] === true;
}

function isUsername(raw) {
  var s = String(raw || "");
  if (s.length < 1 || s.length > 32) return false;
  if (!/^[a-z_][a-z0-9_-]*$/.test(s)) return false;
  if (s.charAt(s.length - 1) === "-") return false;
  if (reservedUsername(s)) return false;
  return true;
}

function parseUsername(raw) {
  var s = String(raw || "").replace(/^\s+|\s+$/g, "");
  return isUsername(s) ? s : "";
}

function isGroupName(raw) {
  var s = String(raw || "");
  if (s.length < 1 || s.length > 32) return false;
  if (!/^[a-z_][a-z0-9_-]*$/.test(s)) return false;
  if (s.charAt(s.length - 1) === "-") return false;
  if (s === "root" || s === "nobody" || s === "nogroup") return false;
  return true;
}

function parseGroupName(raw) {
  var s = String(raw || "").replace(/^\s+|\s+$/g, "");
  return isGroupName(s) ? s : "";
}

function isFullName(raw) {
  var s = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (s.length > 256) return false;
  if (s.charAt(0) === "-") return false;
  if (/[:\n\r,]/.test(s)) return false;
  return true;
}

function parseFullName(raw) {
  var s = String(raw || "").replace(/^\s+|\s+$/g, "");
  return isFullName(s) ? s : "";
}

function isHumanUid(uid) {
  var n = Number(uid);
  if (!isFinite(n)) return false;
  n = Math.round(n);
  return n >= 1000 && n < 65534;
}

function isRemovableGid(gid, name) {
  if (String(name || "") === "wheel" || String(name || "") === "docker") return false;
  var n = Number(gid);
  if (!isFinite(n)) return false;
  n = Math.round(n);
  return n >= 1000 && n < 65534;
}

function gecosFullName(gecos) {
  var text = String(gecos || "");
  var i = text.indexOf(",");
  if (i === -1) return text;
  return text.substring(0, i);
}

function parsePasswd(text, currentUser) {
  var current = String(currentUser || "");
  var users = [];
  var lines = String(text || "").split("\n");
  var i;
  for (i = 0; i < lines.length; i++) {
    var line = lines[i];
    if (!line || line.charAt(0) === "#") continue;
    var parts = line.split(":");
    if (parts.length < 7) continue;
    var name = parts[0];
    var uid = Number(parts[2]);
    if (!isFinite(uid)) continue;
    uid = Math.round(uid);
    if (name !== current && !isHumanUid(uid)) continue;
    users.push({
      name: name,
      uid: uid,
      gid: Math.round(Number(parts[3])) || 0,
      fullName: gecosFullName(parts[4]),
      home: parts[5],
      shell: parts[6],
      wheel: false,
      current: name === current,
    });
  }
  return users;
}

function parseGroup(text) {
  var groups = [];
  var lines = String(text || "").split("\n");
  var i;
  for (i = 0; i < lines.length; i++) {
    var line = lines[i];
    if (!line || line.charAt(0) === "#") continue;
    var parts = line.split(":");
    if (parts.length < 4) continue;
    var name = parts[0];
    var gid = Number(parts[2]);
    if (!isFinite(gid)) continue;
    gid = Math.round(gid);
    var members = [];
    var raw = parts[3] || "";
    if (raw) {
      var bits = raw.split(",");
      var j;
      for (j = 0; j < bits.length; j++) {
        var m = bits[j].replace(/^\s+|\s+$/g, "");
        if (m) members.push(m);
      }
    }
    groups.push({ name: name, gid: gid, members: members });
  }
  return groups;
}

function humanNames(users) {
  var map = {};
  var i;
  var list = users || [];
  for (i = 0; i < list.length; i++) {
    if (list[i] && list[i].name) map[list[i].name] = true;
  }
  return map;
}

function attachWheel(users, groups) {
  var wheel = {};
  var i;
  var g;
  var list = groups || [];
  for (i = 0; i < list.length; i++) {
    if (!list[i] || list[i].name !== "wheel") continue;
    g = list[i].members || [];
    var j;
    for (j = 0; j < g.length; j++) wheel[g[j]] = true;
  }
  var out = [];
  var people = users || [];
  for (i = 0; i < people.length; i++) {
    var row = people[i] || {};
    out.push({
      name: row.name,
      uid: row.uid,
      gid: row.gid,
      fullName: row.fullName,
      home: row.home,
      shell: row.shell,
      wheel: wheel[row.name] === true,
      current: row.current === true,
    });
  }
  return out;
}

function isPrivateUserGroup(group, users) {
  var g = group || {};
  var name = String(g.name || "");
  var gid = Number(g.gid);
  if (!name || !isFinite(gid)) return false;
  gid = Math.round(gid);
  var list = users || [];
  var i;
  for (i = 0; i < list.length; i++) {
    var u = list[i];
    if (u && u.name === name && u.gid === gid) return true;
  }
  return false;
}

function groupHasMember(group, user) {
  var members = (group && group.members) || [];
  var want = String(user || "");
  var i;
  if (!want) return false;
  for (i = 0; i < members.length; i++) {
    if (members[i] === want) return true;
  }
  return false;
}

function visibleGroups(groups, users) {
  var humans = humanNames(users);
  var list = groups || [];
  var out = [];
  var i;
  for (i = 0; i < list.length; i++) {
    var g = list[i];
    if (!g || !g.name) continue;
    if (ALWAYS_GROUPS[g.name] !== true && isPrivateUserGroup(g, users)) continue;
    var keep = ALWAYS_GROUPS[g.name] === true;
    if (!keep && isRemovableGid(g.gid, g.name)) keep = true;
    if (!keep) {
      var members = g.members || [];
      var j;
      for (j = 0; j < members.length; j++) {
        if (humans[members[j]]) {
          keep = true;
          break;
        }
      }
    }
    if (!keep) continue;
    out.push({ name: g.name, gid: g.gid, members: g.members || [] });
  }
  return out;
}

function parseAccounts(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  var currentUser = parseUsername(src.currentUser) || String(src.currentUser || "");
  var users = attachWheel(parsePasswd(src.passwd, currentUser), parseGroup(src.group));
  var groups = visibleGroups(parseGroup(src.group), users);
  return {
    currentUser: currentUser,
    avatarPath: String(src.avatarPath || ""),
    users: users,
    groups: groups,
  };
}

function profileTitle(fullName, user) {
  var name = String(fullName || "").replace(/^\s+|\s+$/g, "");
  if (name) return name;
  var login = String(user || "").replace(/^\s+|\s+$/g, "");
  return login || "Account";
}

function profileHost(user, hostname) {
  var login = String(user || "").replace(/^\s+|\s+$/g, "");
  var host = String(hostname || "").replace(/^\s+|\s+$/g, "");
  if (host.indexOf("..") !== -1 || host.indexOf(" ") !== -1) host = "";
  if (!login) return "";
  if (!host) return login;
  return login + "@" + host;
}

function faceRowDescription(path) {
  var file = String(path || "");
  if (file) return file + ". Omarchy's greeter does not draw this face.";
  return "No face is set. Choose a PNG or JPEG. Omarchy's greeter does not draw it yet, but AccountsService and ~/.face.icon keep the file.";
}

function validHostname(raw) {
  var hostname = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (
    !/^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?)*$/.test(
      hostname,
    ) ||
    hostname.length > 253
  )
    return "";
  return hostname;
}

function validFullName(raw) {
  var fullName = String(raw || "");
  if (fullName.length > 256 || fullName.charAt(0) === "-" || /[:\n\r,]/.test(fullName)) return "";
  return fullName;
}

function validAvatarPath(raw) {
  var avatarPath = String(raw || "");
  if (avatarPath.charAt(0) !== "/" || avatarPath.indexOf("..") !== -1) return "";
  return avatarPath;
}

function applyAccountPatch(cur, parsed) {
  var src = parsed && typeof parsed === "object" ? parsed : {};
  var now = cur && typeof cur === "object" ? cur : {};
  var out = {
    hostname: String(now.hostname || ""),
    fullName: String(now.fullName || ""),
    currentUser: String(now.currentUser || ""),
    avatarPath: String(now.avatarPath || ""),
    users: Array.isArray(now.users) ? now.users : [],
    groups: Array.isArray(now.groups) ? now.groups : [],
  };
  if ("hostname" in src) out.hostname = validHostname(src.hostname);
  if ("fullName" in src) out.fullName = validFullName(src.fullName);
  if ("currentUser" in src) out.currentUser = String(src.currentUser || "");
  if ("avatarPath" in src) out.avatarPath = validAvatarPath(src.avatarPath);
  if ("users" in src) out.users = Array.isArray(src.users) ? src.users : [];
  if ("groups" in src) out.groups = Array.isArray(src.groups) ? src.groups : [];
  return out;
}

function seedFromDisk(src) {
  var raw = src && typeof src === "object" ? src : {};
  var currentUser = parseUsername(raw.currentUser) || String(raw.currentUser || "");
  var hostname = validHostname(raw.hostname);
  var home = String(raw.home || "");
  var users = parsePasswd(raw.passwd, currentUser);
  var fullName = "";
  var i;
  for (i = 0; i < users.length; i++) {
    if (!users[i] || users[i].current !== true) continue;
    fullName = validFullName(users[i].fullName || "");
    if (!home) home = String(users[i].home || "");
    break;
  }
  var groups = visibleGroups(parseGroup(raw.group), attachWheel(users, parseGroup(raw.group)));
  var avatarPath = "";
  if (typeof raw.exists === "function") avatarPath = pickAvatarPath(home, currentUser, raw.exists);
  return {
    hostname: hostname,
    fullName: fullName,
    currentUser: currentUser,
    avatarPath: avatarPath,
    users: attachWheel(users, parseGroup(raw.group)),
    groups: groups,
  };
}

function pickAvatarPath(home, username, exists) {
  var user = String(username || "");
  var dir = String(home || "");
  var candidates = [];
  if (user) candidates.push("/var/lib/AccountsService/icons/" + user);
  if (dir) {
    candidates.push(dir + "/.face.icon");
    candidates.push(dir + "/.face");
  }
  var i;
  for (i = 0; i < candidates.length; i++) {
    if (exists(candidates[i])) return candidates[i];
  }
  return "";
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    reservedUsername: reservedUsername,
    isUsername: isUsername,
    parseUsername: parseUsername,
    isGroupName: isGroupName,
    parseGroupName: parseGroupName,
    isFullName: isFullName,
    parseFullName: parseFullName,
    isHumanUid: isHumanUid,
    isRemovableGid: isRemovableGid,
    gecosFullName: gecosFullName,
    parsePasswd: parsePasswd,
    parseGroup: parseGroup,
    attachWheel: attachWheel,
    isPrivateUserGroup: isPrivateUserGroup,
    groupHasMember: groupHasMember,
    visibleGroups: visibleGroups,
    parseAccounts: parseAccounts,
    profileTitle: profileTitle,
    profileHost: profileHost,
    faceRowDescription: faceRowDescription,
    pickAvatarPath: pickAvatarPath,
    validHostname: validHostname,
    applyAccountPatch: applyAccountPatch,
    seedFromDisk: seedFromDisk,
  };
}
