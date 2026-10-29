//! The pages drawn from the catalog on GPUI's test platform (`common/window.rs`): a hub's pages
//! as tabs, a launcher word that names a hub page, the star every row keeps, the topics a page
//! holds, and its mapped rows: a control that waits for its key, then writes what the QML row
//! writes, a typed write, a link to another page, and a row shown only while its condition holds.

mod common;

use atmos_ui::catalog;
use common::Reply;
use common::window::{Win, open, ordinary};
use odk::gpui::{KeyUpEvent, Keystroke, TestAppContext, px, size};
use serde_json::{Value, json};

fn page(w: &mut Win) -> String {
    w.atmos.read_with(w.cx, |a, _| a.page().to_owned())
}

fn shown(w: &mut Win) -> &'static str {
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    catalog.read_with(w.cx, |c, _| c.page().id)
}

/// A daemon that keeps the favorites list it is given and reads it back.
fn keeps_the_list(request: &Value) -> Reply {
    if request["name"] == "favorites.write" {
        let doc: Value = serde_json::from_str(request["args"]["json"].as_str().unwrap()).unwrap();
        return Reply::Ok(json!({"snapshot": {"favorites": doc["items"]}}));
    }
    ordinary(request)
}

#[test]
fn a_hub_with_pages_shows_them_as_tabs_and_a_tab_opens_its_page() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "monitor", ordinary);
    w.until("the tabs", |w| w.drawn("catalog-tabs/tab-0"));
    assert_eq!((page(&mut w), shown(&mut w)), ("monitor".into(), "monitor"));
    // Monitor's own page, then Processes, then CPU.
    w.click("catalog-tabs/tab-2");
    assert_eq!(
        (page(&mut w), shown(&mut w)),
        ("monitor/cpu".into(), "monitor/cpu")
    );
    assert!(w.drawn("catalog-tabs/tab-6"), "Sensors, the last");
    w.click("catalog-tabs/tab-0");
    assert_eq!(page(&mut w), "monitor");
}

#[test]
fn a_launcher_word_opens_the_hub_page_it_names() {
    // One window at a time: each holds the lock the next one waits for.
    {
        let mut app = TestAppContext::single();
        let mut w = open(&mut app, "system/kernel", ordinary);
        assert_eq!(shown(&mut w), "system/kernel");
    }
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "defaults/agents", ordinary);
    assert_eq!(page(&mut w), "agentic/mcp");
}

/// A hub without pages has no tabs, and one that is whole says nothing is missing: every page is,
/// so none draws the notice.
#[test]
fn a_hub_without_pages_has_no_tabs_and_a_whole_one_no_notice() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "keybindings", ordinary);
    w.until("the page", |w| shown(w) == "keybindings");
    w.settle();
    assert!(!w.drawn("catalog-tabs/tab-0"));
    assert!(!w.drawn("catalog-pending/alert"));
    assert!(catalog::pages().all(|p| p.mapped()), "every page is whole");
}

#[test]
fn the_page_holds_only_the_favorites_topic_for_its_stars() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "profiles", ordinary);
    w.until("the first hold", |w| !w.daemon.of("sub.set").is_empty());
    assert_eq!(
        w.daemon.of("sub.set")[0]["topics"],
        json!({"settings.host": {"intervalMs": 0}})
    );
}

#[test]
fn every_row_has_its_star_and_a_star_writes_the_list_with_that_row() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "monitor/disk", keeps_the_list);
    let disk = catalog::page("monitor/disk").unwrap();
    w.sample("settings.host", 1, json!({"favorites": []}));
    w.settle();
    for line in disk.lines() {
        let id = atmos_ui::favorites::row_key(disk.id, line.label).unwrap();
        assert!(w.drawn(&format!("star-{id}/button")), "{}", line.label);
    }
    w.click("star-monitor/disk/i-o-stall/button");
    w.until("the write", |w| !w.daemon.of("favorites.write").is_empty());
    let sent = &w.daemon.of("favorites.write")[0];
    let doc: Value = serde_json::from_str(sent["args"]["json"].as_str().unwrap()).unwrap();
    assert_eq!(doc["items"][0]["id"], "monitor/disk/i-o-stall");
    assert_eq!(doc["items"][0]["hubTitle"], "Disk I/O");
}

fn press(w: &mut Win, key: &str) {
    let keystroke = Keystroke::parse(key).unwrap();
    w.cx.update(|window, cx| window.dispatch_keystroke(keystroke.clone(), cx));
    w.cx.simulate_event(KeyUpEvent { keystroke });
    w.cx.run_until_parked();
}

