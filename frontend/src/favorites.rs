//! Starred setting rows, as `services/Favorites.js` has them: a row's id from its hub and label,
//! the list in the snapshot's `favorites` key, what a star writes, and the Favorites hub's groups.
//! `tests/favorites_parity.rs` runs the JavaScript under Node against this.
//!
//! The list is the snapshot's: a star writes the whole list through `favorites.write`, and the
//! window shows what the backend reads back, never the list it sent (docs/adr/0005).

use serde_json::{Map, Value, json};

use crate::hubs;
use crate::protocol;
use crate::settings::Held;

/// The snapshot key the list is read from; any page with stars holds it.
pub const HELD: Held = Held {
    key: "favorites",
    topic: "settings.host",
    unread: "Your favorites could not be read",
};

/// A starred row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// `<hub>/<slug of the label>`.
    pub id: String,
    pub hub: String,
    pub hub_title: String,
    pub label: String,
    pub description: String,
}

impl Item {
    /// What a star on a row of `hub` makes of it, `hubTitle` and all; `None` for a row that
    /// cannot be starred.
    pub fn of_row(hub: &str, label: &str, description: &str) -> Option<Item> {
        normalize(&json!({
            "hub": hub, "hubTitle": hubs::title(hub), "label": label, "description": description,
        }))
    }

    fn to_json(&self) -> Value {
        json!({
            "id": self.id, "hub": self.hub, "hubTitle": self.hub_title,
            "label": self.label, "description": self.description,
        })
    }
}

/// A label as an id part: lower case, every run of anything else one dash, none at either end
/// (`slug`).
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in text.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.push(c);
        } else {
            dash = true;
        }
    }
    out
}

/// A row's id, or nothing for a hub that is not a hub path or a label with nothing to slug
/// (`rowKey`).
pub fn row_key(hub: &str, label: &str) -> Option<String> {
    let hub = hub.trim_matches('/');
    let slug = slug(label);
    if hub.is_empty() || slug.is_empty() || !is_hub_path(hub) {
        return None;
    }
    Some(format!("{hub}/{slug}"))
}

/// `^[a-z0-9]+(?:\/[a-z0-9-]+)*$`
fn is_hub_path(hub: &str) -> bool {
    let mut parts = hub.split('/');
    let first = parts.next().unwrap_or_default();
    let lower = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    !first.is_empty()
        && first.chars().all(lower)
        && parts.all(|p| !p.is_empty() && p.chars().all(|c| lower(c) || c == '-'))
}

/// A field as JavaScript's `String(row.x || "")` reads it, for the values a favorites file holds.
fn text(row: &Map<String, Value>, key: &str) -> String {
    match row.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(true)) => "true".into(),
        Some(Value::Number(n)) if n.as_f64() != Some(0.0) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => i.to_string(),
            (None, Some(f)) if f.fract() == 0.0 && f.abs() < 1e21 => format!("{f:.0}"),
            _ => n.to_string(),
        },
        _ => String::new(),
    }
}

/// One row of the file as the list holds it, or nothing for one that is not a row
/// (`normalizeItem`).
fn normalize(row: &Value) -> Option<Item> {
    let row = row.as_object()?;
    let hub = text(row, "hub");
    let label = text(row, "label");
    let id = row_key(&hub, &label)?;
    Some(Item {
        id,
        hub,
        hub_title: text(row, "hubTitle"),
        label,
        description: text(row, "description"),
    })
}

/// The snapshot's list, each row once (`parseDocument`): the backend sends an array, and a
/// document with `items` is read the same way.
pub fn items(value: &Value) -> Vec<Item> {
    let rows = match value {
        Value::Array(rows) => rows.as_slice(),
        Value::Object(doc) => doc
            .get("items")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice),
        _ => &[],
    };
    let mut out: Vec<Item> = Vec::new();
    for item in rows.iter().filter_map(normalize) {
        if !out.iter().any(|seen| seen.id == item.id) {
            out.push(item);
        }
    }
    out
}

