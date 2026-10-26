//! Tweaks: settings that do not deserve their own sidebar entry (`pages/TweaksPage.qml`). The
//! page holds the two settings topics its keys are in while it is shown (the shell does,
//! `app.rs`), and draws each row from what they and the writes' read-backs said (`tweaks.rs`).
//!
//! A toggle is controlled by the snapshot: it changes when the backend reports the change, not
//! when it is clicked (docs/adr/0005). A row whose key has never been read draws no toggle and
//! says why (docs/adr/0006). The QML page's search filter is not here: the Rust window has no
//! search yet (docs/adr/0013).

use std::collections::HashMap;

use odk::prelude::*;
use serde_json::Value;

use crate::client::Client;
use crate::tweaks::{self, Reading, Snapshot, Tweak};

pub struct TweaksPage {
    client: Client,
    snapshot: Snapshot,
    /// The newest write of each snapshot key: only its reply is shown.
    sent: HashMap<&'static str, u64>,
}

impl TweaksPage {
    /// The page sends nothing of its own until a toggle or a Reset is pressed.
    pub fn new(client: Client, _: &mut Context<Self>) -> Self {
        Self {
            client,
            snapshot: Snapshot::default(),
            sent: HashMap::new(),
        }
    }

    /// What the page shows for a tweak now.
    pub fn reading(&self, id: &str) -> Option<Reading> {
        tweaks::by_id(id).map(|t| self.snapshot.reading(t))
    }

    /// The snapshot keys that could not be read, each of which the page says.
    pub fn unread(&self) -> Vec<&'static str> {
        self.snapshot
            .failures()
            .into_iter()
            .map(|(held, _)| held.key)
            .collect()
    }

    /// A sample of a settings topic from the daemon.
    pub fn adopt(&mut self, topic: &str, sampled_at: u64, result: &Value, cx: &mut Context<Self>) {
        if self.snapshot.adopt(topic, sampled_at, result) {
            cx.notify();
        }
    }

    /// A topic the daemon could not collect: what is shown stays, and the page says why.
    pub fn topic_failed(&mut self, topic: &str, why: &str, cx: &mut Context<Self>) {
        if tweaks::holds(topic) {
            self.snapshot.topic_failed(topic, why);
            cx.notify();
        }
    }

    /// A toggle: the opposite of what the backend last reported is written.
    pub fn toggle(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tweak) = tweaks::by_id(id) else {
            return;
        };
        if let Reading::On(on) = self.snapshot.reading(tweak) {
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
        let key = tweak.snapshot_key();
        let seq = self.sent.get(key).copied().unwrap_or(0) + 1;
        self.sent.insert(key, seq);
        let reply = self.client.request(&body);
        cx.spawn_in(window, async move |this, cx| {
            let outcome = reply.await;
            let _ = this.update_in(cx, |page, window, cx| match outcome {
                // A newer write of the key is on its way; its reply is the one to show.
                Ok(result) if page.sent.get(key) == Some(&seq) => {
                    page.snapshot.adopt_reply(&result);
                    cx.notify();
                }
                Ok(_) => {}
                Err(failure) => {
                    toast(
                        window,
                        cx,
                        Toast::new(format!("{} {refused}", tweak.label))
                            .description(failure.message)
                            .variant(Tone::Danger),
                    );
                }
            });
        })
        .detach();
    }
}

const DESCRIPTION: &str = "Settings that do not deserve their own sidebar entry. Each row says what it writes. Reset puts that one tweak back.";

impl Render for TweaksPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let alerts = self.snapshot.failures().into_iter().map(|(held, why)| {
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
                .children(items.into_iter().map(|t| self.row(t, &this)))
        });
        content_column()
            .child(Heading::new("Tweaks").level(1))
            .child(Text::new(DESCRIPTION).muted())
            .children(alerts)
            .children(groups)
    }
}

impl TweaksPage {
    /// One tweak: its toggle and label, what it does, the file it writes, and Reset.
    fn row(&self, t: &'static Tweak, this: &Entity<Self>) -> impl IntoElement {
        let control = match self.snapshot.reading(t) {
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
}
