//! What the QML has of a catalog row, for `catalog_controls.rs`, `catalog_custom.rs` and `catalog_lists.rs`: the row's block in its page,
//! and the QML text each control and condition stands for.

#![allow(dead_code)]

use atmos_ui::catalog::control::{Act, Options, Range, Shape, When};
use atmos_ui::catalog::setter::{Args, Btn, Sends, Setter};
use atmos_ui::catalog::{Field, Line, Page};
use atmos_ui::js;
use serde_json::json;

use super::common::qml;

/// The `SettingRow` block of the QML page (or the shared row it uses) whose label is `label`:
/// the `n`th with that label.
pub fn qml_row(page: &Page, label: &str, n: usize) -> String {
    let pages = qml::repo().join("pages");
    let mut src = std::fs::read_to_string(pages.join(page.file)).unwrap();
    for entry in std::fs::read_dir(pages.join("rows")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let Some(at) = src.find(&format!("{name} {{")) else {
            continue;
        };
        // A page that sets `requirePresent: true` makes the row's `!requirePresent || x` x.
        let shared = std::fs::read_to_string(&path).unwrap();
        let shared = if block(&src[at..]).contains("requirePresent: true") {
            shared.replace("!requirePresent || ", "")
        } else {
            shared
        };
        src.push_str(&shared);
    }
    let quoted = serde_json::to_string(label).unwrap();
    let mut found = Vec::new();
    for (at, _) in src.match_indices("SettingRow {") {
        let block = block(&src[at..]);
        if block.contains(&format!("label: {quoted}")) {
            found.push(block.to_owned());
        }
    }
    found
        .into_iter()
        .nth(n)
        .unwrap_or_else(|| panic!("{}: no row {label:?} ({n})", page.id))
}

/// The text from the start of `src` to the brace that closes its first block.
pub fn block(src: &str) -> &str {
    let mut depth = 0;
    for (i, c) in src.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[..=i];
                }
            }
            _ => {}
        }
    }
    src
}

/// How many rows before this one on its page have the same label.
pub fn nth(page: &Page, line: &Line) -> usize {
    page.lines()
        .take_while(|l| !std::ptr::eq(*l, line))
        .filter(|l| l.label == line.label)
        .count()
}

/// The QML expression of what a field reads.
pub fn qml_field(f: Field) -> String {
    match f {
        Field::Look(f) => format!("Omarchy.hyprLook.{f}"),
        Field::Input(f) => format!("Omarchy.hyprInput.{f}"),
        Field::Key(k) => format!("Omarchy.{k}"),
        Field::Derived(_, name) => format!("Omarchy.{name}"),
    }
}

/// The QML expression of a condition.
pub fn qml_when(w: When) -> String {
    match w {
        When::Always => "true".into(),
        When::On(f) => qml_field(f),
        When::Off(f) => format!("!{}", qml_field(f)),
        When::Is(f, text) => format!("{} === {}", qml_field(f), json!(text)),
        When::IsNot(f, text) => format!("{} !== {}", qml_field(f), json!(text)),
        When::Flag(key, name) => format!("Omarchy.{key} && Omarchy.{key}.{name} === true"),
        When::All(all) => all
            .iter()
            .map(|w| qml_when(*w))
            .collect::<Vec<_>>()
            .join(" && "),
        When::Has(f) => format!("{}.length > 0", qml_field(f)),
        When::Differs(f, n) => format!("{} !== {n}", qml_field(f)),
        When::Idle => "!Omarchy.jobBusy".into(),
        When::Many(f) => format!("{}.length > 1", qml_field(f)),
        When::Any(any) => format!(
            "({})",
            any.iter()
                .map(|w| qml_when(*w))
                .collect::<Vec<_>>()
                .join(" || ")
        ),
    }
}

