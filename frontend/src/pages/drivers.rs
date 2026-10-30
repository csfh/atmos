//! Drivers: the GPUs and their drivers, the hybrid switch and firmware updates
//! (`pages/DriversPage.qml`). The GPUs are the `hardware` inventory's, the rest settings keys
//! (`drivers.rs`); the switch and the update are jobs that need root and ask first.

use odk::prelude::*;

use crate::client::Client;
use crate::drivers::{self, Graphics};
use crate::inventory::Inventories;
use crate::jobs::{Job, Jobs};
use crate::pages::row::{self, Stars};
use crate::protocol;
use crate::settings::{self, Known, Settings};

const HUB: &str = "drivers";

/// What the page is asking before it runs a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    Switch,
    Firmware,
}

pub struct DriversPage {
    client: Client,
    settings: Entity<Settings>,
    inventories: Entity<Inventories>,
    jobs: Entity<Jobs>,
    stars: Stars,
    asking: Option<Ask>,
}

impl DriversPage {
    pub fn new(
        client: Client,
        settings: Entity<Settings>,
        inventories: Entity<Inventories>,
        jobs: Entity<Jobs>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&settings, |_, _, cx| cx.notify()).detach();
        cx.observe(&inventories, |_, _, cx| cx.notify()).detach();
        cx.observe(&jobs, |_, _, cx| cx.notify()).detach();
        let stars = Stars {
            client: client.clone(),
            settings: settings.clone(),
        };
        Self {
            client,
            settings,
            inventories,
            jobs,
            stars,
            asking: None,
        }
    }

    pub fn asking(&self) -> Option<Ask> {
        self.asking
    }

    /// The GPUs listed now, each summed up.
    pub fn gpus(&self, cx: &App) -> Vec<String> {
        match self.inventories.read(cx).known("hardware") {
            Known::Value(doc) => drivers::gpus(doc)
                .iter()
                .map(drivers::gpu_summary)
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn ask(&mut self, ask: Ask, cx: &mut Context<Self>) {
        self.asking = Some(ask);
        cx.notify();
    }

    fn answer(&mut self, go: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ask) = self.asking.take() else {
            return;
        };
        cx.notify();
        if !go {
            return;
        }
        let job: Job = match ask {
            Ask::Switch => drivers::switch_job(),
            Ask::Firmware => drivers::firmware_job(),
        };
        self.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
    }

    fn refresh(&mut self) {
        drop(
            self.client
                .request(&protocol::snapshot_invalidate(settings::ALL_TOPICS)),
        );
    }
}

