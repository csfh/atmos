//! Sentinel: processes that stay busy, hold a lot of memory, or keep a port open for days
//! (`pages/SentinelPage.qml`). The page holds the live feed while it is shown (the shell does,
//! `app.rs`), takes in each frame it is handed, and lists what `sentinel.rs` finds in it.
//!
//! What the QML page did with sliders is a number field here, written when it commits and not as
//! it is typed (ODK's `NumberInput::on_commit`), and what it did with a confirm component is a
//! `Dialog`. A row does not unfold when it is clicked as the QML one does: "Show everything" in its
//! menu does, so the whole of a long command line can be read.

use std::collections::HashSet;

use odk::prelude::*;
use serde_json::{Value, json};

use crate::actions::{self, Signal};
use crate::client::Client;
use crate::live::Feed;
use crate::protocol;
use crate::sentinel::{self, Counts, Filter, Finding, Limit, Thresholds, Tracked, clamp_threshold};

/// A limit as the daemon last said it, or that it has not said.
#[derive(Debug, Clone, PartialEq)]
enum Stored {
    Loading,
    Ready(i64),
    /// The read failed. A failure is never shown as a value (docs/adr/0006), so the field is
    /// not drawn and the reason is.
    Failed(String),
}

fn slot(limit: Limit) -> usize {
    match limit {
        Limit::CpuPercent => 0,
        Limit::MemoryPercent => 1,
        Limit::PortAgeDays => 2,
    }
}

/// What the confirm dialog is asking about.
#[derive(Debug, Clone, PartialEq)]
struct Pending {
    pid: u64,
    comm: String,
    signal: Signal,
}

pub struct SentinelPage {
    client: Client,
    feed: Feed,
    tracked: Tracked,
    /// Every finding of the newest sample, whatever the chip and the search say.
    all: Vec<Finding>,
    filter: Filter,
    query: SharedString,
    limits: [Stored; 3],
    pending: Option<Pending>,
    /// Rows whose whole text is shown.
    expanded: HashSet<String>,
    /// How many frames were taken in.
    taken: usize,
}

impl SentinelPage {
    pub fn new(client: Client, cx: &mut Context<Self>) -> Self {
        let page = Self {
            client,
            feed: Feed::default(),
            tracked: Tracked::default(),
            all: Vec::new(),
            filter: Filter::default(),
            query: SharedString::default(),
            limits: [Stored::Loading, Stored::Loading, Stored::Loading],
            pending: None,
            expanded: HashSet::new(),
            taken: 0,
        };
        for limit in Limit::ALL {
            page.load_limit(limit, cx);
        }
        page
    }

    /// The limits in force: what the daemon stores, and the default for what it has not said.
    pub fn thresholds(&self) -> Thresholds {
        let get = |l: Limit| match self.limits[slot(l)] {
            Stored::Ready(v) => v,
            _ => l.default(),
        };
        Thresholds {
            cpu_percent: get(Limit::CpuPercent),
            memory_percent: get(Limit::MemoryPercent),
            port_age_days: get(Limit::PortAgeDays),
        }
    }

    /// How many frames the page has taken in; one that is not after the last, or is no sample,
    /// is not counted.
    pub fn taken(&self) -> usize {
        self.taken
    }

    /// Whether the confirm dialog is open.
    pub fn asking(&self) -> bool {
        self.pending.is_some()
    }

    /// The chip that is chosen.
    pub fn filter(&self) -> Filter {
        self.filter
    }

    /// The rows that are shown now, in order.
    pub fn shown(&self) -> Vec<Finding> {
        sentinel::list(&self.all, self.filter, &self.query)
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.all = sentinel::rows(self.feed.latest(), &self.tracked, &self.thresholds());
        cx.notify();
    }

    /// A `live` frame from the daemon: when it began collecting, and what it collected.
    pub fn adopt(&mut self, sampled_at: f64, result: &Value, cx: &mut Context<Self>) {
        if !self.feed.adopt(sampled_at, result) {
            return;
        }
        self.taken += 1;
        self.tracked = self.tracked.track(self.feed.latest(), &self.thresholds());
        self.refresh(cx);
    }

    /// The backend is there again after it was gone: what could not be read is read again.
    pub fn backend_up(&mut self, cx: &mut Context<Self>) {
        for limit in Limit::ALL {
            if !matches!(self.limits[slot(limit)], Stored::Ready(_)) {
                self.load_limit(limit, cx);
            }
        }
    }

