//! The Defaults hub's logic, as `services/Defaults.js` and the QML page have it: the six pickers,
//! the programs each one offers once the snapshot says which are installed, the value each shows,
//! and the `settings.set` a choice sends. `tests/defaults_parity.rs` runs the JavaScript under Node
//! against this, so a change to either side fails until the other follows.

use serde_json::Value;

use crate::settings::{self, Held, Known, Settings};

const SYSTEM: &str = "settings.system";
const HOST: &str = "settings.host";

const fn held(key: &'static str, topic: &'static str, unread: &'static str) -> Held {
    Held { key, topic, unread }
}

pub const HELD: &[Held] = &[
    held("browser", SYSTEM, "The default browser could not be read"),
    held("terminal", SYSTEM, "The default terminal could not be read"),
    held("editor", SYSTEM, "The default editor could not be read"),
    held("browsers", HOST, "The installed browsers could not be read"),
    held(
        "terminals",
        HOST,
        "The installed terminals could not be read",
    ),
    held("editors", HOST, "The installed editors could not be read"),
    held("mimePdf", SYSTEM, "The PDF handler could not be read"),
    held("mimeImage", SYSTEM, "The image handler could not be read"),
    held("mimeVideo", SYSTEM, "The video handler could not be read"),
    held(
        "mimePdfOptions",
        SYSTEM,
        "The installed PDF handlers could not be read",
    ),
    held(
        "mimeImageOptions",
        SYSTEM,
        "The installed image viewers could not be read",
    ),
    held(
        "mimeVideoOptions",
        SYSTEM,
        "The installed video players could not be read",
    ),
];

fn held_key(key: &str) -> &'static Held {
    HELD.iter()
        .find(|h| h.key == key)
        .expect("a key the page holds")
}

/// A program a picker offers: the value it writes and the name it shows.
pub type Choice = (&'static str, &'static str);

pub const BROWSERS: &[Choice] = &[
    ("chromium", "Chromium"),
    ("chrome", "Chrome"),
    ("brave", "Brave"),
    ("brave-origin", "Brave Origin"),
    ("edge", "Edge"),
    ("firefox", "Firefox"),
    ("zen", "Zen"),
];

pub const TERMINALS: &[Choice] = &[
    ("alacritty", "Alacritty"),
    ("foot", "Foot"),
    ("ghostty", "Ghostty"),
    ("kitty", "Kitty"),
];

pub const EDITORS: &[Choice] = &[
    ("nvim", "Neovim"),
    ("code", "VS Code"),
    ("cursor", "Cursor"),
    ("zed", "Zed"),
    ("sublime_text", "Sublime Text"),
    ("helix", "Helix"),
    ("vim", "Vim"),
    ("emacs", "Emacs"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Applications,
    Advanced,
}

impl Group {
    pub fn title(self) -> &'static str {
        match self {
            Group::Applications => "Applications",
            Group::Advanced => "Advanced",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Group::Applications => {
                "Omarchy writes these as the XDG defaults. Only programs that are installed show up in the lists."
            }
            Group::Advanced => {
                "XDG defaults for PDFs, images, and video. Browser, terminal, and editor stay in the group above."
            }
        }
    }
}

