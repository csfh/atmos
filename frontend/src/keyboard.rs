//! Input's Hyprland layouts (`pages/InputPage.qml`, `services/HyprPrefs.js`): the layout and
//! variant lists as typed, whether Set can be used, and the one write that sets the layouts,
//! their variants and Alt+Alt together (`Omarchy.setHyprKbOverride`). That write is guarded: the
//! bar puts the old lists back unless kept (`kbOverridePlan`'s revert). `tests/catalog_input.rs`
//! runs `HyprPrefs.js`, the page's code and the setter under Node against this.

use serde_json::{Value, json};

use crate::hypr::{self, INPUT};
use crate::js;

/// What Input holds of the snapshot's `hyprInput`.
pub const INPUT_KEY: crate::catalog::Field = crate::catalog::Field::Key("hyprInput");
/// Whether the workspace swipe lives outside Atmos's block, so a write leaves it out.
pub const GESTURE_UNMANAGED: crate::catalog::Field =
    crate::catalog::Field::Key("hyprWorkspaceGestureUnmanaged");

fn ids(raw: &str) -> Vec<&str> {
    raw.split(',').map(str::trim).collect()
}

/// `sanitizeLayoutList`: lower-case ids of one to eight letters or digits, or "" for a list
/// with one that is not.
pub fn sanitize_layouts(raw: &str) -> String {
    let text = raw.trim().to_lowercase();
    if text.is_empty() {
        return String::new();
    }
    let out = ids(&text);
    let ok = out.iter().all(|id| {
        (1..=8).contains(&id.len())
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    });
    if ok { out.join(",") } else { String::new() }
}

/// `sanitizeVariantList(raw, count)`: one variant per layout, each empty or of letters, digits,
/// `_` and `-`, or "".
pub fn sanitize_variants(raw: &str, count: usize) -> String {
    let text = raw.trim();
    if text.is_empty() {
        return String::new();
    }
    let out = ids(text);
    if count > 0 && out.len() != count {
        return String::new();
    }
    let ok = out.iter().all(|id| {
        id.is_empty()
            || (id.len() <= 32
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c)))
    });
    if ok { out.join(",") } else { String::new() }
}

