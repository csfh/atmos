//! A page drawn from the catalog (docs/adr/0021): every hub and hub page that has not been ported
//! is drawn from what its QML page has, its groups and their rows, with the hub's pages as tabs.
//! A row whose control is mapped draws it (`catalog/control.rs`); a row that is not says so in
//! its place and keeps its star. One page entity draws whichever catalog page is shown.
//!
//! A control is controlled by the snapshot: it shows a change when the backend reports it, not
//! when it is made (docs/adr/0005). A control whose key has never been read is not drawn, and
//! the page says why (docs/adr/0006). A slider of the QML page is a number field here, as on
//! Sentinel.

use std::collections::HashMap;
use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use crate::catalog::control::{Act, Reading, Shape, on};
use crate::catalog::setter::{Btn, Sends, Setter};
use crate::catalog::{self, Control, Field, Group, Line, Page};
use crate::client::Client;
use crate::favorites;
use crate::feed::Live;
use crate::hubs;
use crate::inventory::Inventories;
use crate::jobs::{Job, Jobs};
use crate::js;
use crate::pages::controls::{confirm, number, prompt, select, text};
use crate::pages::favorites::Open;
use crate::pages::row::{self, Stars};
use crate::pages::{agents, appearance, custom, environment, rules};
use crate::profiles::{self, Profile};
use crate::settings::{Held, Known, Settings};

/// What a confirm's yes runs.
pub type Then = Rc<dyn Fn(&mut CatalogPage, &mut Window, &mut Context<CatalogPage>)>;

/// A confirm whose words a row built (`"Remove " + command + "…"`), and what yes runs.
pub struct Asked {
    pub title: String,
    pub message: String,
    pub action: &'static str,
    pub go: Then,
}

pub struct CatalogPage {
    pub(super) stars: Stars,
    /// The window's one queue of jobs, which a button that starts one hands it to.
    pub(super) jobs: Entity<Jobs>,
    /// The window's inventories, which a custom row reads.
    pub(super) inventories: Entity<Inventories>,
    /// The live samples the Monitor and Home pages read.
    pub(super) live: Entity<Live>,
    page: &'static Page,
    pub(super) open: Open,
    /// What has been typed into a text row and not yet written, by the row's id.
    pub(super) drafts: HashMap<String, String>,
    /// What a page read for itself, by name (the agents).
    pub(super) data: HashMap<&'static str, Value>,
    /// What the Export page holds between its rows.
    pub(super) omafile: crate::export::State,
    /// The images `atmosd` sent (the face, the wallpaper), kept so a redraw does not decode
    /// them again.
    pub(super) images: HashMap<&'static str, std::sync::Arc<[u8]>>,
    /// The button whose confirm is open, and its row's label.
    asking: Option<(Btn, &'static str)>,
    /// A confirm whose words a row built, and what it runs on yes.
    pub(super) asked: Option<Asked>,
    /// The button whose prompt is open, its row's label, and what has been typed into it.
    prompting: Option<(Btn, &'static str, String)>,
}

const PENDING: &str = "Not in the new window yet";
const PENDING_DETAIL: &str =
    "Rows marked Not yet are changed in the current Atmos window for now. The rest work here.";
const LISTED: &str = "What this group lists is not in the new window yet.";

impl CatalogPage {
    /// The page on `page`, drawn again whenever the window's settings change.
    pub fn new(
        client: Client,
        settings: Entity<Settings>,
        jobs: Entity<Jobs>,
        (inventories, live): (Entity<Inventories>, Entity<Live>),
        page: &'static Page,
        open: Open,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&settings, |page, _, cx| {
            agents::settings_changed(page, cx);
            environment::settings_changed(page, cx);
            crate::pages::radios::want(page, cx);
            cx.notify()
        })
        .detach();
        cx.observe(&inventories, |_, _, cx| cx.notify()).detach();
        cx.observe(&live, |_, _, cx| cx.notify()).detach();
        // A button waits for the window's job to end (`When::Idle`).
        cx.observe(&jobs, |page, jobs, cx| {
            crate::pages::wifi::took(page, jobs.read(cx));
            crate::pages::export::took(page, jobs.read(cx));
            let ended = {
                let j = jobs.read(cx);
                (j.ended, j.last)
            };
            crate::pages::radios::took(page, ended, cx);
            cx.notify()
        })
        .detach();
        Self {
            stars: Stars { client, settings },
            jobs,
            inventories,
            live,
            page,
            open,
            drafts: HashMap::new(),
            data: HashMap::new(),
            omafile: crate::pages::export::state(),
            images: HashMap::new(),
            asking: None,
            asked: None,
            prompting: None,
        }
    }

    pub fn show(&mut self, page: &'static Page, cx: &mut Context<Self>) {
        self.page = page;
        self.loaded(cx);
        cx.notify();
    }

    /// Reads what the page shown reads for itself.
    pub fn loaded(&mut self, cx: &mut Context<Self>) {
        agents::load(self, None, cx);
        crate::pages::speedtest::loaded(self, cx);
        crate::pages::radios::loaded(self, cx);
    }

    pub fn page(&self) -> &'static Page {
        self.page
    }

