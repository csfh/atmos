//! `services/Monitors.js`, which the Displays page writes its rules with: the modes a rule may
//! name, a monitor's rule patched or started from what Hyprland reports, and the rules of each
//! layout. `tests/catalog_display.rs` runs `Monitors.js` against it.

use serde_json::{Value, json};

use crate::display::{current_mode, key, rules_write};
use crate::js;

fn s(v: &Value, k: &str) -> String {
    v.get(k)
        .filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

fn n(v: &Value, k: &str) -> f64 {
    js::number(v.get(k))
}

/// `sanitizeMode`.
pub fn sanitize_mode(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return "preferred".into();
    }
    if matches!(t, "preferred" | "highres" | "highrr") {
        return t.into();
    }
    let (size, rate) = t.split_once('@').map_or((t, None), |(a, b)| (a, Some(b)));
    let ok_size = size.split_once('x').is_some_and(|(w, h)| {
        let d = |p: &str| (3..=5).contains(&p.len()) && p.chars().all(|c| c.is_ascii_digit());
        d(w) && d(h)
    });
    let ok_rate = rate.is_none_or(|r| {
        let (i, f) = r.split_once('.').map_or((r, None), |(a, b)| (a, Some(b)));
        !i.is_empty()
            && i.chars().all(|c| c.is_ascii_digit())
            && f.is_none_or(|f| !f.is_empty() && f.chars().all(|c| c.is_ascii_digit()))
    });
    if ok_size && ok_rate {
        t.into()
    } else {
        String::new()
    }
}

/// `modeFromHyprctl`.
pub fn mode_from_hyprctl(raw: &str) -> String {
    let t = raw.trim();
    let t = t
        .strip_suffix("Hz")
        .or_else(|| t.strip_suffix("hz"))
        .unwrap_or(t);
    sanitize_mode(t)
}

/// `patchRules(rules, live, output, patch)`: the rules with `output`'s changed, starting from
/// what the monitor reports when it has none.
pub fn patch_rules(
    rules: &[Value],
    monitors: &[Value],
    output: &str,
    patch: &Value,
) -> Option<Vec<Value>> {
    if output.is_empty()
        || !output
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        return None;
    }
    let merge = |row: &mut Value| {
        for (k, v) in patch.as_object().into_iter().flatten() {
            row[k] = v.clone();
        }
    };
    let mut found = false;
    let mut next: Vec<Value> = rules
        .iter()
        .map(|r| {
            let mut row = if r.is_object() { r.clone() } else { json!({}) };
            if r.get("output").map(js::string).unwrap_or_default() == output {
                merge(&mut row);
                found = true;
            }
            row
        })
        .collect();
    if !found {
        let seen = monitors.iter().find(|m| s(m, "name") == output);
        let round0 = |v: f64| {
            if v.is_nan() || v == 0.0 {
                0.0
            } else {
                js::round(v)
            }
        };
        let mut row = match seen {
            Some(m) => {
                let mode = match mode_from_hyprctl(&current_mode(m)) {
                    x if x.is_empty() => "preferred".to_owned(),
                    x => x,
                };
                let scale = if js::truthy(m.get("scale")) {
                    js::number(m.get("scale"))
                } else {
                    1.0
                };
                json!({
                    "output": output, "mode": mode, "position": "auto", "scale": js::json_number(scale),
                    "transform": js::json_number(round0(n(m, "transform"))), "disabled": m.get("enabled") == Some(&Value::Bool(false)),
                    "vrr": js::json_number(round0(n(m, "vrr"))), "bitdepth": 8, "cm": "",
                })
            }
            None => json!({
                "output": output, "mode": "preferred", "position": "auto", "scale": 1, "transform": 0,
                "disabled": false, "vrr": 0, "bitdepth": 8, "cm": "",
            }),
        };
        merge(&mut row);
        next.push(row);
    }
    Some(next)
}

/// `layoutFromLive(name, live)`: a rule per output for Desk, Laptop or Docked.
pub fn layout_from_live(name: &str, monitors: &[Value]) -> Vec<Value> {
    monitors
        .iter()
        .filter_map(|m| {
            let output = match s(m, "name") {
                x if x.is_empty() => s(m, "output"),
                x => x,
            };
            let ok = !output.is_empty()
                && output.len() <= 64
                && output
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
            if !ok {
                return None;
            }
            let internal = m.get("internal") == Some(&Value::Bool(true))
                || output.to_lowercase().starts_with("edp");
            let disabled = match name {
                "laptop" => !internal,
                "docked" => internal,
                _ => false,
            };
            Some(normalize_rule(&output, &live_mode(m), m, disabled))
        })
        .collect()
}

fn live_mode(m: &Value) -> String {
    if js::truthy(m.get("mode")) {
        return match sanitize_mode(&s(m, "mode")) {
            x if x.is_empty() => "preferred".into(),
            x => x,
        };
    }
    let (w, h, hz) = (
        js::round(n(m, "width")),
        js::round(n(m, "height")),
        n(m, "refreshRate"),
    );
    if w.is_finite() && h.is_finite() && w >= 640.0 && h >= 480.0 {
        let size = key(w, h);
        let raw = if hz.is_finite() && hz > 0.0 {
            format!("{size}@{}", js::string(&js::json_number(js::round(hz))))
        } else {
            size
        };
        return match sanitize_mode(&raw) {
            x if x.is_empty() => "preferred".into(),
            x => x,
        };
    }
    "preferred".into()
}

/// `applyMonitorLayout(name)`: the rules write and its guard, none for a layout that would
/// leave no display on.
pub fn layout_write(
    name: &str,
    monitors: &[Value],
    (internal, external): (bool, bool),
    current: Option<&Value>,
) -> Option<(Value, Value)> {
    if (name == "laptop" && !internal) || (name == "docked" && !external) {
        return None;
    }
    let items = layout_from_live(name, monitors);
    (!items.is_empty()).then(|| rules_write(current, items))
}

/// `normalizeItem` of a layout's rule.
fn normalize_rule(output: &str, mode: &str, m: &Value, disabled: bool) -> Value {
    let scale = n(m, "scale");
    let scale = if !scale.is_finite() || scale <= 0.0 {
        1.0
    } else {
        scale.min(4.0)
    };
    let clamp = |k: &str, max: f64| {
        let v = js::round(n(m, k));
        js::json_number(if !v.is_finite() || v < 0.0 {
            0.0
        } else {
            v.min(max)
        })
    };
    let cm = s(m, "cm");
    let cm = if ["auto", "srgb", "dcip3", "wide", "hdr", "hdredid"].contains(&cm.as_str()) {
        cm
    } else {
        String::new()
    };
    json!({
        "output": output, "mode": mode, "position": "auto",
        "scale": js::json_number(js::round(scale * 1000.0) / 1000.0),
        "transform": clamp("transform", 7.0), "disabled": disabled, "mirror": "",
        "vrr": clamp("vrr", 3.0), "bitdepth": if js::round(n(m, "bitdepth")) == 10.0 { 10 } else { 8 }, "cm": cm,
    })
}