fn text(v: Option<&Value>) -> String {
    v.filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

/// `clampInput(raw)`, in its order.
fn clamp_input(raw: &Value) -> Vec<(&'static str, Value)> {
    let layouts = sanitize_layouts(&text(raw.get("kbLayoutOverride")));
    let variants = if layouts.is_empty() {
        String::new()
    } else {
        sanitize_variants(
            &text(raw.get("kbVariantOverride")),
            layouts.split(',').count(),
        )
    };
    let mut out: Vec<(&'static str, Value)> = INPUT
        .iter()
        .filter(|(f, _)| *f != "workspaceGesture")
        .map(|(f, k)| (*f, hypr::clamp(*k, raw, f)))
        .collect();
    out.push(("kbLayoutOverride", Value::from(layouts)));
    out.push(("kbVariantOverride", Value::from(variants)));
    out.push((
        "kbGroupToggle",
        hypr::clamp(hypr::Kind::Bool(false), raw, "kbGroupToggle"),
    ));
    out.push((
        "workspaceGesture",
        hypr::clamp(hypr::Kind::Bool(false), raw, "workspaceGesture"),
    ));
    out
}

/// The window's `hyprInput`: the snapshot's, clamped.
pub fn input(raw: Option<&Value>) -> Value {
    let clamped: serde_json::Map<String, Value> = clamp_input(raw.unwrap_or(&Value::Null))
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    Value::Object(clamped)
}

/// `{"k":v,...}` in the order given, as `JSON.stringify` writes an object built in that order.
fn ordered(fields: &[(&str, Value)]) -> String {
    let parts: Vec<String> = fields
        .iter()
        .map(|(k, v)| format!("{}:{}", Value::from(*k), v))
        .collect();
    format!("{{{}}}", parts.join(","))
}

/// `writeHyprKbOverride(layouts, variants, groupToggle)`: the `hypr.apply` of the input block
/// with the three changed, and the revert the guard holds. `input` is the window's `hyprInput`.
pub fn write(
    input: &Value,
    (layouts, variants, toggle): (&str, &str, bool),
    gesture_unmanaged: bool,
) -> (Value, Value) {
    let mut merged = input.as_object().cloned().unwrap_or_default();
    merged.insert("kbLayoutOverride".into(), Value::from(layouts));
    merged.insert("kbVariantOverride".into(), Value::from(variants));
    merged.insert("kbGroupToggle".into(), Value::Bool(toggle));
    let mut full = clamp_input(&Value::Object(merged));
    let get = |name: &str| {
        full.iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.clone())
    };
    let layout = get("kbLayoutOverride").unwrap_or_default();
    let patch = [
        ("kbLayoutOverride", layout.clone()),
        (
            "kbVariantOverride",
            get("kbVariantOverride").unwrap_or_default(),
        ),
        ("kbGroupToggle", get("kbGroupToggle").unwrap_or_default()),
    ];
    full.push(("kbLayout", layout));
    if gesture_unmanaged {
        full.retain(|(k, _)| *k != "workspaceGesture");
    }
    let payload = format!(
        "{{\"_patch\":{},\"_full\":{}}}",
        ordered(&patch),
        ordered(&full)
    );
    let body = crate::protocol::write(
        "hypr.apply",
        &json!({ "group": "input", "payload": payload }),
    );
    let revert = json!({
        "kind": "kbOverride",
        "layouts": text(input.get("kbLayoutOverride")),
        "variants": text(input.get("kbVariantOverride")),
        "groupToggle": input.get("kbGroupToggle") == Some(&Value::Bool(true)),
    });
    (body, revert)
}

/// `setHyprKbOverride(layouts, variants, groupToggle)`: none for a list that is not valid, or
/// when nothing would change.
pub fn set(
    input: &Value,
    (layouts, variants, toggle): (&str, &str, bool),
    gesture_unmanaged: bool,
) -> Option<(Value, Value)> {
    let parsed = sanitize_layouts(layouts);
    if !layouts.trim().is_empty() && parsed.is_empty() {
        return None;
    }
    let vars = if parsed.is_empty() {
        String::new()
    } else {
        sanitize_variants(variants, parsed.split(',').count())
    };
    if !variants.trim().is_empty() && !parsed.is_empty() && vars.is_empty() {
        return None;
    }
    let same = parsed == text(input.get("kbLayoutOverride"))
        && vars == text(input.get("kbVariantOverride"))
        && toggle == (input.get("kbGroupToggle") == Some(&Value::Bool(true)));
    (!same).then(|| write(input, (&parsed, &vars, toggle), gesture_unmanaged))
}

/// The page's drafts as it reads them (`kbLayoutParsed`, `kbLayoutValid`, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    pub layouts: String,
    pub variants: String,
    pub layouts_ok: bool,
    pub variants_ok: bool,
    /// Whether what is typed differs from what is set.
    pub dirty: bool,
}

impl Draft {
    pub fn of(input: &Value, layouts: &str, variants: &str) -> Self {
        let parsed = sanitize_layouts(layouts);
        let vars = if parsed.is_empty() {
            String::new()
        } else {
            sanitize_variants(variants, parsed.split(',').count())
        };
        Draft {
            layouts_ok: layouts.trim().is_empty() || !parsed.is_empty(),
            variants_ok: variants.trim().is_empty() || !vars.is_empty(),
            dirty: parsed != text(input.get("kbLayoutOverride"))
                || vars != text(input.get("kbVariantOverride")),
            layouts: parsed,
            variants: vars,
        }
    }

    /// `kbOverrideValid`.
    pub fn valid(&self) -> bool {
        self.layouts_ok && self.variants_ok
    }

    /// The Hyprland layouts row's words.
    pub fn layouts_says(&self) -> &'static str {
        if self.layouts_ok {
            "Optional. A comma list such as us,dk for Hyprland only. Leave it blank to keep the System layout."
        } else {
            "Layout ids are 1–8 letters or digits, separated by commas. Spaces around commas are fine. Set stays off until the list is valid."
        }
    }

    /// The Variants row's words.
    pub fn variants_says(&self) -> &'static str {
        if self.variants_ok {
            "Optional variants matching the layout list, such as ,nodeadkeys."
        } else {
            "One variant per layout, same comma count. Letters, digits, underscore, or hyphen. Set stays off until the list matches."
        }
    }

    /// Alt+Alt's write: the drafts while they are valid, else what is set, with the switch
    /// flipped.
    pub fn toggle(&self, input: &Value, gesture_unmanaged: bool) -> Option<(Value, Value)> {
        let flipped = input.get("kbGroupToggle") != Some(&Value::Bool(true));
        let (layouts, variants) = if self.valid() {
            (self.layouts.clone(), self.variants.clone())
        } else {
            (
                text(input.get("kbLayoutOverride")),
                text(input.get("kbVariantOverride")),
            )
        };
        set(input, (&layouts, &variants, flipped), gesture_unmanaged)
    }
}