    fn load_limit(&self, limit: Limit, cx: &mut Context<Self>) {
        let reply = self.client.request(&protocol::settings_get(limit.domain()));
        cx.spawn(async move |this, cx| {
            let outcome = reply.await;
            let _ = this.update(cx, |page, cx| {
                page.limits[slot(limit)] = match outcome {
                    // Unset reads as `null`, and is the default.
                    Ok(value) => Stored::Ready(clamp_threshold(limit, Some(&value))),
                    Err(failure) => Stored::Failed(failure.message),
                };
                page.refresh(cx);
            });
        })
        .detach();
    }

    /// A limit the field committed: kept in range, shown at once, and stored. When the store
    /// refuses it, what is stored is read again and the person is told.
    fn save_limit(
        &mut self,
        limit: Limit,
        number: f64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = clamp_threshold(limit, Some(&json!(number)));
        if self.limits[slot(limit)] == Stored::Ready(value) {
            return;
        }
        self.limits[slot(limit)] = Stored::Ready(value);
        self.refresh(cx);
        // An integer domain takes an integer: `80`, not `80.0`.
        let reply =
            self.client
                .request(&protocol::settings_set(limit.domain(), &json!(value), &[]));
        cx.spawn_in(window, async move |this, cx| {
            if let Err(failure) = reply.await {
                let _ = this.update_in(cx, |page, window, cx| {
                    toast(
                        window,
                        cx,
                        Toast::new("The limit was not saved")
                            .description(failure.message)
                            .variant(Tone::Danger),
                    );
                    page.load_limit(limit, cx);
                });
            }
        })
        .detach();
    }

    fn set_query(&mut self, query: &str, cx: &mut Context<Self>) {
        self.query = query.to_owned().into();
        cx.notify();
    }

    fn set_filter(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(filter) = Filter::from_id(id) {
            self.filter = filter;
            cx.notify();
        }
    }

    fn toggle(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.expanded.remove(id) {
            self.expanded.insert(id.to_owned());
        }
        cx.notify();
    }

    /// Asks before sending `signal` to the process, naming it.
    pub fn ask(&mut self, pid: u64, comm: &str, signal: Signal, cx: &mut Context<Self>) {
        self.pending = Some(Pending {
            pid,
            comm: comm.to_owned(),
            signal,
        });
        cx.notify();
    }

    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        self.pending = None;
        cx.notify();
    }

    /// The person said yes: the signal is sent, and a refusal is told.
    pub fn confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        cx.notify();
        let Some(body) = actions::signal_write(pending.pid, pending.signal) else {
            return;
        };
        let reply = self.client.request(&body);
        cx.spawn_in(window, async move |this, cx| {
            if let Err(failure) = reply.await {
                let _ = this.update_in(cx, |_, window, cx| {
                    toast(
                        window,
                        cx,
                        Toast::new(format!(
                            "{} was not sent to {}",
                            pending.signal.name(),
                            pending.comm
                        ))
                        .description(failure.message)
                        .variant(Tone::Danger),
                    );
                });
            }
        })
        .detach();
    }

    fn copy(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(body) = actions::copy_write(text) else {
            return;
        };
        let reply = self.client.request(&body);
        cx.spawn_in(window, async move |this, cx| {
            if let Err(failure) = reply.await {
                let _ = this.update_in(cx, |_, window, cx| {
                    toast(
                        window,
                        cx,
                        Toast::new("Nothing was copied")
                            .description(failure.message)
                            .variant(Tone::Danger),
                    );
                });
            }
        })
        .detach();
    }
}

const DESCRIPTION: &str = "Processes that stay busy, hold a lot of memory, or keep a port open for days. Linux does not record when a socket opened, so a port's age is its process's age. Ports owned by root services show without a process.";