    /// What a field reads now, as its row shows it.
    pub fn reading(&self, field: Field, cx: &App) -> Reading {
        field.read(self.stars.settings.read(cx))
    }

    /// A change of a field to `value`: written when it changes what the backend last reported.
    pub fn set(
        &mut self,
        label: &str,
        field: Field,
        value: Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let body = field.write(value, self.stars.settings.read(cx));
        self.send(label, field, body, window, cx);
    }

    /// A change that calls a setter of its own with `value`.
    pub fn call(
        &mut self,
        label: &str,
        field: Option<Field>,
        setter: Setter,
        value: Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(body) = setter.body(field, value, self.stars.settings.read(cx)) else {
            return;
        };
        if let Sends::SudoMode = setter.sends {
            self.jobs.update(cx, |jobs, cx| jobs.ask_sudo(cx));
            return;
        }
        if let Sends::Job(t) = setter.sends {
            let job = Job::new(t.name, body["args"].clone(), t.kind, t.what);
            let job = if t.sudo { job.sudo() } else { job };
            let job = if t.interactive {
                job.interactive()
            } else {
                job
            };
            self.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
            return;
        }
        let client = self.stars.client.clone();
        let refused = format!("{label} was not changed");
        let key = field.map_or(setter.name, Field::key);
        self.stars.settings.update(cx, |settings, cx| {
            settings.send(&client, key, &body, refused, window, cx);
        });
    }

    /// A button of a row of buttons: its confirm first, if it has one.
    pub fn press(
        &mut self,
        label: &'static str,
        b: Btn,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if b.prompt.is_some() {
            self.prompting = Some((b, label, String::new()));
            cx.notify();
            return;
        }
        if b.confirm.is_some() {
            self.asking = Some((b, label));
            cx.notify();
            return;
        }
        self.call(label, None, b.setter, Value::Null, window, cx);
    }

    /// The open confirm was answered.
    pub fn answer(&mut self, go: bool, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((b, label)) = self.asking.take()
            && go
        {
            self.call(label, None, b.setter, Value::Null, window, cx);
        }
        if let Some(asked) = self.asked.take()
            && go
        {
            (asked.go)(self, window, cx);
        }
        cx.notify();
    }

    /// Asks a confirm whose words were built for it, and runs `go` on yes.
    pub fn ask(&mut self, asked: Asked, cx: &mut Context<Self>) {
        self.asked = Some(asked);
        cx.notify();
    }

    /// The open confirm's title, whichever kind it is, for what a test asks.
    pub fn asked(&self) -> Option<String> {
        self.asked.as_ref().map(|a| a.title.clone())
    }

    /// What has been typed into the open prompt.
    pub fn type_prompt(&mut self, text: &str, cx: &mut Context<Self>) {
        if let Some((_, _, draft)) = self.prompting.as_mut() {
            *draft = text.to_owned();
            cx.notify();
        }
    }

    /// The open prompt was answered: with what its check keeps of the text, or closed.
    pub fn answer_prompt(&mut self, go: bool, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((b, label, draft)) = self.prompting.take()
            && go
            && let Some(p) = b.prompt
        {
            let parsed = p.parse.parse(&draft);
            if parsed.is_empty() {
                self.prompting = Some((b, label, draft));
                return;
            }
            self.call(label, None, b.setter, Value::from(parsed), window, cx);
        }
        cx.notify();
    }

