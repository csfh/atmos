//! The desktop's file chooser for the catalog pages (ODK's `FileChooser`, ADR 0068 there): the
//! path chosen goes back to the page, in its window, and on to `atmosd`, which reads it; the
//! window never reads it itself. When no dialog can be shown, a toast says why.

use odk::prelude::*;

use super::catalog::CatalogPage;

/// What a page does with the path chosen.
pub type Chosen = Box<dyn FnOnce(&mut CatalogPage, String, &mut Window, &mut Context<CatalogPage>)>;

/// Shows `chooser`, then hands the first path chosen to `go`; a cancel does nothing.
pub fn choose(
    chooser: FileChooser,
    this: &Entity<CatalogPage>,
    window: &mut Window,
    cx: &mut App,
    go: Chosen,
) {
    let handle = window.window_handle();
    let this = this.clone();
    chooser.choose(cx, move |chosen, cx| {
        let _ = handle.update(cx, |_, window, cx| match chosen {
            Ok(Some(paths)) => {
                if let Some(path) = paths.first() {
                    let path = path.to_string_lossy().into_owned();
                    this.update(cx, |page, cx| go(page, path, window, cx));
                }
            }
            Ok(None) => {}
            Err(why) => toast(
                window,
                cx,
                Toast::new("The file chooser could not open")
                    .description(why.to_string())
                    .variant(Tone::Danger),
            ),
        });
    });
}
