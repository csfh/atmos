//! The window uses ODK's components and nothing else. A consumer does not modify a component, draw
//! its own widget or reach for GPUI: what it lacks is added to ODK. This reads `src/` (tests are
//! exempt) and fails on anything that styles an element by hand.

use std::fs;
use std::path::{Path, PathBuf};

/// Whole words and calls that mean raw GPUI or a hand-made colour, size or element.
const FORBIDDEN: &[(&str, &str)] = &[
    ("gpui", "raw GPUI; use the odk prelude"),
    ("px(", "a pixel size; use a Space or an ODK layout helper"),
    ("rems(", "a size; use a Space or an ODK layout helper"),
    ("relative(", "a size; use an ODK layout helper"),
    ("rgb(", "a colour; colours are the theme's"),
    ("rgba(", "a colour; colours are the theme's"),
    ("hsla(", "a colour; colours are the theme's"),
    ("opaque_grey(", "a colour; colours are the theme's"),
    (".bg(", "a hand-set colour; colours are the theme's"),
    (".text_color(", "a hand-set colour; colours are the theme's"),
    (
        ".border_color(",
        "a hand-set colour; colours are the theme's",
    ),
    (".shadow", "a shadow; the design language has none"),
    ("div(", "a bare element; add the layout to ODK"),
    ("canvas(", "custom painting; add a component to ODK"),
    ("unsafe", "unsafe code"),
];

/// Style methods that take a size. A number written into one is a literal size.
const SIZED: &[&str] = &[
    "w",
    "h",
    "size",
    "min_w",
    "min_h",
    "max_w",
    "max_h",
    "p",
    "px",
    "py",
    "pt",
    "pb",
    "pl",
    "pr",
    "m",
    "mx",
    "my",
    "mt",
    "mb",
    "ml",
    "mr",
    "gap",
    "gap_x",
    "gap_y",
    "top",
    "left",
    "right",
    "bottom",
    "text_size",
    "line_height",
    "rounded",
    "border",
    "opacity",
    "flex_basis",
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn code_of(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Whether `needle` is in `line` at the start of a word, so `gpui` does not match `grouping` and
/// `px(` does not match the padding method `.px(`.
fn mentions(line: &str, needle: &str) -> bool {
    let starts_word = needle.starts_with(is_word_char);
    line.match_indices(needle).any(|(at, _)| {
        !starts_word
            || !line[..at]
                .chars()
                .next_back()
                .is_some_and(|c| is_word_char(c) || c == '.')
    })
}

/// A number as the first argument of a sized method: `.gap(8.)`, `.w(220)`, `.p(-4.0)`.
fn literal_size(line: &str) -> Option<&'static str> {
    SIZED.iter().copied().find(|method| {
        let call = format!(".{method}(");
        line.match_indices(&call).any(|(at, _)| {
            let after = line[at + call.len()..].trim_start();
            let after = after.strip_prefix('-').unwrap_or(after);
            after.starts_with(|c: char| c.is_ascii_digit())
        })
    })
}

fn violations(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let code = code_of(line);
        for (needle, why) in FORBIDDEN {
            if mentions(code, needle) {
                found.push(format!("line {}: `{needle}`: {why}", n + 1));
            }
        }
        if let Some(method) = literal_size(code) {
            found.push(format!(
                "line {}: a number in `.{method}(`: use a Space",
                n + 1
            ));
        }
    }
    found
}

#[test]
fn the_window_is_built_from_odk_components_alone() {
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(!files.is_empty(), "no sources found");
    let mut report = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        // A `#[cfg(test)]` module closes a file, so what is below it is test code.
        let text = text.split("#[cfg(test)]").next().unwrap_or("");
        for v in violations(text) {
            report.push(format!("{}: {v}", file.display()));
        }
    }
    assert!(report.is_empty(), "not pure:\n{}", report.join("\n"));
}

#[test]
fn the_check_sees_what_it_forbids() {
    let bad = [
        "use odk::gpui::px;",
        "x.w(px(220.))",
        "x.gap(8.0)",
        "x.p( -4 )",
        "x.bg(rgb(0xffffff))",
        "div().child(a)",
        "x.text_color(c)",
        "let t = unsafe { f() };",
    ];
    for line in bad {
        assert!(!violations(line).is_empty(), "missed: {line}");
    }
    let good = [
        "root().gap(Space::Md).p(Space::Lg).px(Space::Sm)",
        "Text::new(\"grouping\").muted()",
        "let width = 8; x.w(width)",
        "// div(px(1.)) is only a comment",
        "v_flex().child(Heading::new(\"A\").level(1))",
    ];
    for line in good {
        assert!(
            violations(line).is_empty(),
            "false alarm: {line}: {:?}",
            violations(line)
        );
    }
}