    /// The open prompt's title, for what a test asks.
    pub fn prompting(&self) -> Option<&'static str> {
        self.prompting
            .as_ref()
            .and_then(|(b, _, _)| b.prompt)
            .map(|p| p.title)
    }

    /// The open confirm's title, for what a test asks.
    pub fn asking(&self) -> Option<&'static str> {
        self.asking.and_then(|(b, _)| b.confirm).map(|c| c.title)
    }

    /// A text row's value, written as it stands, as the QML row's Set writes it.
    pub fn put(
        &mut self,
        label: &str,
        field: Field,
        value: Value,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let body = field.send(value, self.stars.settings.read(cx));
        self.send(label, field, body, window, cx);
    }

    fn send(
        &mut self,
        label: &str,
        field: Field,
        body: Option<Value>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(body) = body else {
            return;
        };
        let client = self.stars.client.clone();
        let refused = format!("{label} was not changed");
        self.stars.settings.update(cx, |settings, cx| {
            settings.send(&client, field.key(), &body, refused, window, cx);
        });
    }

    /// A typed write's button.
    pub fn act(&mut self, act: Act, window: &mut Window, cx: &mut Context<Self>) {
        let client = self.stars.client.clone();
        let refused = format!("{} did not run", act.label);
        self.stars.settings.update(cx, |settings, cx| {
            settings.send(&client, act.name, &act.body(), refused, window, cx);
        });
    }

    fn line(
        &self,
        line: &'static Line,
        this: &Entity<Self>,
        cx: &App,
    ) -> Option<impl IntoElement + use<>> {
        let settings = self.stars.settings.read(cx);
        if !line
            .available
            .holds_with(settings, self.jobs.read(cx).job_busy())
        {
            return None;
        }
        if let Control::Custom(c) = line.control
            && !custom::shown(c, self, cx)
        {
            return None;
        }
        let id = format!(
            "ctl-{}",
            favorites::row_key(self.page.id, line.label).unwrap_or_default()
        );
        let control = self.control(line, id, this, cx);
        let star = self
            .stars
            .star(self.page.id, line.label, line.description, cx);
        let description = match line.varies {
            // Its words wait for what they are picked by (docs/adr/0006).
            Some(v)
                if v.when
                    .fields()
                    .iter()
                    .all(|f| f.read(settings).value().is_some()) =>
            {
                if v.when.holds_with(settings, self.jobs.read(cx).job_busy()) {
                    v.then
                } else {
                    v.otherwise
                }
            }
            _ => line.description,
        };
        // Words built from what the window holds wait for it too.
        let said = line.words.filter(|w| {
            w.reads().iter().all(|k| {
                crate::catalog::Field::Key(k)
                    .read(settings)
                    .value()
                    .is_some()
            }) && w
                .inventory()
                .is_none_or(|kind| matches!(self.inventories.read(cx).known(kind), Known::Value(_)))
        });
        let jobs = self.jobs.read(cx);
        let words = said.map(|w| {
            let inventory =
                w.inventory()
                    .and_then(|kind| match self.inventories.read(cx).known(kind) {
                        Known::Value(v) => Some(v.clone()),
                        _ => None,
                    });
            w.say(&crate::catalog::words::Said {
                settings,
                job: jobs.running().filter(|_| jobs.job_busy()),
                inventory: inventory.as_ref(),
            })
        });
        let description: SharedString = match line.control {
            Control::Custom(c) => {
                custom::description(c, self, cx).map_or(description.into(), SharedString::from)
            }
            _ => description.into(),
        };
        let description = words.map_or(description, SharedString::from);
        let (label, hint) = (line.label, line.hint);
        // A number or a line of text takes the row's width, under its words.
        Some(match line.control {
            Control::Number(..)
            | Control::Text(..)
            | Control::Minutes(_)
            | Control::Calls(_, Shape::Count(_), _) => {
                row::stacked(label, description, hint, control, star).into_any_element()
            }
            Control::Custom(c) if custom::stacked(c) => {
                row::stacked(label, description, hint, control, star).into_any_element()
            }
            Control::Custom(c) if custom::caption(c, self).is_some() => {
                let caption = custom::caption(c, self).unwrap_or_default();
                row::captioned(label, description, caption, hint, control, star).into_any_element()
            }
            _ => row::setting(label, description, hint, control, star).into_any_element(),
        })
    }

    fn control(
        &self,
        line: &'static Line,
        id: String,
        this: &Entity<Self>,
        cx: &App,
    ) -> impl IntoElement + use<> {
        let settings = self.stars.settings.read(cx);
        let busy = self.jobs.read(cx).job_busy();
        let disabled = !line.enabled.holds_with(settings, busy);
        let label = line.label;
        let reading = line.control.field().map(|f| f.read(settings));
        let current = match reading {
            Some(Reading::Waiting) => return (row::reading(label)).into_any_element(),
            Some(Reading::Unknown) => return (row::unknown()).into_any_element(),
            Some(Reading::Value(v)) => v,
            None => Value::Null,
        };
        let this = this.clone();
        match line.control {
            Control::Pending => (Badge::new("Not yet")).into_any_element(),
            Control::Switch(field) => {
                let next = Control::flip(field, &current);
                (Switch::new(SharedString::from(id), label)
                    .checked(on(&current) && !disabled)
                    .disabled(disabled)
                    .on_change(move |_, window, cx| {
                        let next = next.clone();
                        this.update(cx, |p, cx| p.set(label, field, next, window, cx))
                    }))
                .into_any_element()
            }
            Control::Number(field, range) => {
                let commit = move |value, window: &mut Window, cx: &mut App| {
                    this.update(cx, |p, cx| p.set(label, field, value, window, cx))
                };
                number(id, label, range, &current, disabled, commit).into_any_element()
            }
            Control::Picks(field, options) => {
                let commit = move |chosen: &str, window: &mut Window, cx: &mut App| {
                    let value = Value::from(chosen);
                    this.update(cx, |p, cx| p.set(label, field, value, window, cx))
                };
                let options = options.list(settings);
                select(id, label, &options, &current, disabled, commit).into_any_element()
            }
            Control::Calls(field, shape, setter) => {
                let call = move |value, window: &mut Window, cx: &mut App| {
                    this.update(cx, |p, cx| {
                        p.call(label, Some(field), setter, value, window, cx)
                    })
                };
                match shape {
                    Shape::Toggle => {
                        let next = Control::flip(field, &current);
                        Switch::new(SharedString::from(id), label)
                            .checked(on(&current) && !disabled)
                            .disabled(disabled)
                            .on_change(move |_, window, cx| call(next.clone(), window, cx))
                            .into_any_element()
                    }
                    Shape::Count(range) => {
                        number(id, label, range, &current, disabled, call).into_any_element()
                    }
                    Shape::Button(text) => {
                        let value = current.clone();
                        Button::new(SharedString::from(id))
                            .label(text)
                            .variant(ButtonVariant::Outline)
                            .disabled(disabled)
                            .on_click(move |_, window, cx| call(value.clone(), window, cx))
                            .into_any_element()
                    }
                    Shape::Pick(options) => {
                        let options = options.list(settings);
                        let pick = move |chosen: &str, window: &mut Window, cx: &mut App| {
                            call(Value::from(chosen), window, cx)
                        };
                        select(id, label, &options, &current, disabled, pick).into_any_element()
                    }
                }
            }
            Control::Shows(_) => Text::new(js::string(&current)).into_any_element(),
            Control::Custom(c) => custom::control(c, label, id, self, &this, cx).into_any_element(),
            Control::Bare => v_flex().into_any_element(),
            Control::Minutes(range) => {
                let jobs = self.jobs.clone();
                let minutes = Value::from(jobs.read(cx).minutes);
                let commit = move |value: Value, _: &mut Window, cx: &mut App| {
                    let n = value
                        .as_f64()
                        .unwrap_or(range.min)
                        .clamp(range.min, range.max);
                    jobs.update(cx, |jobs, cx| {
                        jobs.minutes = n as u32;
                        cx.notify();
                    });
                };
                number(id, label, range, &minutes, disabled, commit).into_any_element()
            }
            Control::Buttons(all) => all
                .iter()
                .enumerate()
                .fold(h_flex().gap(Space::Sm), |row, (i, b)| {
                    let this = this.clone();
                    let b = *b;
                    row.child(
                        Button::new(SharedString::from(format!("{id}-{i}")))
                            .label(b.label)
                            .variant(ButtonVariant::Outline)
                            .disabled(disabled || !b.when.holds_with(settings, busy))
                            .on_click(move |_, window, cx| {
                                this.update(cx, |p, cx| p.press(label, b, window, cx))
                            }),
                    )
                })
                .into_any_element(),
            Control::Press(text, setter) => Button::new(SharedString::from(id))
                .label(text)
                .variant(ButtonVariant::Outline)
                .disabled(disabled)
                .on_click(move |_, window, cx| {
                    this.update(cx, |p, cx| {
                        p.call(label, None, setter, Value::Null, window, cx)
                    })
                })
                .into_any_element(),
            Control::Choice(field, choices) => {
                let select = choices.iter().fold(
                    Select::new(SharedString::from(id), label),
                    |select, (value, text)| select.item(*value, *text),
                );
                (select
                    .value(js::string(&current))
                    .disabled(disabled)
                    .on_change(move |chosen, window, cx| {
                        let value = Control::chosen(field, chosen);
                        this.update(cx, |p, cx| p.set(label, field, value, window, cx))
                    }))
                .into_any_element()
            }
            Control::Text(field, placeholder) => {
                let draft = self.drafts.get(&id).cloned();
                (text(
                    id,
                    label,
                    field,
                    placeholder,
                    &current,
                    draft,
                    disabled,
                    this,
                ))
                .into_any_element()
            }
            Control::Acts(acts) => acts
                .iter()
                .fold(h_flex().gap(Space::Sm), |buttons, act| {
                    let this = this.clone();
                    buttons.child(
                        Button::new(SharedString::from(format!("{id}-{}", act.name)))
                            .label(act.label)
                            .variant(ButtonVariant::Outline)
                            .disabled(disabled)
                            .on_click(move |_, window, cx| {
                                this.update(cx, |p, cx| p.act(*act, window, cx))
                            }),
                    )
                })
                .into_any_element(),
            Control::Says(text) => Text::new(text).into_any_element(),
            Control::Go(text, page) => {
                let open = self.open.clone();
                (Button::new(SharedString::from(id))
                    .label(text)
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, _, cx| open(page, cx)))
                .into_any_element()
            }
        }
    }

    /// A built-in profile's Apply: each of its values, written as a setting.
    pub fn apply(
        &mut self,
        profile: &'static Profile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let client = self.stars.client.clone();
        let refused = format!("{} was not applied", profile.title);
        self.stars.settings.update(cx, |settings, cx| {
            for (key, body) in profile.writes() {
                settings.send(&client, key, &body, refused.clone(), window, cx);
            }
        });
    }

    /// A built-in profile's row, with Apply.
    pub(super) fn profile(
        &self,
        p: &'static Profile,
        this: &Entity<Self>,
        cx: &App,
    ) -> impl IntoElement + use<> {
        let this = this.clone();
        let apply = Button::new(SharedString::from(format!("profile-{}", p.id)))
            .label("Apply")
            .variant(ButtonVariant::Primary)
            .on_click(move |_, window, cx| this.update(cx, |page, cx| page.apply(p, window, cx)));
        let star = self.stars.star(self.page.id, p.title, p.description, cx);
        row::setting(p.title, p.description, profiles::HINT, apply, star)
    }

    /// The group as the page's grid holds it, each with whether it takes a row of its own: the
    /// group, then each group the QML keeps out of the catalog that follows it. None when the
    /// group is hidden.
    fn group(
        &self,
        group: &'static Group,
        this: &Entity<Self>,
        cx: &App,
    ) -> Vec<(impl IntoElement + use<>, bool)> {
        let id = self.page.id;
        let list = catalog::list(self.page.id, group.title);
        let mut labels = Vec::new();
        let mut rows: Vec<_> = group
            .rows
            .iter()
            .filter_map(|l| {
                let row = self.line(l, this, cx)?;
                labels.push(l.label);
                Some(row.into_any_element())
            })
            .collect();
        if let Some(list) = list {
            let drawn = crate::pages::lists::list_rows(list, self, this, cx)
                .into_iter()
                .map(|r| r.into_any_element());
            let after = list
                .follows()
                .and_then(|label| labels.iter().position(|l| *l == label));
            let at = match after {
                Some(i) => i + 1,
                None if list.leads() => 0,
                None => rows.len(),
            };
            rows.splice(at..at, drawn);
        }
        let column = v_flex().gap(Space::Xs);
        if rows.is_empty() && !custom::group_shows(self, group.title, cx) {
            return Vec::new();
        }
        if group.title.is_empty() {
            return vec![(column.children(rows).into_any_element(), true)];
        }
        let empty = (group.rows.is_empty() && list.is_none()).then(|| Text::new(LISTED).muted());
        // A group the QML keeps out of the catalog, drawn after the one it follows.
        let unlisted = catalog::UNLISTED
            .iter()
            .filter(|(p, after, ..)| *p == id && *after == group.title)
            .map(|(_, _, title, detail, list)| {
                let rows = crate::pages::lists::list_rows(*list, self, this, cx);
                // One with no title is its words and rows, as the QML draws a group of none.
                let drawn = if title.is_empty() {
                    v_flex()
                        .gap(Space::Xs)
                        .child(Text::new(*detail).muted())
                        .children(rows)
                        .into_any_element()
                } else {
                    row::group(*title, *detail, rows).into_any_element()
                };
                (drawn, catalog::wide(id, title))
            });
        let drawn = column
            .child(row::group(
                group.title,
                list.and_then(|l| agents::detail(l, self))
                    .unwrap_or_else(|| group.detail.to_owned()),
                rows,
            ))
            .children(empty);
        std::iter::once((drawn.into_any_element(), catalog::wide(id, group.title)))
            .chain(unlisted)
            .collect()
    }
}

