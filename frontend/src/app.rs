//! The window: a sidebar of hubs, a page, and the screen shown in its place when the backend is
//! not there. Every page is yet to come, so each hub shows its name and says so.
//!
//! The view holds what the backend has said and draws it. The client's notices arrive on a
//! channel that one task awaits, so the view is drawn when something happened and not on a clock.

use async_channel::Receiver;
use odk::prelude::*;

use crate::client::{Client, Link, Notice};
use crate::hubs;
use crate::protocol::{self, Daemon, Event};
use crate::status;

pub struct Atmos {
    client: Client,
    /// The hub on show.
    page: SharedString,
    link: Link,
    /// What the server last said of the daemon; unknown until it does, and again after a restart.
    daemon: Option<Daemon>,
    /// The backend's version, once it has answered.
    version: Option<SharedString>,
}

impl Atmos {
    /// A window on `page`, a hub id (the first hub when it names none). It follows `notices` for
    /// as long as it lives.
    pub fn new(
        client: Client,
        notices: Receiver<Notice>,
        page: &str,
        cx: &mut Context<Self>,
    ) -> Self {
        let page = hubs::resolve(page).map_or(hubs::HUBS[0].id, |hub| hub.id);
        let version = client.request(&protocol::version());
        cx.spawn(async move |this, cx| {
            while let Ok(notice) = notices.recv().await {
                if this
                    .update(cx, |atmos, cx| {
                        atmos.apply(notice);
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
                    atmos.version = version.as_str().map(|v| format!("ratmos {v}").into());
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
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn page(&self) -> &str {
        &self.page
    }

    pub fn show(&mut self, page: &str, cx: &mut Context<Self>) {
        if let Some(hub) = hubs::find(page) {
            self.page = hub.id.into();
            cx.notify();
        }
    }

    fn apply(&mut self, notice: Notice) {
        match notice {
            Notice::Link(link) => {
                // A server that comes back knows nothing of the daemon until it says.
                if link == Link::Up {
                    self.daemon = None;
                }
                self.link = link;
            }
            Notice::Event(Event::Backend(daemon)) => self.daemon = Some(daemon),
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
            return content_column().child(
                Alert::new("backend-unavailable", notice.title)
                    .variant(Tone::Danger)
                    .description(notice.description()),
            );
        }
        let title = hubs::find(&self.page).map_or("Atmos", |hub| hub.title);
        content_column()
            .child(Heading::new(title).level(1))
            .child(Text::new("This page is not in the new window yet.").muted())
    }
}

impl Render for Atmos {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        root().child(sidebar_frame(
            self.sidebar(cx),
            scroll_y("page").child(self.content()),
        ))
    }
}