#[test]
fn a_mapped_page_holds_the_topics_of_its_keys_and_the_favorites_one() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "windows", ordinary);
    w.until("the first hold", |w| !w.daemon.of("sub.set").is_empty());
    assert_eq!(
        w.daemon.of("sub.set")[0]["topics"],
        json!({"settings.look": {"intervalMs": 0}, "settings.host": {"intervalMs": 0}})
    );
}

#[test]
fn a_switch_waits_for_its_key_then_writes_the_other_value() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "accessibility", ordinary);
    w.settle();
    assert!(
        !w.drawn("ctl-accessibility/animations/track"),
        "not read yet"
    );
    w.sample(
        "settings.look",
        1,
        json!({"hyprLook": {"animations": true}}),
    );
    w.until("the switch", |w| {
        w.drawn("ctl-accessibility/animations/track")
    });
    w.click("ctl-accessibility/animations/track");
    w.until("the write", |w| !w.daemon.of("settings.set").is_empty());
    let mut set = w.daemon.of("settings.set")[0].clone();
    set.as_object_mut().unwrap().remove("id");
    assert_eq!(
        set,
        json!({"op": "settings.set", "domain": "hyprLook.animations", "value": false,
            "readback": ["hyprLook", "hyprLookManaged"]})
    );
}

#[test]
fn a_number_is_written_when_it_is_committed_and_rounded_as_the_slider_rounds() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "input", ordinary);
    w.sample("settings.look", 1, json!({"hyprInput": {"sensitivity": 0}}));
    w.until("the field", |w| w.drawn("ctl-input/sensitivity/box"));
    w.click("ctl-input/sensitivity/box");
    press(&mut w, "ctrl-a");
    w.cx.simulate_input("0.256");
    w.settle();
    assert!(
        w.daemon.of("settings.set").is_empty(),
        "typing writes nothing"
    );
    press(&mut w, "enter");
    w.until("the write", |w| !w.daemon.of("settings.set").is_empty());
    let set = &w.daemon.of("settings.set")[0];
    assert_eq!(
        (&set["domain"], &set["value"]),
        (&json!("hyprInput.sensitivity"), &json!(0.26))
    );
}

#[test]
fn a_typed_write_waits_for_its_condition_and_a_link_opens_its_page() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "windows", ordinary);
    // Tall enough that Reset, under the look's number fields, is in view.
    w.cx.simulate_resize(size(px(1400.0), px(4000.0)));
    let reset = "ctl-windows/reset-look-hypr.reset/button";
    w.sample(
        "settings.look",
        1,
        json!({"hyprLook": {}, "hyprLookManaged": false}),
    );
    w.until("the button", |w| w.drawn(reset));
    w.click(reset);
    w.settle();
    assert!(w.daemon.of("hypr.reset").is_empty(), "nothing to reset");

    w.sample("settings.look", 2, json!({"hyprLookManaged": true}));
    w.settle();
    w.click(reset);
    w.until("the reset", |w| !w.daemon.of("hypr.reset").is_empty());
    assert_eq!(
        w.daemon.of("hypr.reset")[0]["args"],
        json!({"group": "look"})
    );

    w.click("ctl-windows/keybindings/button");
    assert_eq!(page(&mut w), "keybindings");
}

#[test]
fn a_row_is_shown_only_while_its_condition_holds() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "windows", ordinary);
    w.sample(
        "settings.look",
        1,
        json!({"hyprLook": {"layout": "dwindle"}}),
    );
    w.until("the tiling picker", |w| {
        w.drawn("ctl-windows/tiling/trigger")
    });
    assert!(
        !w.drawn("ctl-windows/column-width/box"),
        "dwindle has no columns"
    );
    w.sample(
        "settings.look",
        2,
        json!({"hyprLook": {"layout": "scrolling"}}),
    );
    w.until("the column width", |w| {
        w.drawn("ctl-windows/column-width/box")
    });
}

