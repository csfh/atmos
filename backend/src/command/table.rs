//! The table of named commands, launchers, jobs and interactive tools, and the
//! small interpreter that turns a name and its arguments into a command line.
//!
//! An entry is a name and a template. Each place in the template that takes
//! something from the window names the argument and what it may be, so a value
//! that could pass as an option, hold a line break, or climb out of a path is
//! refused before anything runs. The window sends a name and arguments; it never
//! builds a command line, and arguments never appear in a log or a job list.

use serde_json::Value;

use super::args::{bad, flag, text, uint, word};
use crate::error::Result;

/// What runs the entry, and so how the window follows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Short: runs to the end, answers with the keys it read back (`write`).
    Command,
    /// Starts something that stays open (an app, a window) as its own unit and
    /// does not wait for it (`write`).
    Launch,
    /// Streams output, may read input, may take a while (`job`).
    Job,
    /// Needs the user's session: a picker, a region tool, a recorder (`job`).
    Interactive,
}

/// What an argument may be.
#[derive(Clone, Copy)]
pub enum A {
    /// A plain value: no leading dash, no line break.
    Word,
    /// One line of free text, which may start with a dash.
    Text,
    /// An absolute path with no `..`.
    Path,
    /// A whole number between two bounds.
    Uint(u64, u64),
    Uuid,
    Addr,
    /// `[A-Za-z0-9._-]+`: a name, a unit, an interface.
    Ident,
    /// One of these.
    Is(&'static [&'static str]),
    /// An http(s) or git URL.
    Url,
    /// A name a person gave something: any text with no `/` and no leading dash.
    Label,
    /// One line of JSON, up to a quarter of a megabyte.
    Json,
}