impl Render for CatalogPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.page;
        let this = cx.entity();
        // The groups two side by side once the window has room, as `PrefsPage.qml` puts them; the
        // id is the same on every page, so a page opened from another keeps the count.
        let groups: Vec<_> = page
            .groups
            .iter()
            .flat_map(|g| self.group(g, &this, cx))
            .collect();
        let cells = groups.iter().filter(|(_, wide)| !wide).count();
        let column = content_column_for_grid("catalog-page", 2, cells, window, cx);
        let grid = groups
            .into_iter()
            .fold(Grid::new(column.columns()), |grid, (group, wide)| {
                if wide {
                    grid.wide(group)
                } else {
                    grid.child(group)
                }
            });
        let settings = self.stars.settings.read(cx);
        let held = page.held();
        // Keys that failed for one reason (one tool that did not answer) are said once.
        let mut failed: Vec<(&str, Vec<&Held>)> = Vec::new();
        for (held, why) in settings.failures(&held) {
            match failed.iter_mut().find(|(w, _)| *w == why) {
                Some((_, keys)) => keys.push(held),
                None => failed.push((why, vec![held])),
            }
        }
        let alerts = failed.into_iter().map(|(why, keys)| {
            let title = match keys.as_slice() {
                [one] => one.unread.to_owned(),
                many => format!("{} settings on this page could not be read", many.len()),
            };
            Alert::new(
                SharedString::from(format!("catalog-unread-{}", keys[0].key)),
                title,
            )
            .variant(Tone::Danger)
            .description(why.to_owned())
        });
        let notice = (!page.mapped()).then(|| {
            Alert::new("catalog-pending", PENDING)
                .variant(Tone::Info)
                .description(catalog::blocked(page.id).unwrap_or(PENDING_DETAIL))
        });
        let body = v_flex()
            .gap(Space::Xl)
            .child(Text::new(page.description()).muted())
            .children(notice)
            .children(alerts)
            .child(grid)
            .into_any_element();
        let column = column.child(Heading::new(page.title()).level(1));
        let asked = (self.asked.as_ref()).map(|a| (a.title.clone(), a.message.clone(), a.action));
        let dialogs = [
            confirm(self.asking.and_then(|(b, _)| b.confirm), asked, &this).into_any_element(),
            prompt(self.prompting.clone(), self.jobs.read(cx).running(), &this).into_any_element(),
            rules::dialog(self, &this).into_any_element(),
            crate::pages::bindings::dialog(self, &this).into_any_element(),
            crate::pages::accounts::dialog(self, &this, cx).into_any_element(),
            crate::pages::wifi::dialog(self, &this).into_any_element(),
            appearance::dialog(self, &this, cx).into_any_element(),
            crate::pages::units::dialog(self, &this).into_any_element(),
            crate::pages::launchers::dialog(self, &this, cx).into_any_element(),
            crate::pages::disks::luks_dialog(self, &this, cx).into_any_element(),
            crate::pages::hooks::dialog(self, &this).into_any_element(),
        ];
        let Some(hub) = hubs::hub_of(page.id).filter(|h| !h.children.is_empty()) else {
            return column.child(body).children(dialogs);
        };
        // The hub's pages as tabs; only the one shown is built, and it is this page.
        let mut body = Some(body);
        let pages = std::iter::once((hub.id, hub.title))
            .chain(hub.children.iter().map(|c| (c.id, c.title)));
        let tabs = pages.fold(
            Tabs::new("catalog-tabs", format!("{} pages", hub.title)).value(page.id),
            |tabs, (id, title)| {
                let shown = if id == page.id { body.take() } else { None };
                tabs.tab(id, title, move |_, _| {
                    shown.unwrap_or_else(|| v_flex().into_any_element())
                })
            },
        );
        let open = self.open.clone();
        column
            .child(tabs.on_change(move |id, _, cx| open(id, cx)))
            .children(dialogs)
    }
}