#[test]
fn a_button_with_a_confirm_asks_first_and_then_starts_its_job() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "system", ordinary);
    // Tall enough that the maintenance rows are in view.
    w.cx.simulate_resize(size(px(1400.0), px(9000.0)));
    w.sample("settings.host", 1, json!({"favorites": []}));
    let restore = "ctl-system/restore-hyprland-0/button";
    w.until("the button", |w| w.drawn(restore));
    w.click(restore);
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    assert_eq!(
        catalog.read_with(w.cx, |c, _| c.asking()),
        Some("Restore Hyprland defaults")
    );
    w.until("the confirm", |w| w.drawn("catalog-confirm-cancel/button"));
    w.click("catalog-confirm-cancel/button");
    w.settle();
    assert!(w.daemon.of("job.start").is_empty(), "cancelled");

    w.click(restore);
    w.until("the confirm", |w| w.drawn("catalog-confirm-go/button"));
    let last = std::cell::Cell::new(None);
    w.until("the dialog still", |w| {
        let now = w.bounds("catalog-confirm-go/button");
        last.replace(now) == now
    });
    w.click("catalog-confirm-go/button");
    w.until("the job", |w| !w.daemon.of("job.start").is_empty());
    let start = &w.daemon.of("job.start")[0];
    assert_eq!(
        (&start["name"], &start["label"]),
        (&json!("refresh.hyprland"), &json!("refresh-hyprland"))
    );
}

#[test]
fn add_a_rule_opens_its_dialog_checks_what_it_holds_and_writes_the_rule() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "windows/rules", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(2000.0)));
    w.sample(
        "settings.host",
        1,
        json!({"favorites": [], "focusedClass": "foot",
            "windowRules": [{"match": "steam", "placement": "float", "managed": false}]}),
    );
    w.until("the user's rule", |w| w.drawn("rule-0-remove/button"));
    w.click("ctl-windows/rules/add-a-rule/button");
    w.until("the dialog", |w| w.drawn("rule-go/button"));
    let last = std::cell::Cell::new(None);
    w.until("the dialog still", |w| {
        let now = w.bounds("rule-go/button");
        last.replace(now) == now
    });
    // Leave, no center, no size and no workspace is nothing to add.
    w.click("rule-focused/button");
    w.click("rule-placement/radio-0");
    w.click("rule-center/track");
    w.settle();
    w.click("rule-go/button");
    w.until("the error", |w| w.drawn("rule-error/alert"));
    assert!(w.daemon.of("settings.set").is_empty(), "nothing written");

    w.click("rule-workspace/box");
    w.cx.simulate_input("3");
    w.click("rule-go/button");
    w.until("the write", |w| !w.daemon.of("settings.set").is_empty());
    let set = &w.daemon.of("settings.set")[0];
    assert_eq!(
        (&set["domain"], &set["value"], &set["readback"]),
        (
            &json!("windowRules"),
            &json!([
                {"match": "steam", "placement": "float", "managed": false},
                {"match": "foot", "title": "", "placement": "", "center": false, "width": 0,
                    "height": 0, "workspace": "3", "pin": false, "fullscreen": false,
                    "opacity": "", "managed": true},
            ]),
            &json!(["windowRules", "windowRulesManaged"])
        )
    );
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    assert!(!catalog.read_with(w.cx, |c, _| atmos_ui::pages::rules::adding(c)));
}

#[test]
fn preview_holds_a_write_until_apply_sends_it() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "system/history", ordinary);
    let preview = "ctl-system/history/preview-changes/track";
    w.until("Preview", |w| w.drawn(preview));
    w.click(preview);
    w.settle();
    w.atmos.update(w.cx, |a, cx| a.show("accessibility", cx));
    w.sample(
        "settings.look",
        1,
        json!({"hyprLook": {"animations": true}}),
    );
    w.until("the switch", |w| {
        w.drawn("ctl-accessibility/animations/track")
    });
    w.click("ctl-accessibility/animations/track");
    w.settle();
    assert!(w.daemon.of("settings.set").is_empty(), "held, not written");

    w.atmos.update(w.cx, |a, cx| a.show("system/history", cx));
    let apply = "ctl-system/history/held-apply/button";
    w.until("Apply", |w| w.drawn(apply));
    w.click(apply);
    w.until("the write", |w| !w.daemon.of("settings.set").is_empty());
    let set = &w.daemon.of("settings.set")[0];
    assert_eq!(
        (&set["domain"], &set["value"]),
        (&json!("hyprLook.animations"), &json!(false))
    );
    // Apply sent it once.
    w.settle();
    assert_eq!(w.daemon.of("settings.set").len(), 1);
}

