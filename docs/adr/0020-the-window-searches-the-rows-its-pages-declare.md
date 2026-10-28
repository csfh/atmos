# 0020. The window searches the rows its pages declare

Status: Accepted. Date: 2026-10-27.

## Context

Search is `search.query`: the QML window sends the hub list (`Hubs.searchContext`), and `backend/src/search.rs` builds its catalog by reading the QML page sources from disk, taking the label, description, hint, detail and keywords of every `SettingRow`, `CollectionRow`, `PrefsRow`, `PrefsLink` and `PrefsGroup` it finds, and matches a query as a case-insensitive substring of a row's text, ordered by hub and then label. Nothing else reads that catalog: the MCP server does not use it.

The Rust window replaces the QML one page by page (ADR 0013), and ADR 0013 removes the QML pages once the last hub is ported. A catalog read from QML sources would then be empty. The Rust pages already hold what search needs as data: each page's logic module lists its rows with their labels, descriptions and hints (`defaults::PICKERS`, `tweaks::CATALOG`), and `hubs.rs` holds each hub's title and description pinned to `Hubs.js`. What the first ports dropped is the keywords, which the QML rows carry only for search.

## Decision

The Rust window searches the rows its pages declare, in its own process.

- **A searchable row is data in its page's logic module**: the hub (or hub page) it is on, its label, description, hint and keywords, as `SettingRow` has them. A row whose label is only known at run time (a GPU's name) is not searchable, as it is not in the QML catalog, which reads literals.
- **The catalog is the hubs and those rows**, gathered in one place in `frontend/`; `hubs.rs` carries each hub's keywords too, pinned to `Hubs.js` like its title and description.
- **A query matches as `search.rs` matches**: case-insensitive substring over a row's text, hits ordered by hub and then label. The window asks the backend nothing to search.
- **A port keeps the keywords.** `frontend/tests/search_parity.rs` asks the real `atmosd` for the QML catalog and checks that every row it has on a ported hub, and every hub, is in the window's catalog with the same label, description, hint and keywords (a group by its title and detail), so a port that drops one fails. The window may find more: a row whose label the QML builds from a model, such as a tweak, is not in the QML catalog and is in the window's.
- `search.query` and `backend/src/search.rs` stay while the QML window does, and go with it in the record that removes it (ADR 0013).

## Consequences

- Search in the Rust window covers the ported pages and every hub; until the last port, a row of a hub that is not ported is found only in the QML window.
- A search costs no request and no process (P5's "a read costs a process" no longer applies to it), and works while the daemon is down.
- Rows are restated in Rust for as long as the QML page exists, held to it by the parity tests; after the QML window goes, the Rust rows are the only copy.
- The hubs' keywords are copied once more, from `Hubs.js` into `hubs.rs`.

## Alternatives considered

- **Keep `search.query` and point `search.rs` at the Rust sources.** It would parse Rust source text, as it parses QML now, for data the window already holds.
- **A catalog file both windows read.** It would be a third copy of every label, kept in step with two windows for the few months both exist.
- **ODK `Command`'s fuzzy filter as the match.** It changes what a query finds; the substring match is what users of the QML window know. Which control draws the results is a separate choice.

## Enforced by

- `frontend/tests/search_parity.rs`
- `frontend/tests/pins.rs`
- `frontend/src/search.rs`
