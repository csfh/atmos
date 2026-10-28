//! What every settings page draws the same way: a row with its label, what it does, the command
//! it runs and its control, and the stand-in for a control whose key has not been read
//! (`components/SettingRow.qml`).

use odk::prelude::*;

/// One setting: its label, what it does, the command or file it writes, and its control.
pub fn setting(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    hint: impl Into<SharedString>,
    control: impl IntoElement,
) -> impl IntoElement {
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
                .child(Text::new(hint).mono().muted()),
        )
        .child(control)
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