#[test]
fn a_guarded_write_opens_the_bar_and_revert_writes_the_old_value_back() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "system", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(4000.0)));
    w.sample(
        "settings.system",
        1,
        json!({"keyboardLayout": "us", "keyboardLayouts": ["us", "de", "fr"]}),
    );
    let layout = "ctl-system/layout/trigger";
    w.until("the layout picker", |w| w.drawn(layout));
    w.settle();
    let last = std::cell::Cell::new(None);
    w.until("the page still", |w| {
        let now = w.bounds(layout);
        last.replace(now) == now
    });
    w.click(layout);
    w.until("the layouts", |w| w.drawn("ctl-system/layout/row-1"));
    w.click("ctl-system/layout/row-1");
    w.until("the write", |w| !w.daemon.of("settings.set").is_empty());
    assert_eq!(w.daemon.of("settings.set")[0]["value"], json!("de"));
    w.until("the bar", |w| w.drawn("guard-revert/button"));
    w.click("guard-revert/button");
    w.until("the revert", |w| w.daemon.of("settings.set").len() == 2);
    let back = &w.daemon.of("settings.set")[1];
    assert_eq!(
        (&back["domain"], &back["value"]),
        (&json!("keyboardLayout"), &json!("us"))
    );
}

/// Displays draws each monitor as a group of its own and hides the group that says there is
/// none; disabling one writes its rule behind the guard's bar, and Revert writes the rules back.
#[test]
fn displays_draws_each_monitor_and_guards_a_rule() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "display", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(6000.0)));
    w.sample(
        "settings.look",
        1,
        json!({"monitors": [
            {"name": "eDP-1", "width": 2880, "height": 1920, "refreshRate": 120.0, "scale": 2, "focused": true, "enabled": true},
            {"name": "DP-1", "width": 3840, "height": 2160, "refreshRate": 60.0, "scale": 1, "enabled": true},
        ], "touchpadPresent": true, "touchpadEnabled": true, "internalPresent": true}),
    );
    w.sample("settings.system", 1, json!({"monitorRules": []}));
    let disable = "monitor-1-disable/track";
    w.until("both monitors", |w| w.drawn(disable));
    assert!(!w.drawn("monitors-refresh"), "the empty row hides");
    w.settle();
    w.click(disable);
    w.until("the rule", |w| !w.daemon.of("settings.set").is_empty());
    let set = &w.daemon.of("settings.set")[0];
    assert_eq!(set["domain"], json!("monitorRules"));
    assert_eq!(
        (&set["value"][0]["output"], &set["value"][0]["disabled"]),
        (&json!("DP-1"), &json!(true))
    );
    w.until("the bar", |w| w.drawn("guard-revert/button"));
    w.click("guard-revert/button");
    w.until("the revert", |w| w.daemon.of("settings.set").len() == 2);
    let back = &w.daemon.of("settings.set")[1];
    assert_eq!(
        (&back["domain"], &back["value"]),
        (&json!("monitorRules"), &json!([]))
    );
}

/// Override… on a bound chord opens Add with that chord; a command for a chord that is taken
/// writes the override with the default unbound first.
#[test]
fn override_a_bound_chord_writes_it_with_the_default_unbound() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "keybindings", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(3000.0)));
    w.sample(
        "settings.host",
        1,
        json!({"favorites": [], "bindings": [],
            "keybindings": [{"keys": "SUPER + Q", "action": "Close window"}]}),
    );
    w.until("the bound chord", |w| w.drawn("bound-0-override/button"));
    w.settle();
    w.click("bound-0-override/button");
    w.until("the dialog", |w| w.drawn("bind-go/button"));
    let last = std::cell::Cell::new(None);
    w.until("the dialog still", |w| {
        let now = w.bounds("bind-go/button");
        last.replace(now) == now
    });
    assert!(w.drawn("bind-record/button"), "Record is drawn");
    w.click("bind-command/box");
    w.cx.simulate_input("kitty");
    w.click("bind-go/button");
    w.until("the write", |w| !w.daemon.of("settings.set").is_empty());
    let set = &w.daemon.of("settings.set")[0];
    assert_eq!(
        (&set["domain"], &set["value"]),
        (
            &json!("bindings"),
            &json!([{"keys": "SUPER + Q", "label": "", "command": "kitty", "unbind": true, "managed": true}])
        )
    );
}

