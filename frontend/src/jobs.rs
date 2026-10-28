//! Jobs, as `IoQueue.qml` and `Omarchy.qml` run them: a registry entry by name and arguments,
//! started with `job.start` and run by the daemon so it outlives the window (docs/adr/0009), one
//! at a time, its output and its end arriving as `job` events with the window's own `cid`.
//!
//! A job that needs root waits for sudo mode (`enqueueIo`): when the snapshot does not say it is
//! on, the window asks for the password, runs `sudo.enable` with it, reads `passwordlessSudo`
//! back, and only then runs the job (docs/adr/0019). A failed job says why in a toast, as
//! `Failure.applyBanner` words it; what a job changes the daemon invalidates and pushes when it
//! ends, so nothing is read here after it.

use std::collections::VecDeque;

use odk::prelude::*;
use serde_json::{Value, json};

use crate::client::Client;
use crate::protocol;
use crate::settings::{Held, Settings, now_ms};

/// The snapshot key that says sudo mode is on.
pub const SUDO: Held = Held {
    key: "passwordlessSudo",
    topic: "settings.host",
    unread: "Whether sudo mode is on could not be read",
};

/// How long sudo mode stays on once the prompt turns it on, unless a page says otherwise
/// (`Omarchy.sudoMinutes`).
pub const SUDO_MINUTES: u32 = 15;

#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    /// The registry entry (`backend/src/command/entries.rs`).
    pub name: &'static str,
    pub args: Value,
    /// What the pages call it while it runs (`jobKind`); the daemon lists it by this label.
    pub kind: &'static str,
    /// What a person calls it, for the toast that says it failed: "The firmware update".
    pub what: &'static str,
    /// It runs a privileged step, so it waits for sudo mode.
    pub sudo: bool,
}

impl Job {
    pub fn new(name: &'static str, args: Value, kind: &'static str, what: &'static str) -> Job {
        Job {
            name,
            args,
            kind,
            what,
            sudo: false,
        }
    }

    /// The same job, run once sudo mode is on.
    pub fn sudo(mut self) -> Job {
        self.sudo = true;
        self
    }
}

/// What a failed job says, from its exit summary, or from its output when it has none
/// (`Failure.applyBanner`, `Failure.commandText`). Empty when there is nothing to say: it
/// succeeded, or all it printed was known noise.
pub fn failure_text(summary: Option<&Value>, code: i64, err: &str, out: &str) -> String {
    if let Some(summary) = summary.filter(|s| s.is_object()) {
        if summary.get("ok") == Some(&Value::Bool(true)) {
            return String::new();
        }
        let message = summary
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();
        if !message.is_empty() {
            return message.to_owned();
        }
        let noise = summary
            .get("warnings")
            .and_then(Value::as_array)
            .is_some_and(|w| !w.is_empty());
        return if noise {
            String::new()
        } else {
            "Command failed".into()
        };
    }
    if code == 0 {
        return String::new();
    }
    let (e, o) = (err.trim(), out.trim());
    let text = match (e.is_empty(), o.is_empty()) {
        (false, false) if e != o => format!("{e}\n{o}"),
        (false, _) => e.to_owned(),
        _ => o.to_owned(),
    };
    if text.is_empty() {
        "Command failed".into()
    } else {
        text
    }
}

struct Running {
    cid: String,
    job: Job,
    /// The daemon's number for it, once it has said (`started`).
    id: Option<u64>,
    /// The next output line to take: one already seen is not taken again after an attach.
    next: u64,
    out: String,
    err: String,
}

/// The password prompt, while a job waits for sudo mode.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Prompt {
    pending: Option<Job>,
    pub password: String,
    /// Why the last try failed.
    pub error: String,
    /// `sudo.enable` is running with the password.
    pub enabling: bool,
}

pub struct Jobs {
    client: Client,
    settings: Entity<Settings>,
    queue: VecDeque<Job>,
    running: Option<Running>,
    prompt: Option<Prompt>,
    pub minutes: u32,
    /// This window's prefix for its `cid`s, so that no other window's job is taken for one of
    /// its own (`jobPrefix`).
    prefix: String,
    next: u64,
}

impl Jobs {
    pub fn new(client: Client, settings: Entity<Settings>) -> Jobs {
        Jobs {
            client,
            settings,
            queue: VecDeque::new(),
            running: None,
            prompt: None,
            minutes: SUDO_MINUTES,
            prefix: format!("w{:x}{:x}", std::process::id(), now_ms() % 0x10_0000),
            next: 0,
        }
    }

    /// Whether a job is running.
    pub fn busy(&self) -> bool {
        self.running.is_some()
    }

