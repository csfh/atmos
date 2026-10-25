//! `atmos-ui`: the Atmos window on ODK. It keeps one `ratmos serve` for as long as it lives.

use atmos_ui::app::Atmos;
use atmos_ui::client::{Client, ServeProcess};
use odk::prelude::*;

fn main() {
    // The page to open on, as `atmos <page>` sets it for the QML window.
    let page = std::env::var("ATMOS_PAGE").unwrap_or_default();
    odk::ui::run(move |cx| {
        let (client, notices) = Client::start(ServeProcess::from_env());
        let options = WindowOptions {
            app_id: Some("dev.csfh.atmos".into()),
            ..Default::default()
        };
        odk::ui::open_window(cx, options, move |_, cx| {
            cx.new(|cx| Atmos::new(client, notices, &page, cx))
        })
        .unwrap();
    });
}
