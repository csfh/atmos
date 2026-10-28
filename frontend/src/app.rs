//! The window: a sidebar of hubs, a page, and the screen shown in its place when the backend is
//! not there. A hub that has not been ported shows its name and says so; Sentinel
//! (`pages/sentinel.rs`), Tweaks (`pages/tweaks.rs`), Defaults (`pages/defaults.rs`),
//! Favorites (`pages/favorites.rs`) and Drivers (`pages/drivers.rs`) have been. Every page reads
//! the settings topics from one [`Settings`], the inventories from one [`Inventories`], and runs
//! its jobs through one [`Jobs`], whose sudo prompt the shell draws.
//!
//! The view holds what the backend has said and draws it. The client's notices arrive on a
//! channel that one task awaits, so the view is drawn when something happened and not on a clock.

use std::rc::Rc;

use async_channel::Receiver;
use odk::prelude::*;

use crate::client::{Client, Link, Notice};
use crate::hubs;
use crate::inventory::{self, Inventories};
use crate::jobs::Jobs;
use crate::pages::defaults::DefaultsPage;
use crate::pages::drivers::DriversPage;
use crate::pages::favorites::FavoritesPage;
use crate::pages::sentinel::SentinelPage;
use crate::pages::sudo;
use crate::pages::tweaks::TweaksPage;
use crate::protocol::{self, Daemon, Event};
use crate::settings::{self, Settings};
use crate::{defaults, drivers, favorites, status, tweaks};

/// The hubs that have a page in this window; every other hub says it has not been ported.
pub const PORTED: &[&str] = &["sentinel", "tweaks", "defaults", "favorites", "drivers"];

/// How often the daemon samples `live` for a page that reads it, in milliseconds: the QML window's.
const LIVE_MS: u64 = 2000;

/// The topics a page needs the daemon to hold while it is shown. A page that reads nothing live
/// holds none, and nothing is sampled for it. A settings topic is held at 0, "when it changes":
/// the daemon collects it when it is first held and again when something changes it
/// (docs/adr/0011).
fn wanted(page: &str) -> Vec<(&'static str, u64)> {
    match page {
        "sentinel" => vec![("live", LIVE_MS)],
        "tweaks" => starred(tweaks::HELD),
        "defaults" => starred(defaults::HELD),
        "favorites" => starred(&[]),
        "drivers" => {
            let mut held = starred(drivers::HELD);
            held.push(("hardware", 0));
            held
        }
        _ => Vec::new(),
    }
}

/// The topics of a settings page's keys and of the favorites list its stars read.
fn starred(held: &[settings::Held]) -> Vec<(&'static str, u64)> {
    let mut held = held.to_vec();
    held.push(favorites::HELD);
    settings::topics(&held)
}

pub struct Atmos {
    client: Client,
    /// The hub on show.
    page: SharedString,
    link: Link,
    /// What the server last said of the daemon; unknown until it does, and again after a restart.
    daemon: Option<Daemon>,
    /// The backend's version, once it has answered.
    version: Option<SharedString>,
    /// What the settings topics and the writes' read-backs said, for every page.
    settings: Entity<Settings>,
    /// The jobs the pages start, one at a time, and the sudo prompt.
    jobs: Entity<Jobs>,
    /// The five inventories, for every page that reads one.
    inventories: Entity<Inventories>,
    sentinel: Entity<SentinelPage>,
    tweaks: Entity<TweaksPage>,
    defaults: Entity<DefaultsPage>,
    favorites: Entity<FavoritesPage>,
    drivers: Entity<DriversPage>,
}

