//! Favorites: the rows starred on any hub, grouped by hub (`pages/FavoritesPage.qml`). The list is
//! the snapshot's `favorites` key (`favorites.rs`); Open… shows the row's page, and the star takes
//! a row out. A list that has not been read draws no rows and says so (docs/adr/0006).

use std::rc::Rc;

use odk::prelude::*;

use crate::client::Client;
use crate::favorites;
use crate::pages::row::{self, Stars};
use crate::settings::{Known, Settings};

/// What Open… does with a hub or hub page id: the shell shows it.
pub type Open = Rc<dyn Fn(&str, &mut App)>;

pub struct FavoritesPage {
    settings: Entity<Settings>,
    stars: Stars,
    open: Open,
}

const DESCRIPTION: &str =
    "Settings you starred. Open jumps to the hub. The star on a row adds or removes it.";

impl FavoritesPage {
    pub fn new(
        client: Client,
        settings: Entity<Settings>,
        open: Open,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&settings, |_, _, cx| cx.notify()).detach();
        let stars = Stars {
            client,
            settings: settings.clone(),
        };
        Self {
            settings,
            stars,
            open,
        }
    }

    /// The groups shown now, each its title and the labels of its rows.
    pub fn groups(&self, cx: &App) -> Vec<(String, Vec<String>)> {
        favorites::groups(&row::favorite_items(self.settings.read(cx)))
            .into_iter()
            .map(|(title, rows)| (title, rows.into_iter().map(|r| r.label.clone()).collect()))
            .collect()
    }
}

impl Render for FavoritesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.settings.read(cx);
        let page = content_column()
            .child(Heading::new("Favorites").level(1))
            .child(Text::new(DESCRIPTION).muted());
        let list = match settings.known(&favorites::HELD) {
            Known::Waiting => {
                return page
                    .child(Skeleton::line().label("Reading your favorites"))
                    .into_any_element();
            }
            Known::Unknown(why) => {
                return page
                    .child(
                        Alert::new("favorites-unread", favorites::HELD.unread)
                            .variant(Tone::Danger)
                            .description(why.to_owned()),
                    )
                    .into_any_element();
            }
            Known::Value(list) => favorites::items(list),
        };
        let groups = favorites::groups(&list);
        if groups.is_empty() {
            return page
                .child(row::group(
                    "Favorites",
                    "Star a setting on any hub. It shows up here.",
                    [Text::new("No favorites yet. Star a setting on any page.").muted()],
                ))
                .into_any_element();
        }
        let failed = settings.failure(&favorites::HELD).map(|why| {
            Alert::new("favorites-unread", favorites::HELD.unread)
                .variant(Tone::Danger)
                .description(why.to_owned())
        });
        let groups = groups.into_iter().map(|(title, rows)| {
            let rows = rows.into_iter().map(|item| {
                let open = self.open.clone();
                let hub = item.hub.clone();
                let button = Button::new(SharedString::from(format!("open-{}", item.id)))
                    .label("Open…")
                    .variant(ButtonVariant::Outline)
                    .accessible_name(format!("Open {}", item.label))
                    .on_click(move |_, _, cx| open(&hub, cx));
                let star = self
                    .stars
                    .star(&item.hub, &item.label, &item.description, cx);
                row::setting(
                    item.label.clone(),
                    item.description.clone(),
                    "",
                    button,
                    star,
                )
            });
            row::group(title, "Starred rows from this hub.", rows)
        });
        page.children(failed).children(groups).into_any_element()
    }
}