/// Accounts lists each user and group; this session's login has no Remove…, and Manage… opens
/// the group's members, this session locked in wheel.
#[test]
fn accounts_lists_users_and_groups_and_manage_opens_the_members() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "accounts", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(3000.0)));
    w.sample(
        "settings.accounts",
        1,
        json!({"currentUser": "me", "avatarPath": "", "fullName": "Me",
            "users": [{"name": "me", "uid": 1000, "wheel": true, "current": true},
                      {"name": "ana", "uid": 1001, "wheel": false, "current": false}],
            "groups": [{"name": "wheel", "gid": 998, "members": ["me"]}]}),
    );
    w.until("the users", |w| w.drawn("user-1-remove/button"));
    assert!(!w.drawn("user-0-remove/button"), "this session stays");
    assert!(!w.drawn("group-0-remove/button"), "wheel stays");
    w.settle();
    w.click("group-0-manage/button");
    w.until("the members", |w| w.drawn("member-1/track"));
    assert!(w.drawn("member-0/track"));
    w.click("acct-done/button");
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    w.until("the dialog closes", |w| {
        catalog.read_with(w.cx, |p, _| {
            atmos_ui::pages::accounts::open_dialog(p).is_empty()
        })
    });
}

/// The Monitor hub holds the live feed, and its interval chips hold it again at the interval
/// picked; Home holds it too and draws what it brings: a process of this user's with End, which
/// asks first.
#[test]
fn home_and_the_monitor_hub_draw_the_live_feed_and_pick_its_interval() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "monitor", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(6000.0)));
    let held = |w: &common::window::Win, ms: u64| {
        w.daemon
            .of("sub.set")
            .iter()
            .any(|s| s["topics"]["live"]["intervalMs"] == json!(ms))
    };
    w.until("the feed held", |w| held(w, 2000));
    w.until("the chips", |w| w.drawn("interval-1000/button"));
    w.settle();
    w.click("interval-1000/button");
    w.until("the feed held again", |w| held(w, 1000));
    w.atmos.update(w.cx, |a, cx| a.show("home", cx));
    for (at, ticks) in [(1000u64, 100u64), (3000, 300)] {
        w.daemon.say(json!({
            "event": "sample", "kind": "live", "sampledAt": at, "tookMs": 1,
            "result": {"uid": 1000, "cpuIdle": at, "cpuTotal": at * 2, "clkTck": 100,
                "processes": [{"pid": 4242, "comm": "busy", "uid": 1000, "ticks": ticks, "mine": true}]},
        }));
    }
    w.until("the process", |w| w.drawn("proc-0-end/button"));
    w.settle();
    w.click("proc-0-end/button");
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    w.until("the confirm", |w| {
        catalog.read_with(w.cx, |p, _| p.asked()) == Some("End this process".into())
    });
}

/// The Memory page's treemap: a press focuses the process under it, a press on the focus goes
/// back out to its parent, and so does a press on the breadcrumb above the rectangles.
#[test]
fn a_press_on_the_treemap_focuses_a_process_and_goes_back_out() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "monitor/memory", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(9000.0)));
    let procs = json!([
        {"pid": 1, "ppid": 0, "comm": "systemd", "rssKb": 10, "uid": 0},
        {"pid": 900, "ppid": 1, "comm": "Hyprland", "rssKb": 1000, "uid": 1000},
        {"pid": 950, "ppid": 900, "comm": "kitty", "rssKb": 3000, "uid": 1000},
    ]);
    for at in [1000u64, 3000] {
        w.daemon.say(json!({
            "event": "sample", "kind": "live", "sampledAt": at, "tookMs": 1,
            "result": {"uid": 1000, "cpuIdle": at, "cpuTotal": at * 2, "clkTck": 100,
                "memTotal": 16000000, "processes": procs},
        }));
    }
    let chart = "rss-treemap/chart";
    w.until("the treemap", |w| w.drawn(chart));
    w.settle();
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    let focus = |w: &mut Win| {
        catalog.read_with(w.cx, |p, _| atmos_ui::pages::memory_plots::treemap_focus(p))
    };
    w.click(chart);
    assert_eq!(focus(&mut w), "950", "the largest leaf is in the middle");
    w.settle();
    w.click(chart);
    assert_eq!(focus(&mut w), "900", "a press on the focus goes out");
    w.settle();
    let corner = w.bounds(chart).unwrap().origin + odk::gpui::point(px(2.0), px(2.0));
    w.cx.simulate_click(corner, odk::gpui::Modifiers::default());
    w.cx.run_until_parked();
    assert_eq!(focus(&mut w), "1", "a press on the breadcrumb goes out");
}