/// Where a picker's programs come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A fixed list, less what the snapshot key says is not installed (`installedOptions`).
    Installed {
        all: &'static [Choice],
        key: &'static str,
    },
    /// The handlers the snapshot key lists, and the current one when it is not among them
    /// (`mimeOptions`).
    Handlers { key: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Picker {
    pub id: &'static str,
    pub group: Group,
    pub label: &'static str,
    pub description: &'static str,
    pub hint: &'static str,
    /// The settings domain it writes, which is also the snapshot key it shows.
    pub domain: &'static str,
    pub source: Source,
}

pub const PICKERS: &[Picker] = &[
    Picker {
        id: "browser",
        group: Group::Applications,
        label: "Browser",
        description: "The browser that opens links and web apps. Only installed browsers show up.",
        hint: "omarchy default browser",
        domain: "browser",
        source: Source::Installed {
            all: BROWSERS,
            key: "browsers",
        },
    },
    Picker {
        id: "terminal",
        group: Group::Applications,
        label: "Terminal",
        description: "The terminal other apps launch when they need a console. Only installed terminals show up.",
        hint: "omarchy default terminal",
        domain: "terminal",
        source: Source::Installed {
            all: TERMINALS,
            key: "terminals",
        },
    },
    Picker {
        id: "editor",
        group: Group::Applications,
        label: "Editor",
        description: "The editor Omarchy opens when you ask to edit a file. Only installed editors show up.",
        hint: "omarchy default editor",
        domain: "editor",
        source: Source::Installed {
            all: EDITORS,
            key: "editors",
        },
    },
    Picker {
        id: "pdf",
        group: Group::Advanced,
        label: "PDF",
        description: "The program that opens PDF files.",
        hint: "xdg-mime default application/pdf",
        domain: "mimePdf",
        source: Source::Handlers {
            key: "mimePdfOptions",
        },
    },
    Picker {
        id: "image",
        group: Group::Advanced,
        label: "Images",
        description: "The program that opens pictures.",
        hint: "xdg-mime default image/png",
        domain: "mimeImage",
        source: Source::Handlers {
            key: "mimeImageOptions",
        },
    },
    Picker {
        id: "video",
        group: Group::Advanced,
        label: "Video",
        description: "The program that opens videos.",
        hint: "xdg-mime default video/mp4",
        domain: "mimeVideo",
        source: Source::Handlers {
            key: "mimeVideoOptions",
        },
    },
];

pub fn by_id(id: &str) -> Option<&'static Picker> {
    PICKERS.iter().find(|p| p.id == id)
}

/// The pickers in their groups, in order.
pub fn groups() -> [(Group, Vec<&'static Picker>); 2] {
    [Group::Applications, Group::Advanced]
        .map(|g| (g, PICKERS.iter().filter(|p| p.group == g).collect()))
}

/// Omarchy calls Zed's program `zeditor`; the picker calls it `zed` (`editorValue`).
pub fn editor_value(editor: &str) -> &str {
    if editor == "zeditor" { "zed" } else { editor }
}

/// The snapshot's name for the program a picker value names: the editors key says `zeditor`
/// (`editorsInstalled`).
fn installed_name<'a>(picker: &Picker, value: &'a str) -> &'a str {
    if picker.domain == "editor" && value == "zed" {
        "zeditor"
    } else {
        value
    }
}

/// A snapshot value as the text QML's `String()` of it would show, for the values a picker can
/// hold: a string, and nothing for anything else.
fn text(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or_default().to_owned()
}

impl Picker {
    /// What the window knows of the key it shows.
    pub fn known<'a>(&self, settings: &'a Settings) -> Known<'a> {
        settings.known(held_key(self.domain))
    }

    /// The value it shows, once its key has been read.
    pub fn current(&self, settings: &Settings) -> Option<String> {
        let value = settings.value(held_key(self.domain))?;
        let text = text(Some(value));
        Some(if self.domain == "editor" {
            editor_value(&text).to_owned()
        } else {
            text
        })
    }

    /// The programs it offers, as (value, label).
    pub fn options(&self, settings: &Settings) -> Vec<(String, String)> {
        match self.source {
            Source::Installed { all, key } => {
                let available = settings.value(held_key(key)).and_then(Value::as_object);
                all.iter()
                    .filter(|(value, _)| {
                        available
                            .and_then(|a| a.get(installed_name(self, value)))
                            .is_none_or(|v| v != &Value::Bool(false))
                    })
                    .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
                    .collect()
            }
            Source::Handlers { key } => {
                let mut out: Vec<(String, String)> = settings
                    .value(held_key(key))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|item| {
                        let value = item.get("value")?.as_str()?;
                        let label = item.get("label").and_then(Value::as_str).unwrap_or(value);
                        Some((value.to_owned(), label.to_owned()))
                    })
                    .collect();
                let current = text(settings.value(held_key(self.domain)));
                if !current.is_empty() && !out.iter().any(|(v, _)| *v == current) {
                    out.push((current.clone(), current));
                }
                out
            }
        }
    }

    /// The `settings.set` choosing `chosen` sends, or nothing when it would change nothing or is
    /// not a program the writer takes: an empty choice, the one shown, or a handler that is not a
    /// desktop file (`SettingTable.plan`, `setMimeDefault`). The editor is compared both as the
    /// picker shows it and as Omarchy names it.
    pub fn choose(&self, chosen: &str, settings: &Settings) -> Option<Value> {
        let raw = text(settings.value(held_key(self.domain)));
        let shown = self.current(settings).unwrap_or_default();
        if chosen.is_empty() || chosen == shown || chosen == raw {
            return None;
        }
        if let Source::Handlers { .. } = self.source
            && !is_desktop_file(chosen)
        {
            return None;
        }
        Some(settings::write(
            self.domain,
            &Value::String(chosen.to_owned()),
        ))
    }
}

