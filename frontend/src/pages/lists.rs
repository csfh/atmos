//! The rows of a group's list (`catalog::LISTS`), as the page module that owns each draws them.

use odk::prelude::*;

use super::catalog::CatalogPage;
use super::custom::reminders;
use crate::catalog::List;
use crate::pages::{agents, capture, environment, hardware, rules, software, startup, workspaces};
use crate::profiles::PROFILES;

/// The rows of a group's list, as its page draws them.
pub fn list_rows(
    list: List,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    match list {
        List::Profiles => PROFILES
            .iter()
            .map(|p| page.profile(p, this, cx).into_any_element())
            .collect(),
        List::Agents => agents::agent_rows(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::McpAgents => agents::mcp_rows(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Autostart => startup::lines(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Failures => startup::failures(page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::WorkspaceNames => workspaces::names(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Scratch => workspaces::scratch(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Recording => vec![capture::recording_row(page, this, cx).into_any_element()],
        List::AppAutostart => crate::pages::launchers::autostart_rows(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Launchers(kind) => crate::pages::launchers::rows(kind, page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Hooks => crate::pages::hooks::groups(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Sys(which) => crate::pages::system::rows(which, page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Plugins => crate::pages::bar::plugins(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Diag(which) => crate::pages::diagnostics::rows(which, page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Disk(which) => match which {
            "drives" => crate::pages::disks::drives(page, this, cx)
                .into_iter()
                .map(|r| r.into_any_element())
                .collect(),
            "luks" => crate::pages::disks::luks(page, this, cx)
                .into_iter()
                .map(|r| r.into_any_element())
                .collect(),
            "snapshots" => crate::pages::disks::snapshots(page, this, cx)
                .into_iter()
                .map(|r| r.into_any_element())
                .collect(),
            _ => crate::pages::disks::swap(page, cx)
                .into_iter()
                .map(|r| r.into_any_element())
                .collect(),
        },
        List::Meters(which) => crate::pages::meters::list(which, page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Live("quick") => crate::pages::live::quick(page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Live("hero") => vec![crate::pages::live::hero(page, cx).into_any_element()],
        List::Live("gpus") => crate::pages::live::gpus(page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Live("procs") => crate::pages::live::procs(true, page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Live("hottest") => crate::pages::live::procs(false, page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Live("sampling") => crate::pages::live::sampling(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Live("now") => crate::pages::live::now(page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Live("cores") => crate::pages::live::cores(page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Live(_) => crate::pages::live::cards(page)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Acct("face") => vec![crate::pages::accounts::face(page, this, cx).into_any_element()],
        List::Acct("users") => crate::pages::accounts::users(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Acct("groups") => crate::pages::accounts::groups(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Acct(_) => Vec::new(),
        List::Wifi("saved") => crate::pages::wifi::saved(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Wifi(_) => crate::pages::radios::networks(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Bt("paired") => crate::pages::bluetooth::paired(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Bt(_) => crate::pages::radios::devices(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Export => crate::pages::export::sections(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Link => vec![crate::pages::network::link(page, cx).into_any_element()],
        List::Wallpaper => vec![crate::pages::appearance::wallpaper(page, cx).into_any_element()],
        List::Bindings("overrides") => crate::pages::bindings::overrides(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Bindings(_) => crate::pages::bindings::bound(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Display("monitors") => crate::pages::display::monitors(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Display(_) => crate::pages::display::empty(page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Held => crate::pages::history::held(page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Changes => crate::pages::history::changes(page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Units => crate::pages::units::rows(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Software(group) => software::items(group, page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Hardware(section) => hardware::rows(section, page, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Rules => rules::rows(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::EnvVars => environment::vars(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
        List::Reminders => reminders(page, this, cx)
            .into_iter()
            .map(|r| r.into_any_element())
            .collect(),
    }
}
