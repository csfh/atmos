//! The Export page's own logic (`pages/ExportPage.qml`): the sections chosen, the paths it
//! reads and writes, what each answer of `atmosd` makes it say, the apply and undo jobs, their
//! progress, and what an apply that ended leaves. What an Omafile holds and how a plan is made
//! is `omafile`; `tests/catalog_export.rs` runs the page's own functions under Node against
//! these.

use serde_json::{Map, Value, json};

use crate::jobs::Job;
use crate::omafile::{self, lines, markdown, plan};
use crate::settings::{Held, Settings};

const HOST: &str = "settings.host";

/// The keys the page reads that no mapped row of another page does.
const EXTRA: &[Held] = &[Held {
    key: "dmiProduct",
    topic: HOST,
    unread: "The hardware model could not be read",
}];

/// The other keys the file's header and the planner read.
const HEADER: &[&str] = &[
    "hostname",
    "atmosRevision",
    "atmosVersion",
    "omarchyVersion",
    "dmiProduct",
    "hyprWorkspaceGestureUnmanaged",
];

fn known(key: &str) -> Option<Held> {
    crate::catalog::control::KEYS
        .iter()
        .map(|(h, _)| h)
        .chain(crate::defaults::HELD)
        .chain(EXTRA)
        .find(|h| h.key == key)
        .copied()
}

/// The snapshot keys the page holds: each key an Omafile carries (a nested one by its top),
/// the options the planner checks a value against, and what the file's header says. A key no
/// page knows is left out (`tests/pins.rs` holds each to the topic that owns it).
pub fn held() -> Vec<Held> {
    let mut out: Vec<Held> = Vec::new();
    let tops = omafile::CATALOG
        .iter()
        .map(|i| i.key.split('.').next().unwrap_or(i.key));
    let options = omafile::CATALOG
        .iter()
        .map(|i| i.options)
        .filter(|o| !o.is_empty());
    for key in tops.chain(options).chain(HEADER.iter().copied()) {
        if let Some(h) = known(key)
            && !out.contains(&h)
        {
            out.push(h);
        }
    }
    out
}

/// The machine's snapshot as the page has it (`Omarchy.snapshotData`): each key it holds that
/// has been read.
pub fn snapshot(settings: &Settings) -> Value {
    let mut out = Map::new();
    for h in held() {
        if let Some(v) = settings.value(&h) {
            out.insert(h.key.to_owned(), v.clone());
        }
    }
    Value::Object(out)
}

/// `Date.toISOString()` of a time in milliseconds.
pub fn iso(ms: u64) -> String {
    let (day, ..) = crate::charts::utc_day(ms as f64);
    let t = ms % 86_400_000;
    format!(
        "{day}T{:02}:{:02}:{:02}.{:03}Z",
        t / 3_600_000,
        t / 60_000 % 60,
        t / 1000 % 60,
        t % 1000
    )
}

/// `FailureJs.errorText(error)`: a failure's message, after its context when the message does
/// not already say it.
pub fn error_text(message: &str, context: Option<&str>) -> String {
    let message = omafile::trim(message);
    let context = omafile::trim(context.unwrap_or_default());
    if message.is_empty() {
        context.to_owned()
    } else if !context.is_empty() && !message.contains(context) {
        format!("{context}: {message}")
    } else {
        message.to_owned()
    }
}

/// `realPath(path)`: a leading tilde is the home folder, as no shell expands it here.
pub fn real_path(path: &str, home: &str) -> String {
    if path == "~" {
        return home.to_owned();
    }
    match path.strip_prefix("~/") {
        Some(rest) => format!("{home}/{rest}"),
        None => path.to_owned(),
    }
}

/// `folderUrl(path)`'s folder: the one the path is in, or home.
pub fn folder(path: &str, home: &str) -> String {
    let text = real_path(path, home);
    match text.rfind('/') {
        Some(cut) if cut > 0 => text[..cut].to_owned(),
        _ => home.to_owned(),
    }
}

/// `withMarkdownSuffix(path)`: a name typed without `.md` gets one.
pub fn with_md(path: &str, home: &str) -> String {
    if path.is_empty() {
        return format!("{home}/{}", omafile::file_name());
    }
    if path.to_ascii_lowercase().ends_with(".md") {
        path.to_owned()
    } else {
        format!("{path}.md")
    }
}