    /// The kind of the job that is running.
    pub fn running(&self) -> Option<&'static str> {
        self.running.as_ref().map(|r| r.job.kind)
    }

    pub fn prompt(&self) -> Option<&Prompt> {
        self.prompt.as_ref()
    }

    fn sudo_on(&self, cx: &App) -> bool {
        self.settings.read(cx).value(&SUDO) == Some(&Value::Bool(true))
    }

    /// Runs `job` after those before it; one that needs root first waits for sudo mode.
    pub fn run(&mut self, job: Job, window: &mut Window, cx: &mut Context<Self>) {
        if job.sudo && !self.sudo_on(cx) {
            self.prompt = Some(Prompt {
                pending: Some(job),
                ..Prompt::default()
            });
            cx.notify();
            return;
        }
        self.queue.push_back(job);
        self.kick(window, cx);
    }

    fn kick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.running.is_some() {
            return;
        }
        let Some(job) = self.queue.pop_front() else {
            return;
        };
        self.next += 1;
        let cid = format!("{}-{}", self.prefix, self.next);
        let reply = self
            .client
            .request(&protocol::job_start(&cid, job.name, &job.args, job.kind));
        self.running = Some(Running {
            cid: cid.clone(),
            job,
            id: None,
            next: 0,
            out: String::new(),
            err: String::new(),
        });
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            if let Err(failure) = reply.await {
                let _ = this.update_in(cx, |jobs, window, cx| {
                    if jobs.running.as_ref().is_some_and(|r| r.cid == cid) {
                        jobs.finish(1, None, Some(failure.message), window, cx);
                    }
                });
            }
        })
        .detach();
    }

    /// A `job` event from the daemon. One for another window's job, or for a job that is not
    /// running, is not this window's.
    pub fn event(&mut self, event: &Value, window: &mut Window, cx: &mut Context<Self>) {
        let Some(running) = self.running.as_mut() else {
            return;
        };
        let kind = event.get("kind").and_then(Value::as_str);
        let id = event.get("job").and_then(Value::as_u64);
        // The daemon forgot the job (it restarted and the job did not survive): it failed.
        if kind == Some("gone") {
            if running.id.is_some() && running.id == id {
                let gone = json!({"ok": false, "code": 1, "message": "The job is no longer running",
                    "warnings": []});
                self.finish(1, Some(gone), None, window, cx);
            }
            return;
        }
        if event.get("cid").and_then(Value::as_str) != Some(running.cid.as_str()) {
            return;
        }
        match kind {
            Some("started") => running.id = id.or(running.id),
            Some("stream") => {
                let seq = event
                    .get("seq")
                    .and_then(Value::as_u64)
                    .unwrap_or(running.next);
                if seq < running.next {
                    return;
                }
                running.next = seq + 1;
                let line = event
                    .get("line")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let buf = if event.get("fd").and_then(Value::as_str) == Some("err") {
                    &mut running.err
                } else {
                    &mut running.out
                };
                buf.push_str(line);
                buf.push('\n');
            }
            Some("exit") => {
                let code = event.get("code").and_then(Value::as_i64).unwrap_or(1);
                let summary = event.get("summary").cloned();
                self.finish(code, summary, None, window, cx);
            }
            _ => {}
        }
    }

    /// The server is back after being gone, or the daemon is: the job that runs is pointed at
    /// this window again, from the line after the last one seen, so its end is heard
    /// (`attachJobs`). One whose start was sent just before the link dropped has no number yet:
    /// the daemon's list says whether it took it.
    pub fn reattach(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(running) = self.running.as_ref() else {
            return;
        };
        if let Some(id) = running.id {
            drop(self.client.request(&protocol::job_attach(id, running.next)));
            return;
        }
        let cid = running.cid.clone();
        let list = self.client.request(&protocol::job_list());
        cx.spawn_in(window, async move |this, cx| {
            let rows = list.await.ok();
            let _ = this.update_in(cx, |jobs, window, cx| {
                let Some(running) = jobs.running.as_mut().filter(|r| r.cid == cid && r.id.is_none())
                else {
                    return;
                };
                let found = rows
                    .as_ref()
                    .and_then(|r| r.get("jobs"))
                    .and_then(Value::as_array)
                    .and_then(|rows| {
                        rows.iter()
                            .find(|row| row.get("cid").and_then(Value::as_str) == Some(cid.as_str()))
                    })
                    .and_then(|row| row.get("job").and_then(Value::as_u64));
                match found {
                    Some(id) => {
                        running.id = Some(id);
                        drop(jobs.client.request(&protocol::job_attach(id, 0)));
                    }
                    None => {
                        let lost = json!({"ok": false, "code": 1, "message": "The job did not start",
                            "warnings": []});
                        jobs.finish(1, Some(lost), None, window, cx);
                    }
                }
            });
        })
        .detach();
    }

    fn finish(
        &mut self,
        code: i64,
        summary: Option<Value>,
        refused: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(done) = self.running.take() else {
            return;
        };
        cx.notify();
        if done.job.name == "sudo.enable" {
            // A refusal to run it at all is not a wrong password, and says what it is.
            let why = match (code, refused) {
                (0, None) => None,
                (_, Some(refused)) => Some(refused),
                _ => Some("Wrong password.".to_owned()),
            };
            self.sudo_enabled(why, window, cx);
        } else {
            let text = refused
                .unwrap_or_else(|| failure_text(summary.as_ref(), code, &done.err, &done.out));
            if !text.is_empty() {
                toast(
                    window,
                    cx,
                    Toast::new(format!("{} failed", done.job.what))
                        .description(text)
                        .variant(Tone::Danger),
                );
            }
        }
        self.kick(window, cx);
    }

    /// The text in the password field.
    pub fn type_password(&mut self, text: &str, cx: &mut Context<Self>) {
        if let Some(prompt) = self.prompt.as_mut() {
            prompt.password = text.to_owned();
            cx.notify();
        }
    }

    /// Continue: sudo mode is turned on with the password, unless it already is
    /// (`confirmSudoMode`).
    pub fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let on = self.sudo_on(cx);
        let Some(prompt) = self.prompt.as_mut() else {
            return;
        };
        if prompt.enabling {
            return;
        }
        if prompt.password.is_empty() || prompt.password.contains('\n') {
            prompt.error = "Password cannot be empty.".into();
            cx.notify();
            return;
        }
        prompt.error.clear();
        if on {
            let pending = self.prompt.take().and_then(|p| p.pending);
            if let Some(job) = pending {
                self.queue.push_back(Job { sudo: false, ..job });
            }
            self.kick(window, cx);
            return;
        }
        prompt.enabling = true;
        let password = std::mem::take(&mut prompt.password);
        let job = Job::new(
            "sudo.enable",
            json!({"minutes": self.minutes, "password": password}),
            "passwordless-sudo",
            "Sudo mode",
        );
        // Before anything waiting: it is what they wait for.
        self.queue.push_front(job);
        self.kick(window, cx);
    }

    /// Cancel: the job that waited is not run.
    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        if self.prompt.as_ref().is_some_and(|p| !p.enabling) {
            self.prompt = None;
            cx.notify();
        }
    }

    /// `sudo.enable` ended. Sudo mode is on when the machine says so, not when the job exits:
    /// `passwordlessSudo` is read back, and then the job that waited runs.
    fn sudo_enabled(
        &mut self,
        failed: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(prompt) = self.prompt.as_mut() else {
            return;
        };
        prompt.enabling = false;
        if let Some(why) = failed {
            prompt.error = why;
            return;
        }
        let pending = self.prompt.take().and_then(|p| p.pending);
        let sent_at = now_ms();
        let read = self.client.request(&protocol::snapshot_keys(
            &[SUDO.key],
            &json!({SUDO.key: true}),
        ));
        let settings = self.settings.clone();
        cx.spawn_in(window, async move |this, cx| {
            let values = read.await.ok().and_then(|r| r.get("values").cloned());
            let _ = this.update_in(cx, |jobs, window, cx| {
                if let Some(values) = values {
                    settings.update(cx, |s, cx| {
                        s.adopt_reply(&json!({"snapshot": values}), sent_at);
                        cx.notify();
                    });
                }
                if let Some(job) = pending {
                    jobs.queue.push_back(Job { sudo: false, ..job });
                    jobs.kick(window, cx);
                }
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failure_says_the_summary_s_message_or_the_output() {
        let s = |v: Value| failure_text(Some(&v), 1, "", "");
        assert_eq!(s(json!({"ok": true, "message": "x"})), "");
        assert_eq!(s(json!({"ok": false, "message": " no space "})), "no space");
        assert_eq!(
            s(json!({"ok": false, "message": "", "warnings": ["noise"]})),
            ""
        );
        assert_eq!(s(json!({"ok": false, "message": ""})), "Command failed");
        assert_eq!(failure_text(None, 0, "err", "out"), "");
        assert_eq!(failure_text(None, 2, " err ", "out"), "err\nout");
        assert_eq!(failure_text(None, 2, "same", "same"), "same");
        assert_eq!(failure_text(None, 2, "", ""), "Command failed");
    }

    #[test]
    fn a_sudo_job_is_the_same_job_marked_to_wait() {
        let job = Job::new("update.firmware", json!({}), "update-firmware", "x").sudo();
        assert!(job.sudo);
        assert_eq!(job.name, "update.firmware");
    }
}