/// One piece of a command line.
#[derive(Clone, Copy)]
pub enum T {
    /// Always this.
    L(&'static str),
    /// The ratmos binary and a task: `ratmos task NAME`.
    Task(&'static str),
    /// The argument, required.
    V(&'static str, A),
    /// The argument, when given.
    O(&'static str, A),
    /// The argument, or an empty argument when it is not given.
    Z(&'static str, A),
    /// This text when the boolean argument is true.
    F(&'static str, &'static str),
    /// One text when the boolean argument is true, another when false.
    B(&'static str, &'static str, &'static str),
    /// A prefix joined to the argument: `--webcam-size=` and a size.
    J(&'static str, &'static str, A),
    /// Each word of a list argument, between a minimum and a maximum count.
    Rest(&'static str, A, usize, usize),
}

/// What the job reads on stdin.
#[derive(Clone, Copy)]
pub enum In {
    No,
    /// The argument, as it is.
    Field(&'static str),
    /// The argument and a line break.
    Line(&'static str),
    /// The argument and a line break, when it is given.
    LineOpt(&'static str),
    /// Each argument on its own line.
    Lines(&'static [&'static str]),
}

pub struct Entry {
    pub name: &'static str,
    pub kind: Kind,
    /// The lane. Empty is a lane of its own for each run.
    pub lane: &'static str,
    pub argv: &'static [T],
    pub stdin: In,
    /// Run through `ratmos task gum-job`, which shows a job's progress.
    pub gum: bool,
    /// Needs root: the window asks for passwordless sudo first.
    pub root: bool,
    /// Display kinds and settings topics the command or interactive tool
    /// changes: they are dropped when it ends, so the next read of them
    /// collects again. A `Job` changes every settings topic without saying.
    pub affects: &'static [&'static str],
    /// Snapshot keys read back once it has run, which is what the window shows.
    pub readback: &'static [&'static str],
}

impl Entry {
    pub const fn new(name: &'static str, kind: Kind, argv: &'static [T]) -> Entry {
        Entry {
            name,
            kind,
            lane: "",
            argv,
            stdin: In::No,
            gum: false,
            root: false,
            affects: &[],
            readback: &[],
        }
    }

    pub const fn readback(mut self, keys: &'static [&'static str]) -> Entry {
        self.readback = keys;
        self
    }

    pub const fn affects(mut self, kinds: &'static [&'static str]) -> Entry {
        self.affects = kinds;
        self
    }

    pub const fn lane(mut self, lane: &'static str) -> Entry {
        self.lane = lane;
        self
    }

    pub const fn stdin(mut self, stdin: In) -> Entry {
        self.stdin = stdin;
        self
    }

    pub const fn gum(mut self) -> Entry {
        self.gum = true;
        self
    }

    pub const fn root(mut self) -> Entry {
        self.root = true;
        self
    }
}

fn plain(args: &Value, field: &str, a: A) -> Result<String> {
    let value = args.get(field);
    let string = |v: Option<&Value>| v.and_then(Value::as_str).map(str::to_string);
    Ok(match a {
        A::Word => word(args, field)?.to_string(),
        A::Text => text(args, field)?.to_string(),
        A::Path => {
            let p = text(args, field)?;
            if !p.starts_with('/') || p.split('/').any(|part| part == "..") {
                return Err(bad(field, "an absolute path is required"));
            }
            p.to_string()
        }
        A::Uint(min, max) => uint(args, field, min, max)?.to_string(),
        A::Uuid => {
            let u = word(args, field)?;
            if u.len() != 36 || !u.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
                return Err(bad(field, "not a connection id"));
            }
            u.to_string()
        }
        A::Addr => {
            let a = word(args, field)?;
            let octets: Vec<&str> = a.split(':').collect();
            if octets.len() != 6
                || !octets
                    .iter()
                    .all(|o| o.len() == 2 && o.chars().all(|c| c.is_ascii_hexdigit()))
            {
                return Err(bad(field, "not a device address"));
            }
            a.to_string()
        }
        A::Ident => {
            let w = word(args, field)?;
            if !w
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-@:\\".contains(c))
            {
                return Err(bad(field, "unexpected characters"));
            }
            w.to_string()
        }
        A::Is(allowed) => {
            let w = string(value).ok_or_else(|| bad(field, "a value is required"))?;
            if !allowed.contains(&w.as_str()) {
                return Err(bad(field, &format!("one of {}", allowed.join(", "))));
            }
            w
        }
        A::Label => {
            let l = text(args, field)?;
            if l.starts_with('-') || l.contains('/') {
                return Err(bad(field, "not a name"));
            }
            l.to_string()
        }
        A::Json => {
            let j = string(value).ok_or_else(|| bad(field, "text is required"))?;
            if j.is_empty() || j.len() > 256 * 1024 || j.contains(['\n', '\r', '\0']) {
                return Err(bad(field, "one line of JSON"));
            }
            j
        }
        A::Url => {
            let u = word(args, field)?;
            let ok = ["https://", "http://", "git@", "ssh://", "git://"]
                .iter()
                .any(|scheme| u.starts_with(scheme));
            if !ok || u.contains(' ') {
                return Err(bad(field, "not a URL"));
            }
            u.to_string()
        }
    })
}

fn given(args: &Value, field: &str) -> bool {
    match args.get(field) {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

/// The command line the entry stands for, with its arguments checked.
pub fn argv(entry: &Entry, args: &Value) -> Result<Vec<String>> {
    let exe = || {
        std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "ratmos".into())
    };
    let mut out: Vec<String> = Vec::new();
    for token in entry.argv {
        match *token {
            T::L(lit) => out.push(lit.into()),
            T::Task(name) => out.extend([exe(), "task".into(), name.into()]),
            T::V(field, a) => out.push(plain(args, field, a)?),
            T::O(field, a) => {
                if given(args, field) {
                    out.push(plain(args, field, a)?);
                }
            }
            T::Z(field, a) => {
                out.push(if given(args, field) {
                    plain(args, field, a)?
                } else {
                    String::new()
                });
            }
            T::F(field, lit) => {
                if args.get(field).and_then(Value::as_bool) == Some(true) {
                    out.push(lit.into());
                }
            }
            T::B(field, yes, no) => out.push(if flag(args, field)? { yes } else { no }.into()),
            T::J(prefix, field, a) => {
                if given(args, field) {
                    out.push(format!("{prefix}{}", plain(args, field, a)?));
                }
            }
            T::Rest(field, a, min, max) => {
                let list = args
                    .get(field)
                    .and_then(Value::as_array)
                    .ok_or_else(|| bad(field, "a list is required"))?;
                if list.len() < min || list.len() > max {
                    return Err(bad(field, &format!("between {min} and {max} values")));
                }
                for item in list {
                    let one = serde_json::json!({ "v": item });
                    out.push(plain(&one, "v", a)?);
                }
            }
        }
    }
    if entry.gum {
        let mut wrapped = vec![exe(), "task".into(), "gum-job".into()];
        wrapped.append(&mut out);
        out = wrapped;
    }
    Ok(out)
}

/// What the entry feeds the command on stdin.
pub fn stdin(entry: &Entry, args: &Value) -> Result<Option<String>> {
    let field = |name: &str| -> Result<String> {
        args.get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| bad(name, "text is required"))
    };
    let checked = |name: &str, s: String| -> Result<String> {
        if s.contains('\0') || s.len() > 4 * 1024 * 1024 {
            return Err(bad(name, "not text this size"));
        }
        Ok(s)
    };
    Ok(match entry.stdin {
        In::No => None,
        In::Field(name) => Some(checked(name, field(name)?)?),
        In::LineOpt(name) => {
            if args
                .get(name)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Ok(None);
            }
            let s = field(name)?;
            if s.contains(['\n', '\r']) {
                return Err(bad(name, "one line"));
            }
            Some(format!("{}\n", checked(name, s)?))
        }
        In::Line(name) => {
            let s = field(name)?;
            if s.contains(['\n', '\r']) {
                return Err(bad(name, "one line"));
            }
            Some(format!("{}\n", checked(name, s)?))
        }
        In::Lines(names) => {
            let mut all = String::new();
            for name in names {
                let s = field(name)?;
                if s.contains(['\n', '\r']) {
                    return Err(bad(name, "one line"));
                }
                all.push_str(&checked(name, s)?);
                all.push('\n');
            }
            Some(all)
        }
    })
}
