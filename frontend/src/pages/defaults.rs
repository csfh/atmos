//! Defaults: the programs Omarchy opens when something asks for one (`pages/DefaultsPage.qml`).
//! The page holds the topics its keys are in while it is shown (the shell does, `app.rs`), and
//! draws each picker from what the window's settings say (`defaults.rs`).
//!
//! A picker is controlled by the snapshot: it shows a choice when the backend reports it, not
//! when it is made (docs/adr/0005). A key that has never been read draws no picker and says why
//! (docs/adr/0006). The QML page's search filter and its Advanced toggle are not here: the Rust
//! window has neither yet (docs/adr/0013).

use odk::prelude::*;

use crate::client::Client;
use crate::defaults::{self, HELD, Picker};
use crate::pages::row;
use crate::settings::{Known, Settings};

pub struct DefaultsPage {
    client: Client,
    settings: Entity<Settings>,
}

impl DefaultsPage {
    /// The page sends nothing of its own until a choice is made, and is drawn again whenever the
    /// window's settings change.
    pub fn new(client: Client, settings: Entity<Settings>, cx: &mut Context<Self>) -> Self {
        cx.observe(&settings, |_, _, cx| cx.notify()).detach();
        Self { client, settings }
    }

    /// What a picker shows as chosen now, once its key has been read.
    pub fn current(&self, id: &str, cx: &App) -> Option<String> {
        defaults::by_id(id)?.current(self.settings.read(cx))
    }

    /// The programs a picker offers now, by value.
    pub fn offered(&self, id: &str, cx: &App) -> Vec<String> {
        defaults::by_id(id).map_or_else(Vec::new, |p| {
            p.options(self.settings.read(cx))
                .into_iter()
                .map(|(value, _)| value)
                .collect()
        })
    }

    /// The snapshot keys that could not be read, each of which the page says.
    pub fn unread(&self, cx: &App) -> Vec<&'static str> {
        self.settings
            .read(cx)
            .failures(HELD)
            .into_iter()
            .map(|(held, _)| held.key)
            .collect()
    }

    /// A choice: written when it changes what the backend last reported.
    pub fn choose(&mut self, id: &str, chosen: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(picker) = defaults::by_id(id) else {
            return;
        };
        let Some(body) = picker.choose(chosen, self.settings.read(cx)) else {
            return;
        };
        let client = self.client.clone();
        self.settings.update(cx, |settings, cx| {
            let refused = format!("{} was not changed", picker.label);
            settings.send(&client, picker.domain, &body, refused, window, cx);
        });
    }
}

const DESCRIPTION: &str = "The programs Omarchy opens when something asks for a default.";

impl Render for DefaultsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let settings = self.settings.read(cx);
        let alerts = settings.failures(HELD).into_iter().map(|(held, why)| {
            Alert::new(
                SharedString::from(format!("defaults-unread-{}", held.key)),
                held.unread,
            )
            .variant(Tone::Danger)
            .description(why.to_owned())
        });
        let groups = defaults::groups().into_iter().map(|(group, pickers)| {
            row::group(
                group.title(),
                group.detail(),
                pickers.into_iter().map(|p| picker(p, settings, &this)),
            )
        });
        content_column()
            .child(Heading::new("Defaults").level(1))
            .child(Text::new(DESCRIPTION).muted())
            .children(alerts)
            .children(groups)
    }
}

/// One picker: what it chooses, and the programs it offers once its key has been read.
fn picker(
    p: &'static Picker,
    settings: &Settings,
    this: &Entity<DefaultsPage>,
) -> impl IntoElement {
    let control = match p.known(settings) {
        Known::Waiting => row::reading(p.label).into_any_element(),
        Known::Unknown(_) => row::unknown().into_any_element(),
        Known::Value(_) => {
            let options = p.options(settings);
            let this = this.clone();
            options
                .iter()
                .fold(
                    Select::new(SharedString::from(format!("default-{}", p.id)), p.label),
                    |select, (value, label)| select.item(value.clone(), label.clone()),
                )
                .value(p.current(settings).unwrap_or_default())
                .disabled(options.is_empty())
                .on_change(move |chosen, window, cx| {
                    this.update(cx, |page, cx| page.choose(p.id, chosen, window, cx))
                })
                .into_any_element()
        }
    };
    row::setting(p.label, p.description, p.hint, control)
}