impl Render for DriversPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let settings = self.settings.read(cx);
        let jobs = self.jobs.read(cx);
        let busy = jobs.busy();
        let updating = jobs.running() == Some("update-firmware");
        let g = Graphics::of(settings);
        let star = |label: &str, description: &str| self.stars.star(HUB, label, description, cx);
        let copy = |id: String, text: String, enabled: bool| {
            let client = self.client.clone();
            Button::new(SharedString::from(id))
                .label("Copy")
                .variant(ButtonVariant::Outline)
                .disabled(!enabled)
                .on_click(move |_, window, cx| row::copy(&client, &text, window, cx))
        };

        let mut graphics = Vec::new();
        let hardware = self.inventories.read(cx).known("hardware");
        let gpus = match hardware {
            Known::Value(doc) => drivers::gpus(doc),
            Known::Waiting => {
                // A whole row, not a control: a line across the group.
                graphics.push(
                    Skeleton::line()
                        .label("Reading the GPUs")
                        .into_any_element(),
                );
                Vec::new()
            }
            Known::Unknown(why) => {
                graphics.push(
                    Alert::new("drivers-unread-hardware", "The GPUs could not be read")
                        .variant(Tone::Danger)
                        .description(why.to_owned())
                        .into_any_element(),
                );
                Vec::new()
            }
        };
        for (n, gpu) in gpus.iter().enumerate() {
            let summary = drivers::gpu_summary(gpu);
            let label = if gpu.name.is_empty() {
                "GPU"
            } else {
                &gpu.name
            };
            let text = if summary.is_empty() {
                gpu.name.clone()
            } else {
                summary.clone()
            };
            let enabled = !gpu.name.is_empty() || !summary.is_empty();
            graphics.push(
                row::setting(
                    label.to_owned(),
                    summary.clone(),
                    "lspci",
                    copy(format!("copy-gpu-{n}"), text, enabled),
                    star(label, &summary),
                )
                .into_any_element(),
            );
        }
        if matches!(hardware, Known::Value(_)) && !g.any(&gpus) {
            graphics.push(
                row::setting(
                    "Graphics",
                    "No GPUs reported.",
                    "lspci",
                    h_flex(),
                    star("Graphics", "No GPUs reported."),
                )
                .into_any_element(),
            );
        }
        if let Some((description, hint, copied)) = g.stack() {
            graphics.push(
                row::setting(
                    "Active stack",
                    description,
                    hint,
                    copy("copy-stack".into(), copied.into(), true),
                    star("Active stack", description),
                )
                .into_any_element(),
            );
        }
        if g.hybrid {
            let this = this.clone();
            graphics.push(
                row::setting(
                    "Hybrid GPU",
                    g.hybrid_text(),
                    "omarchy toggle hybrid gpu",
                    Button::new("drivers-switch")
                        .label("Switch…")
                        .variant(ButtonVariant::Outline)
                        .disabled(busy)
                        .on_click(move |_, _, cx| {
                            this.update(cx, |page, cx| page.ask(Ask::Switch, cx))
                        }),
                    star("Hybrid GPU", g.hybrid_text()),
                )
                .into_any_element(),
            );
        }

        let firmware_text = if updating {
            "Updating firmware…"
        } else {
            "Install firmware updates through fwupd when the vendor ships them."
        };
        let firmware = [
            row::setting(
                "Firmware",
                firmware_text,
                "omarchy update firmware",
                Button::new("drivers-firmware")
                    .label("Update…")
                    .variant(ButtonVariant::Outline)
                    .disabled(busy)
                    .on_click({
                        let this = this.clone();
                        move |_, _, cx| this.update(cx, |page, cx| page.ask(Ask::Firmware, cx))
                    }),
                star("Firmware", firmware_text),
            )
            .into_any_element(),
            row::setting(
                "Refresh",
                drivers::REFRESH_TEXT,
                "snapshot",
                Button::new("drivers-refresh")
                    .label("Refresh")
                    .variant(ButtonVariant::Outline)
                    .on_click({
                        let this = this.clone();
                        move |_, _, cx| this.update(cx, |page, _| page.refresh())
                    }),
                star(
                    "Refresh",
                    "Read the units again after a firmware or GPU change.",
                ),
            )
            .into_any_element(),
        ];

        let alerts = settings
            .failures(drivers::HELD)
            .into_iter()
            .map(|(held, why)| {
                Alert::new(
                    SharedString::from(format!("drivers-unread-{}", held.key)),
                    held.unread,
                )
                .variant(Tone::Danger)
                .description(why.to_owned())
            });
        content_column()
            .child(Heading::new("Drivers").level(1))
            .child(
                Text::new(
                    "Bound GPU drivers, hybrid switching, and firmware updates through fwupd.",
                )
                .muted(),
            )
            .children(alerts)
            // The QML frames the graphics drivers' group, a list of the GPUs and their drivers.
            .child(row::list_group(
                "Graphics",
                drivers::GRAPHICS_DETAIL,
                graphics,
            ))
            .child(row::group("Firmware", drivers::FIRMWARE_DETAIL, firmware))
            .child(self.confirmation(&g, &this))
    }
}

impl DriversPage {
    fn confirmation(&self, g: &Graphics, this: &Entity<Self>) -> impl IntoElement {
        let (title, text, go) = match self.asking {
            Some(Ask::Switch) | None => ("Switch GPU mode", g.switch_text(), "Switch and reboot"),
            Some(Ask::Firmware) => (
                "Firmware update",
                "Ask fwupd to install available firmware. You may need to reboot afterward.",
                "Update",
            ),
        };
        let (closed, cancelled, confirmed) = (this.clone(), this.clone(), this.clone());
        Dialog::new("drivers-confirm", title)
            .open(self.asking.is_some())
            .description(text)
            .on_close(move |window, cx| {
                closed.update(cx, |page, cx| page.answer(false, window, cx))
            })
            .action(move |_, _| {
                let cancelled = cancelled.clone();
                Button::new("drivers-confirm-cancel")
                    .label("Cancel")
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, window, cx| {
                        cancelled.update(cx, |page, cx| page.answer(false, window, cx))
                    })
            })
            .action(move |_, _| {
                let confirmed = confirmed.clone();
                Button::new("drivers-confirm-go")
                    .label(go)
                    .variant(ButtonVariant::Primary)
                    .on_click(move |_, window, cx| {
                        confirmed.update(cx, |page, cx| page.answer(true, window, cx))
                    })
            })
    }
}