impl Render for SentinelPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let counts = Counts::of(&self.all);
        let summary = if self.feed.latest().is_none() {
            "Reading the first sample…".to_owned()
        } else {
            format!(
                "{} flagged · {} listening ports",
                counts.flagged, counts.port
            )
        };
        let shown = self.shown();

        let mut kinds = ToggleGroup::new("sentinel-kinds", "Show")
            .value([self.filter.id()])
            .on_change({
                let this = this.clone();
                move |values, _, cx| {
                    if let Some(id) = values.first() {
                        this.update(cx, |page, cx| page.set_filter(id, cx));
                    }
                }
            });
        for filter in Filter::ALL {
            kinds = kinds.item(
                filter.id(),
                format!("{} {}", filter.label(), counts.of_filter(filter)),
            );
        }

        let rows = v_flex().children(shown.iter().map(|finding| self.row(finding, &this)));

        content_column()
            .child(Heading::new("Sentinel").level(1))
            .child(Text::new(DESCRIPTION).muted())
            .child(Text::new(summary).muted())
            .child(v_flex().w_1_2().child(
                SearchInput::new("sentinel-search", "Search processes and ports")
                    .placeholder("Search processes and ports…")
                    .value(self.query.clone())
                    .on_change({
                        let this = this.clone();
                        move |text, _, cx| this.update(cx, |page, cx| page.set_query(text, cx))
                    }),
            ))
            .child(kinds)
            .child(if shown.is_empty() {
                self.empty().into_any_element()
            } else {
                rows.into_any_element()
            })
            .child(Heading::new("Limits").level(2))
            .child(
                Text::new("What counts as rogue. A process has to stay over the CPU limit for a few samples in a row before it is flagged.")
                    .muted(),
            )
            .children(self.limit_alert())
            .child(form().w_1_2().children(Limit::ALL.map(|limit| self.limit_row(limit, &this))))
            .child(self.confirm_dialog(&this))
    }
}

impl SentinelPage {
    fn empty(&self) -> impl IntoElement {
        let waiting = self.feed.latest().is_none();
        let (title, why) = if waiting {
            (
                "Reading the first sample".to_owned(),
                "CPU findings need a few samples, about every two seconds.",
            )
        } else {
            (
                if self.filter == Filter::Port {
                    "No listening ports".to_owned()
                } else {
                    "Nothing looks wrong".to_owned()
                },
                if self.query.is_empty() {
                    "No process is over its limits, and no long-running process holds a port open."
                } else {
                    "Nothing matches that search."
                },
            )
        };
        v_flex()
            .child(Heading::new(title).level(3))
            .child(Text::new(why).muted())
    }

    /// One finding: the name, what is wrong, the command line, and what can be done to it.
    fn row(&self, f: &Finding, this: &Entity<Self>) -> impl IntoElement {
        let open = self.expanded.contains(&f.id);
        let mut title = h_flex()
            .gap(Space::Sm)
            .child(Heading::new(f.comm.clone()).level(3));
        if f.flagged {
            title = title.child(Badge::new("Flagged").tone(Tone::Danger).marked());
        }
        let detail = Text::new(f.detail.clone()).muted();
        let detail = if open { detail } else { detail.single_line() };
        let command = (!f.cmdline.is_empty()).then(|| {
            let t = Text::new(f.cmdline.clone()).mono().muted();
            if open { t } else { t.single_line() }
        });

        let mut text = v_flex()
            .flex_1()
            .min_w_0()
            .gap(Space::Xs)
            .child(title)
            .child(detail);
        if let Some(command) = command {
            text = text.child(command);
        }
        h_flex().gap(Space::Md).py(Space::Sm).child(text).children(
            f.proc
                .as_ref()
                .filter(|p| actions::can_signal(p))
                .map(|p| self.actions(f, p.pid, &p.comm, &p.cmdline, this)),
        )
    }

    /// End, and the rest in a menu.
    fn actions(
        &self,
        f: &Finding,
        pid: u64,
        comm: &str,
        cmdline: &str,
        this: &Entity<Self>,
    ) -> impl IntoElement {
        let id = f.id.clone();
        let end = Button::new(SharedString::from(format!("end-{id}")))
            .label("End")
            .on_click({
                let (this, comm) = (this.clone(), comm.to_owned());
                move |_, _, cx| {
                    this.update(cx, |page, cx| page.ask(pid, &comm, Signal::Term, cx));
                }
            });
        let copy_command = if cmdline.is_empty() { comm } else { cmdline }.to_owned();
        let open = self.expanded.contains(&f.id);
        let more = DropdownMenu::new(SharedString::from(format!("more-{id}")), "More").menu({
            let (this, comm) = (this.clone(), comm.to_owned());
            move |menu| {
                let (ask, copy_pid, copy_cmd) = (this.clone(), this.clone(), this.clone());
                let (unfold, row) = (this.clone(), id.clone());
                menu.item(
                    // A row shows one line of each until this says otherwise, so that a long
                    // command line can be read whole.
                    MenuItem::new(if open { "Show less" } else { "Show everything" }).on_select(
                        move |_, cx| unfold.update(cx, |page, cx| page.toggle(&row, cx)),
                    ),
                )
                .separator()
                .item(
                    MenuItem::new("Force quit")
                        .danger()
                        .on_select(move |_, cx| {
                            ask.update(cx, |page, cx| page.ask(pid, &comm, Signal::Kill, cx));
                        }),
                )
                .item(MenuItem::new("Copy pid").on_select(move |window, cx| {
                    copy_pid.update(cx, |page, cx| page.copy(&pid.to_string(), window, cx));
                }))
                .item(MenuItem::new("Copy command").on_select(move |window, cx| {
                    copy_cmd.update(cx, |page, cx| page.copy(&copy_command, window, cx));
                }))
            }
        });
        h_flex().gap(Space::Sm).child(end).child(more)
    }

