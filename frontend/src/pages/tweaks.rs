//! Tweaks: settings that do not deserve their own sidebar entry (`pages/TweaksPage.qml`). The
//! page holds the two settings topics its keys are in while it is shown (the shell does,
//! `app.rs`), and draws each row from what the window's settings say of them (`tweaks.rs`).
//!
//! A toggle is controlled by the snapshot: it changes when the backend reports the change, not
//! when it is clicked (docs/adr/0005). A row whose key has never been read draws no toggle and
//! says why (docs/adr/0006). The QML page's search filter is not here: the Rust window has no
//! search yet (docs/adr/0013).

use odk::prelude::*;
use serde_json::Value;

use crate::client::Client;
use crate::settings::Settings;
use crate::tweaks::{self, HELD, Reading, Tweak};

pub struct TweaksPage {
    client: Client,
    settings: Entity<Settings>,
}

impl TweaksPage {
    /// The page sends nothing of its own until a toggle or a Reset is pressed, and is drawn again
    /// whenever the window's settings change.
    pub fn new(client: Client, settings: Entity<Settings>, cx: &mut Context<Self>) -> Self {
        cx.observe(&settings, |_, _, cx| cx.notify()).detach();
        Self { client, settings }
    }

    /// What the page shows for a tweak now.
    pub fn reading(&self, id: &str, cx: &App) -> Option<Reading> {
        tweaks::by_id(id).map(|t| tweaks::reading(self.settings.read(cx), t))
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

    /// A toggle: the opposite of what the backend last reported is written.
    pub fn toggle(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tweak) = tweaks::by_id(id) else {
            return;
        };
        if let Reading::On(on) = tweaks::reading(self.settings.read(cx), tweak) {
            self.send(tweak, tweak.write(!on), "was not changed", window, cx);
        }
    }

    /// Reset: the tweak's reset value is written.
    pub fn reset(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tweak) = tweaks::by_id(id) {
            self.send(tweak, tweak.reset(), "was not reset", window, cx);
        }
    }

    fn send(
        &mut self,
        tweak: &'static Tweak,
        body: Value,
        refused: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let client = self.client.clone();
        self.settings.update(cx, |settings, cx| {
            let refused = format!("{} {refused}", tweak.label);
            settings.send(&client, tweak.snapshot_key(), &body, refused, window, cx);
        });
    }
}

const DESCRIPTION: &str = "Settings that do not deserve their own sidebar entry. Each row says what it writes. Reset puts that one tweak back.";

impl Render for TweaksPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let settings = self.settings.read(cx);
        let alerts = settings.failures(HELD).into_iter().map(|(held, why)| {
            Alert::new(
                SharedString::from(format!("tweaks-unread-{}", held.key)),
                held.unread,
            )
            .variant(Tone::Danger)
            .description(why.to_owned())
        });
        let groups = tweaks::groups().into_iter().map(|(name, items)| {
            v_flex()
                .gap(Space::Xs)
                .child(Heading::new(name).level(2))
                .child(Text::new("Each row names the file it writes.").muted())
                .children(
                    items
                        .into_iter()
                        .map(|t| row(tweaks::reading(settings, t), t, &this)),
                )
        });
        content_column()
            .child(Heading::new("Tweaks").level(1))
            .child(Text::new(DESCRIPTION).muted())
            .children(alerts)
            .children(groups)
    }
}

/// One tweak: its toggle and label, what it does, the file it writes, and Reset.
fn row(reading: Reading, t: &'static Tweak, this: &Entity<TweaksPage>) -> impl IntoElement {
    let control = match reading {
        Reading::On(on) => {
            let this = this.clone();
            Switch::new(SharedString::from(format!("tweak-{}", t.id)), t.label)
                .checked(on)
                .on_change(move |_, window, cx| {
                    this.update(cx, |page, cx| page.toggle(t.id, window, cx))
                })
                .into_any_element()
        }
        // Not a toggle with a state made up for it: the key is being read, or could not be.
        Reading::Waiting => v_flex()
            .gap(Space::Xs)
            .child(Text::new(t.label))
            .child(Skeleton::line().label(format!("Reading {}", t.label)))
            .into_any_element(),
        Reading::Unknown(_) => v_flex()
            .gap(Space::Xs)
            .child(Text::new(t.label))
            .child(Text::new("Unknown").muted())
            .into_any_element(),
    };
    let reset = Button::new(SharedString::from(format!("reset-{}", t.id)))
        .label("Reset")
        .variant(ButtonVariant::Outline)
        .accessible_name(format!("Reset {}", t.label))
        .on_click({
            let this = this.clone();
            move |_, window, cx| this.update(cx, |page, cx| page.reset(t.id, window, cx))
        });
    h_flex()
        .gap(Space::Md)
        .py(Space::Sm)
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(Space::Xs)
                .child(control)
                .child(Text::new(t.description).muted())
                .child(Text::new(t.modifies).mono().muted()),
        )
        .child(reset)
}