/// A choice's options as the QML row has them: each fixed one in order, or a page property
/// built from the field's list and labelled by the table (`profileOptions`, `bandOptions`).
pub fn options_row(at: &str, row: &str, page_src: &str, options: Options) {
    match options {
        Options::Fixed(all) => {
            let mut last = 0;
            for (value, label) in all {
                let item = format!("{{ value: {}, label: {} }}", json!(value), json!(label));
                let found = row[last..].find(&item);
                assert!(found.is_some(), "{at}: {item} in order");
                last += found.unwrap();
            }
        }
        Options::Devices(f) => {
            let options = format!("options: root.deviceOptions({})", qml_field(f));
            assert!(row.contains(&options), "{at}: {options}");
            for line in [
                "function deviceOptions(list) {",
                "if (!item || !item.name) continue",
                "out.push({ value: String(item.name), label: String(item.description || item.name) })",
            ] {
                assert!(page_src.contains(line), "{at}: deviceOptions has {line}");
            }
        }
        Options::Listed(f, labels) if row.contains(&format!("options: {}\n", qml_field(f))) => {
            assert!(labels.is_empty(), "{at}: the QML labels its items itself");
        }
        Options::Listed(f, labels) => {
            assert!(
                row.contains("options: root."),
                "{at}: options from the page"
            );
            assert!(
                page_src.contains(&qml_field(f)),
                "{at}: listed from {}",
                qml_field(f)
            );
            for (value, label) in labels {
                let bare = format!("{value}: {}", json!(label));
                let quoted = format!("{}: {}", json!(value), json!(label));
                assert!(
                    page_src.contains(&bare) || page_src.contains(&quoted),
                    "{at}: the label {quoted}"
                );
            }
        }
    }
}

/// The call that writes a field, as the QML row makes it.
pub fn qml_writer(f: Field) -> String {
    match f {
        Field::Look(f) => format!("writeHyprLook({{ {f}:"),
        Field::Input(f) => format!("writeHyprInput({{ {f}:"),
        Field::Key(k) => format!("Omarchy.set(\"{k}\""),
        Field::Derived(..) => String::new(),
    }
}

/// The JavaScript literal of a write's arguments, as `Omarchy.qml` writes it.
pub fn js_args(act: &Act) -> String {
    if act.args.is_empty() {
        return "{}".into();
    }
    let fields: Vec<_> = act
        .args
        .iter()
        .map(|(k, v)| format!("{k}: {}", json!(v)))
        .collect();
    format!("{{ {} }}", fields.join(", "))
}

/// The number as the QML writes it: `0.3` and `1`, not `0.30` or `1.0`.
pub fn qml_number(n: f64) -> String {
    js::string(&js::json_number(n))
}

/// A button of a row of buttons: its label, its own `enabled:`, and the setter it calls, or
/// the confirm it asks, whose title, message and action are the `PrefsConfirm`'s and which
/// calls the setter.
pub fn buttons_row(at: &str, row: &str, page: &str, omarchy: &str, b: &Btn) {
    let text = format!("text: {}", json!(b.label));
    let button = row
        .split("PrefsButton {")
        .skip(1)
        .find(|part| part.contains(&text))
        .unwrap_or_else(|| panic!("{at}: no button {text}"));
    let button = &button[..button.find("\n        }").unwrap_or(button.len())];
    let enabled: Vec<&str> = button
        .lines()
        .filter_map(|l| l.trim().strip_prefix("enabled: "))
        .collect();
    let want: Vec<String> = match b.when {
        When::Always => Vec::new(),
        w => vec![qml_when(w)],
    };
    assert_eq!(enabled, want, "{at} / {}: enabled", b.label);
    let call = format!("Omarchy.{}()", b.setter.name);
    if let Some(p) = b.prompt {
        // The button opens a dialog with one field, whose texts are these, and which calls the
        // setter.
        let id = button
            .split(".open()")
            .next()
            .and_then(|head| head.rsplit(|c: char| c.is_whitespace() || c == '{').next())
            .unwrap_or_else(|| panic!("{at} / {}: opens a dialog", b.label));
        let block = page
            .split(&format!("id: {id}\n"))
            .nth(1)
            .and_then(|rest| rest.split("\n  }\n").next())
            .unwrap_or_else(|| panic!("{at}: no dialog {id}"));
        for text in [p.title, p.placeholder, p.action, p.intro, p.invalid, p.busy] {
            assert!(
                block.contains(&json!(text).to_string()),
                "{at}: {id} says {text}"
            );
        }
        let setter = format!("Omarchy.{}(", b.setter.name);
        assert!(block.contains(&setter), "{at}: {id} calls {setter}");
        return;
    }
    match b.confirm {
        None => assert!(button.contains(&call), "{at} / {}: calls {call}", b.label),
        Some(c) => {
            let id = button
                .split("onClicked: ")
                .nth(1)
                .and_then(|rest| rest.split(".ask()").next())
                .unwrap_or_else(|| panic!("{at} / {}: asks a confirm", b.label))
                .trim();
            let block = page
                .split(&format!("id: {id}\n"))
                .nth(1)
                .and_then(|rest| rest.split("\n  }").next())
                .unwrap_or_else(|| panic!("{at}: no confirm {id}"));
            for (name, v) in [
                ("title", c.title),
                ("message", c.message),
                ("confirmText", c.action),
            ] {
                let want = format!("{name}: {}", json!(v));
                assert!(block.contains(&want), "{at} / {}: {want}", b.label);
            }
            let on = format!("onConfirmed: {call}");
            assert!(block.contains(&on), "{at} / {}: {on}", b.label);
        }
    }
    if let Sends::Job(t) = b.setter.sends {
        let body = omarchy
            .split(&format!("  function {}(", b.setter.name))
            .nth(1)
            .and_then(|rest| rest.split("\n  }\n").next())
            .unwrap_or_default();
        assert!(
            body.contains(&json!(t.kind).to_string()),
            "{at}: {}",
            t.kind
        );
    }
}

