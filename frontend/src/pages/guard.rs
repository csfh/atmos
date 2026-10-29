//! The keep-or-revert bar (`components/PrefsGuardBar.qml`): while a guarded write waits, it
//! says how long until it is put back, with Keep and Revert. It is a view of its own, drawn
//! again whenever the settings change, so the countdown moves without the whole window. The QML
//! bar also reverts on Esc; `odk::prelude` has no key events, so here it is the two buttons.

use odk::prelude::*;

use crate::client::Client;
use crate::guard;
use crate::settings::{Settings, now_ms};

pub struct GuardBar {
    settings: Entity<Settings>,
    client: Client,
}

impl GuardBar {
    pub fn new(settings: Entity<Settings>, client: Client, cx: &mut Context<Self>) -> Self {
        cx.observe(&settings, |_, _, cx| cx.notify()).detach();
        GuardBar { settings, client }
    }
}

impl Render for GuardBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let g = &self.settings.read(cx).guard;
        let (open, left) = (g.open, g.seconds_left(now_ms()));
        let ms_left = g.deadline.saturating_sub(now_ms());
        let (kept, reverted, client) = (
            self.settings.clone(),
            self.settings.clone(),
            self.client.clone(),
        );
        let bar = open.then(|| {
            let buttons = h_flex()
                .gap(Space::Sm)
                .child(
                    Button::new("guard-keep")
                        .label("Keep")
                        .variant(ButtonVariant::Primary)
                        .on_click(move |_, _, cx| kept.update(cx, |s, cx| s.keep_guard(cx))),
                )
                .child(
                    Button::new("guard-revert")
                        .label("Revert")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, window, cx| {
                            reverted.update(cx, |s, cx| s.revert_guard(&client, window, cx))
                        }),
                );
            v_flex()
                .gap(Space::Xs)
                .child(
                    Alert::new("guard", guard::says(left))
                        .variant(Tone::Warning)
                        .description("Keep the change if everything still works.")
                        .action(buttons),
                )
                .child(
                    Progress::new("guard-left", "Time left")
                        .value(ms_left as f64)
                        .max(guard::WINDOW_MS as f64),
                )
        });
        v_flex().children(bar)
    }
}