    fn confirm_dialog(&self, this: &Entity<Self>) -> impl IntoElement {
        let (title, message, go) = match &self.pending {
            Some(p) if p.signal == Signal::Kill => (
                "Force quit this process".to_owned(),
                format!(
                    "Send SIGKILL to {} ({}). It will not get a chance to exit cleanly.",
                    p.comm, p.pid
                ),
                "Force quit",
            ),
            Some(p) => (
                "End this process".to_owned(),
                format!("Send SIGTERM to {} ({}).", p.comm, p.pid),
                "End",
            ),
            None => (String::new(), String::new(), "End"),
        };
        Dialog::new("sentinel-confirm", title)
            .open(self.pending.is_some())
            .description(message)
            .close_on_backdrop(false)
            .on_close({
                let this = this.clone();
                move |_, cx| this.update(cx, |page, cx| page.cancel(cx))
            })
            .action({
                let this = this.clone();
                move |_, _| {
                    let this = this.clone();
                    Button::new("sentinel-cancel")
                        .label("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| this.update(cx, |page, cx| page.cancel(cx)))
                }
            })
            .action({
                let this = this.clone();
                move |_, _| {
                    let this = this.clone();
                    Button::new("sentinel-confirm-go")
                        .label(go)
                        .variant(ButtonVariant::Destructive)
                        .on_click(move |_, window, cx| {
                            this.update(cx, |page, cx| page.confirm(window, cx))
                        })
                }
            })
    }

    /// Why the limits cannot be shown, when the daemon could not say.
    fn limit_alert(&self) -> Option<impl IntoElement> {
        self.limits.iter().find_map(|s| match s {
            Stored::Failed(why) => Some(
                Alert::new("sentinel-limits-alert", "The limits could not be read")
                    .variant(Tone::Danger)
                    .description(why.clone()),
            ),
            _ => None,
        })
    }

    fn limit_row(&self, limit: Limit, this: &Entity<Self>) -> impl IntoElement {
        let (label, what, unit, hint) = match limit {
            Limit::CpuPercent => (
                "CPU",
                "Flag a process using more than this share of one core.",
                "%",
                "shell.json atmos.sentinel.cpuPercent",
            ),
            Limit::MemoryPercent => (
                "Memory",
                "Flag a process holding more than this share of installed memory.",
                "%",
                "shell.json atmos.sentinel.memoryPercent",
            ),
            Limit::PortAgeDays => (
                "Port age",
                "Flag a listening port whose process has been up longer than this.",
                "days",
                "shell.json atmos.sentinel.portAgeDays",
            ),
        };
        let (low, high) = limit.range();
        let state = &self.limits[slot(limit)];
        match state {
            Stored::Ready(value) => {
                let this = this.clone();
                FormField::new(
                    SharedString::from(format!("limit-field-{}", limit.domain())),
                    format!("{label} ({unit})"),
                )
                .description(what)
                .help(hint)
                .child(
                    NumberInput::new(
                        SharedString::from(format!("limit-{}", limit.domain())),
                        format!("{label} limit"),
                    )
                    .min(low as f64)
                    .max(high as f64)
                    .step(limit.step() as f64)
                    .value(*value as f64)
                    .on_commit(move |number, window, cx| {
                        this.update(cx, |page, cx| page.save_limit(limit, number, window, cx))
                    }),
                )
                .into_any_element()
            }
            // Not a field with a number made up for it: the limit is being read, or could not be.
            Stored::Loading => v_flex()
                .gap(Space::Xs)
                .child(Text::new(label))
                .child(Skeleton::line().label(format!("Reading the {label} limit")))
                .into_any_element(),
            Stored::Failed(_) => v_flex()
                .gap(Space::Xs)
                .child(Text::new(label))
                .child(Text::new("Unknown").muted())
                .into_any_element(),
        }
    }
}
