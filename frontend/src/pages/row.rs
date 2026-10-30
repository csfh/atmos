//! What every settings page draws the same way: a row with its label, what it does, the command
//! it runs, its control and its star, and the stand-in for a control whose key has not been read
//! (`components/SettingRow.qml`).

use odk::prelude::*;
use serde_json::Value;

use crate::actions;
use crate::client::Client;
use crate::favorites;
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
    item(label, description, hint).action(control).action(star)
}

/// One setting with a muted caption under its words (`SettingRow.caption`): a count or a status.
pub fn captioned(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    caption: impl Into<SharedString>,
    hint: impl Into<SharedString>,
    control: impl IntoElement,
    star: impl IntoElement,
) -> impl IntoElement {
    let caption = caption.into();
    item(label, description, hint)
        .children((!caption.is_empty()).then(|| Text::new(caption).muted()))
        .action(control)
        .action(star)
}

/// A setting whose control takes the row's width, under its words, as a QML slider row does
/// (`stretchControl`): a number or a line of text.
pub fn stacked(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    hint: impl Into<SharedString>,
    control: impl IntoElement,
    star: impl IntoElement,
) -> impl IntoElement {
    item(label, description, hint).action(star).footer(control)
}

/// A row's label, what it does and the command it runs, as ODK's `Item`: a description or a hint
/// that is empty (one the QML builds at run time) takes no line.
fn item(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    hint: impl Into<SharedString>,
) -> Item {
    let (description, hint) = (description.into(), hint.into());
    let row = Item::new(label).children((!hint.is_empty()).then(|| Text::new(hint).mono().muted()));
    if description.is_empty() {
        row
    } else {
        row.description(description)
    }
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
    pub fn star(
        &self,
        hub: &str,
        label: &str,
        description: &str,
        cx: &App,
    ) -> impl IntoElement + use<> {
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
        let row = favorites::Item::of_row(hub, label, description);
        Button::icon(id, if on { Icon::StarFilled } else { Icon::Star }, name)
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

/// Copies `text` to the clipboard through the daemon (`clipboard.copy`), and says so when it could
/// not. Text that cannot be copied (`actions::copy_write`) sends nothing.
pub fn copy(client: &Client, text: &str, window: &mut Window, cx: &mut App) {
    let Some(body) = actions::copy_write(text) else {
        return;
    };
    let reply = client.request(&body);
    window
        .spawn(cx, async move |cx| {
            if let Err(failure) = reply.await {
                let _ = cx.update(|window, cx| {
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

/// The favorites list as the window holds it, empty until read.
pub fn favorite_items(settings: &Settings) -> Vec<favorites::Item> {
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

/// A group of rows under its title and what it says of them, as ODK's `ItemGroup`: a line
/// between one row and the next. What it says takes no line when it is empty.
pub fn group(
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    rows: impl IntoIterator<Item = impl IntoElement>,
) -> ItemGroup {
    let (group, detail) = (ItemGroup::new(title), detail.into());
    let group = if detail.is_empty() {
        group
    } else {
        group.description(detail)
    };
    group.children(rows)
}

/// A catalog page's group: a list group when the QML frames it (`catalog::FRAMED`).
pub fn catalog_group(
    page: &str,
    title: &'static str,
    detail: impl Into<SharedString>,
    rows: impl IntoIterator<Item = impl IntoElement>,
) -> ItemGroup {
    if crate::catalog::framed(page, title) {
        list_group(title, detail, rows)
    } else {
        group(title, detail, rows)
    }
}

/// List groups, one box under another the page's gap apart, as one row of the group they are
/// listed in, whose rows sit closer; nothing when there are none.
pub fn stack<E: IntoElement>(groups: Vec<E>) -> Option<impl IntoElement + use<E>> {
    (!groups.is_empty()).then(|| v_flex().gap(Space::Xl).children(groups))
}

/// A group that lists things (a disk, a monitor, the hooks of one kind, paired devices), drawn in
/// a box with its title inside: an outline `ItemGroup`.
pub fn list_group(
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    rows: impl IntoIterator<Item = impl IntoElement>,
) -> ItemGroup {
    group(title, detail, rows).variant(ItemGroupVariant::Outline)
}
