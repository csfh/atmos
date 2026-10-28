//! What every settings page draws the same way: a row with its label, what it does, the command
//! it runs, its control and its star, and the stand-in for a control whose key has not been read
//! (`components/SettingRow.qml`).

use odk::prelude::*;
use serde_json::Value;

use crate::client::Client;
use crate::favorites::{self, Item};
use crate::settings::Settings;

/// One setting: its label, what it does, the command or file it writes (none when empty), its
/// control and its star.
pub fn setting(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    hint: impl Into<SharedString>,
    control: impl IntoElement,
    star: impl IntoElement,
) -> impl IntoElement {
    let hint = hint.into();
    h_flex()
        .gap(Space::Md)
        .py(Space::Sm)
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(Space::Xs)
                .child(Text::new(label))
                .child(Text::new(description).muted())
                .children((!hint.is_empty()).then(|| Text::new(hint).mono().muted())),
        )
        .child(control)
        .child(star)
}

/// What a row's star needs: the window's settings, which hold the list, and the way to write it.
#[derive(Clone)]
pub struct Stars {
    pub client: Client,
    pub settings: Entity<Settings>,
}

impl Stars {
    /// The star of the row `label` on `hub` (`toggleStar`). It is lit when the list says the row
    /// is in it, and it waits, unpressable, until the list has been read: a star pressed before
    /// would write a list of one over the file.
    pub fn star(&self, hub: &str, label: &str, description: &str, cx: &App) -> impl IntoElement {
        let list = self.settings.read(cx).value(&favorites::HELD).cloned();
        let on = list
            .as_ref()
            .is_some_and(|l| favorites::is_favorite(&favorites::items(l), hub, label));
        let name = if on {
            format!("Remove from favorites: {label}")
        } else {
            format!("Add to favorites: {label}")
        };
        let id = SharedString::from(format!(
            "star-{}",
            favorites::row_key(hub, label).unwrap_or_default()
        ));
        let stars = self.clone();
        let row = Item::of_row(hub, label, description);
        Button::icon(id, if on { "★" } else { "☆" }, name)
            .variant(ButtonVariant::Ghost)
            .disabled(list.is_none() || row.is_none())
            .on_click(move |_, window, cx| {
                let Some(row) = row.clone() else {
                    return;
                };
                let client = stars.client.clone();
                stars.settings.update(cx, |settings, cx| {
                    let Some(list) = settings.value(&favorites::HELD) else {
                        return;
                    };
                    let next = favorites::toggle(&favorites::items(list), &row, None);
                    let body = favorites::write(&next);
                    settings.send(
                        &client,
                        "favorites",
                        &body,
                        "Favorites were not saved",
                        window,
                        cx,
                    );
                });
            })
    }
}

/// The favorites list as the window holds it, empty until read.
pub fn favorite_items(settings: &Settings) -> Vec<Item> {
    settings
        .value(&favorites::HELD)
        .map_or_else(Vec::new, |list: &Value| favorites::items(list))
}

/// In place of a control, while its key is being read: never a control with a state made up for
/// it (docs/adr/0006).
pub fn reading(label: &str) -> impl IntoElement {
    Skeleton::line().label(format!("Reading {label}"))
}

/// In place of a control whose key could not be read.
pub fn unknown() -> impl IntoElement {
    Text::new("Unknown").muted()
}

/// A group of rows under its title and what it says of them.
pub fn group(
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    rows: impl IntoIterator<Item = impl IntoElement>,
) -> impl IntoElement {
    v_flex()
        .gap(Space::Xs)
        .child(Heading::new(title).level(2))
        .child(Text::new(detail).muted())
        .children(rows)
}