pub fn is_favorite(items: &[Item], hub: &str, label: &str) -> bool {
    row_key(hub, label).is_some_and(|id| items.iter().any(|item| item.id == id))
}

/// The list with `row` starred or not: `want` says which, and `None` flips it (`toggleItem`).
pub fn toggle(items: &[Item], row: &Item, want: Option<bool>) -> Vec<Item> {
    let mut list = items.to_vec();
    let found = list.iter().position(|item| item.id == row.id);
    match (want.unwrap_or(found.is_none()), found) {
        (true, None) => list.push(row.clone()),
        (true, Some(at)) => list[at] = row.clone(),
        (false, Some(at)) => {
            list.remove(at);
        }
        (false, None) => {}
    }
    list
}

/// The file's text for `items` (`serialize`).
pub fn serialize(items: &[Item]) -> String {
    json!({"items": items.iter().map(Item::to_json).collect::<Vec<_>>()}).to_string()
}

/// The write that makes `items` the list.
pub fn write(items: &[Item]) -> Value {
    protocol::write("favorites.write", &json!({"json": serialize(items)}))
}

/// The Favorites hub's groups: one per hub in the order its first row appears, titled by the
/// title the row was starred with, of the rows whose hub the window knows (`groups`).
pub fn groups(items: &[Item]) -> Vec<(String, Vec<&Item>)> {
    let mut out: Vec<(String, String, Vec<&Item>)> = Vec::new();
    for item in items.iter().filter(|item| hubs::known(&item.hub)) {
        match out.iter_mut().find(|(hub, _, _)| *hub == item.hub) {
            Some((_, _, rows)) => rows.push(item),
            None => {
                let title = if item.hub_title.is_empty() {
                    &item.hub
                } else {
                    &item.hub_title
                };
                out.push((item.hub.clone(), title.clone(), vec![item]));
            }
        }
    }
    out.into_iter()
        .map(|(_, title, rows)| (title, rows))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_is_its_hub_and_the_slug_of_its_label() {
        assert_eq!(
            slug("  Disable mouse-acceleration! "),
            "disable-mouse-acceleration"
        );
        assert_eq!(slug("Wi‑Fi / 5 GHz"), "wi-fi-5-ghz");
        assert_eq!(
            row_key("/system/kernel/", "Direct boot").as_deref(),
            Some("system/kernel/direct-boot")
        );
        assert_eq!(row_key("System", "x"), None);
        assert_eq!(row_key("system/", "!!"), None);
        assert_eq!(row_key("a//b", "x"), None);
    }

    #[test]
    fn a_star_adds_and_a_second_takes_away() {
        let row = Item::of_row("defaults", "Browser", "Links").unwrap();
        assert_eq!(row.hub_title, "Defaults");
        let on = toggle(&[], &row, None);
        assert!(is_favorite(&on, "defaults", "browser"));
        assert!(toggle(&on, &row, None).is_empty());
        assert_eq!(toggle(&on, &row, Some(true)).len(), 1);
    }

    #[test]
    fn the_list_is_read_once_per_row_and_grouped_by_known_hub() {
        let list = items(&json!([
            {"hub": "defaults", "hubTitle": "Defaults", "label": "Browser"},
            {"hub": "defaults", "label": "browser"},
            {"hub": "nothing", "label": "X"},
            {"hub": "system/kernel", "label": "Direct boot"},
            "junk",
        ]));
        assert_eq!(list.len(), 3);
        let g = groups(&list);
        assert_eq!(
            g.iter()
                .map(|(t, r)| (t.as_str(), r.len()))
                .collect::<Vec<_>>(),
            [("Defaults", 1), ("system/kernel", 1)]
        );
        assert_eq!(
            items(&json!({"items": [{"hub": "home", "label": "A"}]})).len(),
            1
        );
    }
}