/// The source of a page's QML file.
pub fn page_src(page: &Page) -> String {
    std::fs::read_to_string(qml::repo().join("pages").join(page.file)).unwrap()
}

/// A row that calls a setter: the control, what it shows, the call with what it passes, and
/// the row's own check of the value shown. What the setter then does is `catalog_setters.rs`'s.
pub fn calls_row(at: &str, row: &str, page_src: &str, f: Field, shape: Shape, setter: Setter) {
    let call = format!("Omarchy.{}(", setter.name);
    assert!(row.contains(&call), "{at}: calls {call}");
    if let Args::GivenAnd(g) = setter.args {
        let and = format!(", {})", qml_field(g));
        assert!(row.contains(&and), "{at}: passes {and}");
    }
    if let Args::Nothing = setter.args {
        assert!(row.contains(&format!("{call})")), "{at}: passes nothing");
    }
    let shown = qml_field(f);
    match shape {
        Shape::Toggle => {
            assert!(row.contains("PrefsToggle"), "{at}");
            assert!(
                row.contains(&format!("checked: {shown}")),
                "{at}: shows {shown}"
            );
        }
        Shape::Count(range) => {
            for (name, n) in [("from", range.min), ("to", range.max)] {
                let want = format!("{name}: {}", qml_number(n));
                assert!(row.contains(&want), "{at}: {want}");
            }
            let step = format!("stepSize: {}", qml_number(range.step));
            assert!(
                row.contains(&step) || (range.step == 1.0 && !row.contains("stepSize:")),
                "{at}: {step}"
            );
            let value = match range.unset {
                Some(n) => vec![
                    format!("value: {shown} || {}", qml_number(n)),
                    format!("value: {shown} > 0 ? {shown} : {}", qml_number(n)),
                ],
                None => vec![format!("value: {shown}")],
            };
            assert!(value.iter().any(|v| row.contains(v)), "{at}: {value:?}");
        }
        Shape::Pick(options) => {
            assert!(row.contains("PrefsSelect"), "{at}");
            assert!(row.contains(&format!("value: {shown}")), "{at}");
            options_row(at, row, page_src, options);
        }
        Shape::Button(label) => {
            assert!(row.contains(&format!("text: {}", json!(label))), "{at}");
        }
    }
    let same = format!("!== {shown}");
    assert_eq!(
        row.contains(&same),
        setter.check.row_same,
        "{at}: the row checks {same}"
    );
}

/// A slider: its range and step, the setter it calls, and how it rounds what it writes.
pub fn number_row(at: &str, row: &str, f: Field, range: Range) {
    for (name, n) in [
        ("from", range.min),
        ("to", range.max),
        ("stepSize", range.step),
    ] {
        let want = format!("{name}: {}", qml_number(n));
        assert!(row.contains(&want), "{at}: {want}\n{row}");
    }
    let rounds = match range.places {
        0 => "Math.round(value)".to_owned(),
        p => {
            let scale = 10i64.pow(p as u32);
            format!("Math.round(value * {scale}) / {scale}")
        }
    };
    assert!(row.contains(&rounds), "{at}: {rounds}");
    let writer = match f {
        Field::Key(k) => {
            let mut setter = k.to_owned();
            setter[..1].make_ascii_uppercase();
            format!("Omarchy.set{setter}(")
        }
        other => qml_writer(other),
    };
    assert!(row.contains(&writer), "{at}: {writer}");
}

/// The text of `name:` in a QML block, to the next property, on one line.
pub fn property(block: &str, name: &str) -> String {
    let head = format!("{name}: ");
    let lines: Vec<&str> = block
        .lines()
        .map(str::trim)
        .skip_while(|l| !l.starts_with(&head))
        .collect();
    let mut out = vec![&lines[0][head.len()..]];
    for l in &lines[1..] {
        if l.starts_with('?') || l.starts_with(':') || l.starts_with('+') {
            out.push(l);
        } else {
            break;
        }
    }
    out.join(" ")
}