/// What the page holds between its rows (the QML page's properties).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    /// The sections chosen, by id; none until a box is changed (`resetSections`).
    pub sections: Option<Vec<(&'static str, bool)>>,
    pub export_path: String,
    pub import_path: String,
    pub export_status: String,
    pub import_status: String,
    pub written_path: String,
    pub plan: Option<Value>,
    pub last_doc: Option<Value>,
    pub applied_count: usize,
    pub last_backup_dir: String,
    pub last_undo_needs_root: bool,
    pub pending_apply: bool,
    pub undoing: bool,
    pub step_total: usize,
    /// What the page was answered for and is waiting on: `write`, `open` or `read`.
    pub io: Option<&'static str>,
    /// The ends and the cancels of the window's jobs already taken.
    pub seen: (u64, u64),
}

impl State {
    /// The page as it opens: every section that starts chosen, and a fresh name in home.
    pub fn new(home: &str) -> State {
        State {
            export_path: format!("{home}/{}", omafile::file_name()),
            ..State::default()
        }
    }

    /// Whether a section is chosen.
    pub fn chosen(&self, id: &str) -> bool {
        match &self.sections {
            Some(s) => s.iter().any(|(k, on)| *k == id && *on),
            None => omafile::selectable_sections()
                .iter()
                .any(|s| s.id == id && s.by_default),
        }
    }

    /// `setSection(id, on)`.
    pub fn set_section(&mut self, id: &'static str, on: bool) {
        let mut next: Vec<(&'static str, bool)> = omafile::selectable_sections()
            .iter()
            .map(|s| (s.id, self.chosen(s.id)))
            .collect();
        if let Some(s) = next.iter_mut().find(|s| s.0 == id) {
            s.1 = on;
        }
        self.sections = Some(next);
        self.forget_export();
    }

    /// `setAllSections(on)`.
    pub fn set_all(&mut self, on: bool) {
        self.sections = Some(
            omafile::selectable_sections()
                .iter()
                .map(|s| (s.id, on))
                .collect(),
        );
        self.forget_export();
    }

    /// `chosenKeys`: the keys of the sections chosen.
    pub fn chosen_keys(&self) -> Vec<&'static str> {
        let ids: Vec<&str> = omafile::selectable_sections()
            .iter()
            .filter(|s| self.chosen(s.id))
            .map(|s| s.id)
            .collect();
        omafile::keys_for_sections(&ids)
    }

    /// The Sections row's words.
    pub fn sections_says(&self) -> String {
        let count = omafile::selectable_sections()
            .iter()
            .filter(|s| self.chosen(s.id))
            .count();
        format!(
            "{} settings across {count} sections.",
            self.chosen_keys().len()
        )
    }

    /// `forgetExport()`: what was written is only offered while it matches what would be.
    pub fn forget_export(&mut self) {
        self.written_path.clear();
        self.export_status.clear();
    }

    /// `forgetPlan()`: a plan belongs to one file.
    pub fn forget_plan(&mut self) {
        self.plan = None;
        self.import_status.clear();
    }

    /// `planHasChanges`.
    pub fn has_changes(&self) -> bool {
        self.plan
            .as_ref()
            .and_then(|p| p["changes"].as_array())
            .is_some_and(|c| !c.is_empty())
    }

    /// The Export button's file: none when no setting is chosen.
    pub fn export_text(&self, snap: &Value, now_iso: &str) -> Option<String> {
        let keys = self.chosen_keys();
        if keys.is_empty() {
            return None;
        }
        let meta = markdown::Meta {
            exported: now_iso.to_owned(),
            hardware: omafile::text_or_empty(snap.get("dmiProduct")),
        };
        Some(markdown::export(snap, &keys, &meta))
    }

    /// `ioAnswered(env)`: what an answer to a write, an open or a read leaves, with the error
    /// text (`FailureJs.errorText`) of one that failed.
    pub fn answered(
        &mut self,
        result: Result<&Value, &str>,
        path: &str,
        snap: &Value,
        opts: &plan::Options,
    ) {
        let Some(kind) = self.io.take() else {
            return;
        };
        let err = |e: &str, or: String| if e.is_empty() { or } else { e.to_owned() };
        match (kind, result) {
            ("write", Ok(_)) => {
                self.written_path = path.to_owned();
                self.export_status =
                    format!("Wrote {} settings to {path}", self.chosen_keys().len());
            }
            ("write", Err(e)) => self.export_status = err(e, format!("Could not write {path}")),
            ("open", Ok(_)) => {}
            ("open", Err(e)) => {
                self.export_status = err(e, "Nothing on this machine opens that file.".into())
            }
            (_, Err(e)) => self.import_status = err(e, format!("Could not read {path}")),
            (_, Ok(reply)) => {
                let raw = reply["files"][0]["text"].as_str().unwrap_or_default();
                let doc = markdown::parse(raw);
                let p = plan::plan(&doc, snap, opts);
                self.import_status = if p["changes"].as_array().is_none_or(Vec::is_empty) {
                    format!("Nothing to change. {}", crate::js::string(&p["summary"]))
                } else {
                    String::new()
                };
                self.last_doc = Some(doc);
                self.plan = Some(p);
            }
        }
    }