impl Atmos {
    /// A window on `page`, a hub id (the first hub when it names none). It follows `notices` for
    /// as long as it lives.
    pub fn new(
        client: Client,
        notices: Receiver<Notice>,
        page: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let page = hubs::resolve(page).map_or(hubs::HUBS[0].id, |hub| hub.id);
        let version = client.request(&protocol::version());
        let sentinel = {
            let client = client.clone();
            cx.new(|cx| SentinelPage::new(client, cx))
        };
        let settings = cx.new(|_| Settings::default());
        let jobs = {
            let (client, settings) = (client.clone(), settings.clone());
            cx.new(|_| Jobs::new(client, settings))
        };
        // The shell draws the sudo prompt, so it is drawn again whenever the jobs change.
        cx.observe(&jobs, |_, _, cx| cx.notify()).detach();
        let inventories = cx.new(|_| Inventories::default());
        let drivers = {
            let (client, settings) = (client.clone(), settings.clone());
            let (inventories, jobs) = (inventories.clone(), jobs.clone());
            cx.new(|cx| DriversPage::new(client, settings, inventories, jobs, cx))
        };
        let tweaks = {
            let (client, settings) = (client.clone(), settings.clone());
            cx.new(|cx| TweaksPage::new(client, settings, cx))
        };
        let defaults = {
            let (client, settings) = (client.clone(), settings.clone());
            cx.new(|cx| DefaultsPage::new(client, settings, cx))
        };
        let favorites = {
            let (client, settings) = (client.clone(), settings.clone());
            let atmos = cx.weak_entity();
            let open = Rc::new(move |hub: &str, cx: &mut App| {
                let _ = atmos.update(cx, |atmos, cx| atmos.show(hub, cx));
            });
            cx.new(|cx| FavoritesPage::new(client, settings, open, cx))
        };
        // The topics of the page it opens on. The server holds them again after a restart.
        drop(client.hold(&wanted(page)));
        cx.spawn_in(window, async move |this, cx| {
            while let Ok(notice) = notices.recv().await {
                if this
                    .update_in(cx, |atmos, window, cx| {
                        atmos.apply(notice, window, cx);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        cx.spawn(async move |this, cx| {
            if let Ok(version) = version.await {
                let _ = this.update(cx, |atmos, cx| {
                    atmos.version = version.as_str().map(|v| format!("atmosd {v}").into());
                    cx.notify();
                });
            }
        })
        .detach();
        Self {
            client,
            page: page.into(),
            link: Link::Starting,
            daemon: None,
            version: None,
            settings,
            jobs,
            inventories,
            sentinel,
            tweaks,
            defaults,
            favorites,
            drivers,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn page(&self) -> &str {
        &self.page
    }

    /// The Sentinel page, for what a test asks of it.
    pub fn sentinel(&self) -> &Entity<SentinelPage> {
        &self.sentinel
    }

    /// The jobs, for what a test asks of them.
    pub fn jobs(&self) -> &Entity<Jobs> {
        &self.jobs
    }

    /// The Tweaks page, for what a test asks of it.
    pub fn tweaks(&self) -> &Entity<TweaksPage> {
        &self.tweaks
    }

    /// The Defaults page, for what a test asks of it.
    pub fn defaults(&self) -> &Entity<DefaultsPage> {
        &self.defaults
    }

    /// The Drivers page, for what a test asks of it.
    pub fn drivers(&self) -> &Entity<DriversPage> {
        &self.drivers
    }

    /// The Favorites page, for what a test asks of it.
    pub fn favorites(&self) -> &Entity<FavoritesPage> {
        &self.favorites
    }

    /// Shows the hub `page` names, or the hub a page of it is under: the window has no hub pages
    /// of its own yet.
    pub fn show(&mut self, page: &str, cx: &mut Context<Self>) {
        if let Some(hub) = hubs::resolve(page) {
            let before = wanted(&self.page);
            self.page = hub.id.into();
            // The daemon samples for the page in view and for no other: `hold` replaces what was
            // held, and a page with nothing live to read releases the feed.
            let now = wanted(&self.page);
            if now != before {
                drop(self.client.hold(&now));
            }
            cx.notify();
        }
    }

    fn apply(&mut self, notice: Notice, window: &mut Window, cx: &mut Context<Self>) {
        match notice {
            Notice::Link(link) => {
                // A server that comes back knows nothing of the daemon until it says.
                if link == Link::Up {
                    self.daemon = None;
                    self.sentinel.update(cx, |page, cx| page.backend_up(cx));
                    self.jobs.update(cx, |jobs, cx| jobs.reattach(window, cx));
                }
                self.link = link;
            }
            Notice::Event(Event::Backend(daemon)) => {
                // A daemon that comes back has lost this window's session: its job is attached
                // again.
                let back = matches!(self.daemon, Some(Daemon::Down { .. })) && daemon == Daemon::Up;
                self.daemon = Some(daemon);
                if back {
                    self.jobs.update(cx, |jobs, cx| jobs.reattach(window, cx));
                }
            }
            Notice::Event(Event::Sample {
                kind,
                sampled_at: Some(at),
                stale: false,
                result: Some(result),
                ..
            }) if kind == "live" => {
                self.sentinel
                    .update(cx, |page, cx| page.adopt(at as f64, &result, cx));
            }
            Notice::Event(Event::Sample {
                kind,
                sampled_at,
                stale,
                result,
                error,
            }) if settings::is_topic(&kind) => {
                self.settings
                    .update(cx, |settings, cx| match (stale, sampled_at, result) {
                        (false, Some(at), Some(result)) => {
                            if settings.adopt(&kind, at, &result) {
                                cx.notify();
                            }
                        }
                        (true, _, _) => {
                            settings.topic_failed(
                                &kind,
                                error
                                    .as_deref()
                                    .unwrap_or("The daemon could not collect it."),
                            );
                            cx.notify();
                        }
                        _ => {}
                    });
            }
            Notice::Event(Event::Sample {
                kind,
                sampled_at,
                stale,
                result,
                error,
            }) if inventory::is_kind(&kind) => {
                self.inventories
                    .update(cx, |inv, cx| match (stale, sampled_at, result) {
                        (false, Some(at), Some(result)) => {
                            if inv.adopt(&kind, at, &result) {
                                cx.notify();
                            }
                        }
                        (true, _, _) => {
                            inv.failed(
                                &kind,
                                error
                                    .as_deref()
                                    .unwrap_or("The daemon could not collect it."),
                            );
                            cx.notify();
                        }
                        _ => {}
                    });
            }
            Notice::Event(Event::Job(event)) => {
                self.jobs
                    .update(cx, |jobs, cx| jobs.event(&event, window, cx));
            }
            Notice::Event(_) => {}
        }
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let nav = hubs::clusters().into_iter().fold(
            NavList::new("nav", "Pages").value(self.page.clone()),
            |nav, (heading, list)| {
                let nav = if heading.is_empty() {
                    nav
                } else {
                    nav.group(heading)
                };
                list.into_iter()
                    .fold(nav, |nav, hub| nav.item(hub.id, hub.title))
            },
        );
        let nav = nav.on_change(move |page, _, cx| {
            this.update(cx, |atmos, cx| atmos.show(page, cx));
        });
        v_flex()
            .size_full()
            .child(scroll_y("nav-scroll").child(nav))
            .children(
                self.version
                    .clone()
                    .map(|version| v_flex().p(Space::Sm).child(Text::new(version).muted())),
            )
    }

    fn content(&self) -> impl IntoElement {
        if let Some(notice) = status::unavailable(&self.link, self.daemon.as_ref()) {
            return content_column()
                .child(
                    Alert::new("backend-unavailable", notice.title)
                        .variant(Tone::Danger)
                        .description(notice.description()),
                )
                .into_any_element();
        }
        // The pages that have been ported; every other hub says it has not.
        match self.page() {
            "sentinel" => return self.sentinel.clone().into_any_element(),
            "tweaks" => return self.tweaks.clone().into_any_element(),
            "defaults" => return self.defaults.clone().into_any_element(),
            "favorites" => return self.favorites.clone().into_any_element(),
            "drivers" => return self.drivers.clone().into_any_element(),
            _ => {}
        }
        let title = hubs::find(&self.page).map_or("Atmos", |hub| hub.title);
        content_column()
            .child(Heading::new(title).level(1))
            .child(Text::new("This page is not in the new window yet.").muted())
            .into_any_element()
    }
}

impl Render for Atmos {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        root()
            .child(sidebar_frame(
                self.sidebar(cx),
                scroll_y("page").child(self.content()),
            ))
            .child(sudo::prompt(&self.jobs, cx))
    }
}