/// `^[A-Za-z0-9._-]+\.desktop$`, the handlers `setMimeDefault` takes.
fn is_desktop_file(name: &str) -> bool {
    name.strip_suffix(".desktop").is_some_and(|stem| {
        !stem.is_empty()
            && stem
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn settings(doc: Value) -> Settings {
        let mut s = Settings::default();
        s.adopt(SYSTEM, 1, &doc);
        s.adopt(HOST, 1, &doc);
        s
    }

    #[test]
    fn a_program_the_snapshot_says_is_missing_is_not_offered() {
        let s = settings(json!({"browsers": {"chromium": true, "chrome": false, "zen": false}}));
        let offered: Vec<_> = by_id("browser")
            .unwrap()
            .options(&s)
            .into_iter()
            .map(|(v, _)| v)
            .collect();
        assert_eq!(
            offered,
            ["chromium", "brave", "brave-origin", "edge", "firefox"]
        );
        assert_eq!(
            by_id("terminal").unwrap().options(&s).len(),
            TERMINALS.len()
        );
    }

    #[test]
    fn zed_is_zeditor_to_omarchy() {
        let s = settings(json!({"editor": "zeditor", "editors": {"zeditor": false}}));
        let editor = by_id("editor").unwrap();
        assert_eq!(editor.current(&s).as_deref(), Some("zed"));
        assert!(!editor.options(&s).iter().any(|(v, _)| v == "zed"));
        assert_eq!(editor.choose("zed", &s), None, "already chosen");
    }

    #[test]
    fn a_handler_not_in_the_list_is_kept_and_only_a_desktop_file_is_written() {
        let s = settings(json!({"mimePdf": "foo.desktop",
            "mimePdfOptions": [{"value": "org.gnome.Evince.desktop", "label": "Evince"}]}));
        let pdf = by_id("pdf").unwrap();
        assert_eq!(
            pdf.options(&s),
            [
                ("org.gnome.Evince.desktop".to_owned(), "Evince".to_owned()),
                ("foo.desktop".to_owned(), "foo.desktop".to_owned())
            ]
        );
        assert_eq!(pdf.choose("rm -rf", &s), None);
        assert_eq!(pdf.choose(".desktop", &s), None);
        assert_eq!(
            pdf.choose("org.gnome.Evince.desktop", &s),
            Some(json!({"op": "settings.set", "domain": "mimePdf",
                "value": "org.gnome.Evince.desktop", "readback": ["mimePdf"]}))
        );
    }

    #[test]
    fn the_groups_hold_every_picker() {
        let [(apps, a), (adv, b)] = groups();
        assert_eq!((apps, adv), (Group::Applications, Group::Advanced));
        assert_eq!(a.len() + b.len(), PICKERS.len());
    }
}