    /// `doApply()`: the apply job, which waits for sudo mode when a change needs root.
    pub fn apply(&mut self, seen: (u64, u64)) -> Option<Job> {
        let plan = self.plan.clone().filter(|_| self.has_changes())?;
        self.pending_apply = true;
        self.undoing = false;
        self.import_status.clear();
        self.step_total = plan["changes"].as_array().map_or(0, Vec::len);
        self.last_undo_needs_root = lines::password_count(&plan) > 0;
        self.seen = seen;
        let job = Job::new(
            "settings.apply",
            json!({ "plan": lines::plan_json(&plan) }),
            "settings-import",
            "The import",
        );
        Some(if self.last_undo_needs_root {
            job.sudo()
        } else {
            job
        })
    }

    /// `doUndo()`: the way back the last apply wrote.
    pub fn undo(&mut self, seen: (u64, u64)) -> Option<Job> {
        if self.last_backup_dir.is_empty() {
            return None;
        }
        self.pending_apply = true;
        self.undoing = true;
        self.import_status.clear();
        self.step_total = self.applied_count;
        self.seen = seen;
        let job = Job::new(
            "settings.undo",
            json!({ "plan": format!("{}/undo.json", self.last_backup_dir) }),
            "settings-undo",
            "Putting it back",
        );
        Some(if self.last_undo_needs_root {
            job.sudo()
        } else {
            job
        })
    }

    /// `onApplyFinished(exitCode, out, err)`.
    pub fn finished(&mut self, ok: bool, out: &str, err: &str) {
        self.pending_apply = false;
        let result = lines::apply_result(out);
        let applied = lines::applied_count(&result);
        let backup = lines::backup_dir(&result);
        let reason = omafile::trim(err).to_owned();
        let or = |words: &str| {
            if reason.is_empty() {
                words.to_owned()
            } else {
                reason.clone()
            }
        };
        if self.undoing {
            self.undoing = false;
            if ok {
                self.applied_count = 0;
                self.last_backup_dir.clear();
                self.import_status = "Put back.".into();
            } else {
                self.import_status = or("Could not put everything back.");
            }
            return;
        }
        self.plan = None;
        if !backup.is_empty() {
            self.last_backup_dir = backup;
        }
        self.applied_count = applied;
        self.import_status = if ok {
            format!("Applied {applied}. Try it, and put it back below if you do not like it.")
        } else if applied > 0 {
            format!(
                "{} Put it back restores what did.",
                or("Some changes did not apply.")
            )
        } else {
            or("Some changes did not apply.")
        };
    }

    /// The sudo prompt closed with the job that waited on it not run: nothing was applied.
    pub fn refused(&mut self) {
        self.pending_apply = false;
        self.undoing = false;
        self.import_status = "Nothing applied: these changes need sudo mode.".into();
    }
}

/// `onApplyStdout(line)` over the output so far: the step reached, of how many, on which key.
pub fn step(out: &str, total: usize) -> (usize, usize, String) {
    let mut at = (0, total, String::new());
    for line in out.split('\n') {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts[0] != "progress" {
            continue;
        }
        let n = |i: usize| {
            let v = crate::js::number(parts.get(i).map(|p| Value::from(*p)).as_ref());
            if v.is_finite() && v != 0.0 {
                Some(v as usize)
            } else {
                None
            }
        };
        at.0 = n(1).unwrap_or(0);
        at.1 = n(2).unwrap_or(at.1);
        at.2 = parts.get(3).copied().unwrap_or_default().to_owned();
    }
    at
}

/// The Applying row's words.
pub fn applying_says((now, total, key): &(usize, usize, String), prompt_open: bool) -> String {
    if !key.is_empty() {
        format!("{key}  ({now} of {total})")
    } else if prompt_open {
        "Waiting for sudo mode…".into()
    } else {
        "Working…".into()
    }
}

/// The Put it back row's words.
pub fn put_back_says(applied: usize) -> String {
    if applied > 0 {
        format!("Restores the {applied} values that import replaced. Try the machine first.")
    } else {
        "Restores the values the last import replaced.".into()
    }
}
