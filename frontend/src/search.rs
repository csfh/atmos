//! Search in the window (docs/adr/0020): the hubs and the rows the ported pages declare, matched
//! as `backend/src/search.rs` matches the QML catalog, a case-insensitive substring of a row's
//! text, with hits ordered by hub and then label. Nothing is asked of the backend.

use crate::hubs;

/// A row search can find: a setting, or a group's heading (`detail` is what a group says of
/// itself).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    /// The hub, or hub page, it is on.
    pub hub: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub hint: &'static str,
    pub detail: &'static str,
    pub keywords: &'static [&'static str],
}

impl Row {
    /// A setting row.
    pub const fn setting(
        hub: &'static str,
        label: &'static str,
        description: &'static str,
        hint: &'static str,
        keywords: &'static [&'static str],
    ) -> Row {
        Row {
            hub,
            label,
            description,
            hint,
            detail: "",
            keywords,
        }
    }

    /// A group's heading.
    pub const fn group(hub: &'static str, title: &'static str, detail: &'static str) -> Row {
        Row {
            hub,
            label: title,
            description: "",
            hint: "",
            detail,
            keywords: &[],
        }
    }

    /// Whether `needle`, already lower case and trimmed, is in the row's text.
    fn matches(&self, needle: &str) -> bool {
        needle.is_empty()
            || [self.label, self.description, self.hint, self.detail]
                .iter()
                .chain(self.keywords)
                .any(|text| text.to_lowercase().contains(needle))
    }
}

/// The rows of the ported pages.
pub fn page_rows() -> Vec<Row> {
    let mut out = crate::defaults::rows();
    out.extend(crate::tweaks::rows());
    out.extend(crate::drivers::ROWS);
    out
}

/// Everything search can find: each hub as a row of its own, then the pages' rows.
pub fn catalog() -> Vec<Row> {
    let mut out: Vec<Row> = hubs::HUBS
        .iter()
        .map(|hub| Row {
            hub: hub.id,
            label: hub.title,
            description: hub.description,
            hint: "",
            detail: "",
            keywords: hub.keywords,
        })
        .collect();
    out.extend(page_rows());
    out
}

/// The rows that hold `query`, ordered by hub and then label (`search.rs`'s `query`).
pub fn query(query: &str) -> Vec<Row> {
    let needle = query.to_lowercase();
    let needle = needle.trim();
    let mut hits: Vec<Row> = catalog()
        .into_iter()
        .filter(|row| row.matches(needle))
        .collect();
    hits.sort_by(|a, b| a.hub.cmp(b.hub).then_with(|| a.label.cmp(b.label)));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_matches_any_part_of_a_row_in_any_case_and_orders_by_hub() {
        let hits = query("  FIREFOX ");
        assert!(
            hits.iter()
                .any(|r| r.hub == "defaults" && r.label == "Browser")
        );
        let hubs: Vec<_> = hits.iter().map(|r| r.hub).collect();
        let mut sorted = hubs.clone();
        sorted.sort();
        assert_eq!(hubs, sorted);
        assert_eq!(query("").len(), catalog().len());
        assert!(query("no row says this").is_empty());
    }
}
