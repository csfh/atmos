//! The Drivers hub's logic, as the QML page and `services/Hardware.js` have it: the GPUs the
//! `hardware` inventory lists and how each is summed up (`normalize`, `gpuSummary`), what the
//! active graphics stack and the hybrid switch say, and the two jobs the page runs.
//! `tests/drivers_parity.rs` runs the JavaScript under Node against this.

use serde_json::{Value, json};

use crate::jobs::Job;
use crate::settings::{Held, Settings};

const HOST: &str = "settings.host";

const fn held(key: &'static str, unread: &'static str) -> Held {
    Held {
        key,
        topic: HOST,
        unread,
    }
}

pub const HELD: &[Held] = &[
    held(
        "hwNvidia",
        "Whether an NVIDIA GPU is present could not be read",
    ),
    held("hwNvidiaGsp", "The NVIDIA firmware could not be read"),
    held(
        "hwNvidiaWithoutGsp",
        "The NVIDIA firmware could not be read",
    ),
    held("hwVulkan", "Whether Vulkan is present could not be read"),
    held(
        "hybridGpuAvailable",
        "Whether this machine can switch GPUs could not be read",
    ),
    held("hybridGpuMode", "The GPU mode could not be read"),
];

/// A text field as `cleanText` keeps it: one line, trimmed, nothing for the placeholders
/// firmware leaves (`Not Specified`, `To be filled by O.E.M.`, all zeros), and at most `max`
/// UTF-16 units.
pub fn clean_text(value: Option<&Value>, max: usize) -> String {
    let raw = match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => i.to_string(),
            (None, Some(f)) if f.fract() == 0.0 && f.abs() < 1e21 => format!("{f:.0}"),
            _ => n.to_string(),
        },
        Some(other) => other.to_string(),
    };
    let mut one = String::new();
    let mut run = false;
    for c in raw.chars() {
        if matches!(c, '\r' | '\n' | '\t') {
            if !run {
                one.push(' ');
            }
            run = true;
        } else {
            one.push(c);
            run = false;
        }
    }
    let s = one.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    if s.is_empty() {
        return String::new();
    }
    let lower = s.to_lowercase();
    let placeholder = [
        "not specified",
        "unknown",
        "none",
        "to be filled by o.e.m.",
        "default string",
        "fill by oem",
        "system product name",
        "system manufacturer",
        "oem",
        "1234567890",
    ]
    .contains(&lower.as_str())
        || s.chars().all(|c| c == '0')
        || s.chars().all(|c| c == 'f' || c == 'F');
    if placeholder {
        return String::new();
    }
    let units: Vec<u16> = s.encode_utf16().collect();
    if units.len() > max {
        String::from_utf16_lossy(&units[..max])
    } else {
        s.to_owned()
    }
}

/// One GPU as `normalizeGpu` keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gpu {
    pub name: String,
    pub vendor: String,
    pub pci_id: String,
    pub driver: String,
}

/// The GPUs of the `hardware` inventory that have a name or a PCI id.
pub fn gpus(hardware: &Value) -> Vec<Gpu> {
    hardware
        .get("gpus")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|row| {
            let row = row.as_object();
            let field = |key: &str, max| clean_text(row.and_then(|r| r.get(key)), max);
            Gpu {
                name: field("name", 160),
                vendor: field("vendor", 80),
                pci_id: field("pciId", 20),
                driver: field("driver", 40),
            }
        })
        .filter(|g| !g.name.is_empty() || !g.pci_id.is_empty())
        .collect()
}

/// `joinParts`: the parts that are not empty once cleaned, joined by `sep`.
fn join(parts: &[&str], sep: &str) -> String {
    parts
        .iter()
        .map(|p| clean_text(Some(&Value::String((*p).to_owned())), 200))
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(sep)
}

/// A GPU in a sentence: vendor and name, its driver and its PCI id (`gpuSummary`).
pub fn gpu_summary(gpu: &Gpu) -> String {
    let who = join(&[&gpu.vendor, &gpu.name], " ");
    let driver = if gpu.driver.is_empty() {
        String::new()
    } else {
        format!("Driver {}", gpu.driver)
    };
    let text = join(&[&who, &driver, &gpu.pci_id], ". ");
    if text.is_empty() || text.ends_with(['.', '!', '?']) {
        text
    } else {
        format!("{text}.")
    }
}

/// What the page shows of a key: whether it is `true`, as QML's `Omarchy.x === true` reads it
/// with the property's `false` before anything is read.
fn flag(settings: &Settings, key: &str) -> bool {
    HELD.iter()
        .find(|h| h.key == key)
        .and_then(|h| settings.value(h))
        .is_some_and(|v| v == &Value::Bool(true))
}

fn text<'a>(settings: &'a Settings, key: &str) -> &'a str {
    HELD.iter()
        .find(|h| h.key == key)
        .and_then(|h| settings.value(h))
        .and_then(Value::as_str)
        .unwrap_or_default()
}

/// What the page knows of the graphics stack and the hybrid switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graphics {
    pub nvidia: bool,
    pub gsp: bool,
    pub without_gsp: bool,
    pub vulkan: bool,
    pub hybrid: bool,
    pub mode: String,
}

impl Graphics {
    pub fn of(settings: &Settings) -> Graphics {
        Graphics {
            nvidia: flag(settings, "hwNvidia"),
            gsp: flag(settings, "hwNvidiaGsp"),
            without_gsp: flag(settings, "hwNvidiaWithoutGsp"),
            vulkan: flag(settings, "hwVulkan"),
            hybrid: flag(settings, "hybridGpuAvailable"),
            mode: text(settings, "hybridGpuMode").to_owned(),
        }
    }

    /// Whether there is anything to say of graphics (`hasGraphics`).
    pub fn any(&self, gpus: &[Gpu]) -> bool {
        !gpus.is_empty() || self.nvidia || self.vulkan || self.hybrid
    }

    /// The Active stack row, when there is one: what it says, its hint, and what Copy copies.
    pub fn stack(&self) -> Option<(&'static str, &'static str, &'static str)> {
        if self.nvidia {
            Some(if self.gsp {
                (
                    "NVIDIA, with GSP firmware (Turing or newer).",
                    "omarchy hw nvidia",
                    "NVIDIA GSP",
                )
            } else if self.without_gsp {
                (
                    "NVIDIA, without GSP firmware (Maxwell, Pascal, or Volta).",
                    "omarchy hw nvidia",
                    "NVIDIA without GSP",
                )
            } else {
                ("NVIDIA.", "omarchy hw nvidia", "NVIDIA")
            })
        } else if self.vulkan {
            Some(("Vulkan.", "omarchy hw vulkan", "Vulkan"))
        } else {
            None
        }
    }

    /// What the Hybrid GPU row says of the mode.
    pub fn hybrid_text(&self) -> &'static str {
        match self.mode.as_str() {
            "Integrated" => {
                "Using the integrated GPU only. Switch to hybrid if you want the dedicated GPU."
            }
            "Hybrid" => "Hybrid mode. The dedicated GPU can wake for a game or CUDA.",
            _ => "This machine can switch between integrated-only and hybrid.",
        }
    }

    /// What the switch's confirmation says it will do.
    pub fn switch_text(&self) -> &'static str {
        if self.mode == "Integrated" {
            "Turn the dedicated GPU on (hybrid) and reboot."
        } else {
            "Use only the integrated GPU and reboot."
        }
    }
}

/// `omarchy toggle hybrid gpu`, which reboots (`toggleHybridGpu`).
pub fn switch_job() -> Job {
    Job::new(
        "hybrid-gpu.toggle",
        json!({}),
        "hybrid-gpu",
        "The GPU switch",
    )
    .sudo()
}

/// `omarchy update firmware` through fwupd (`updateFirmware`).
pub fn firmware_job() -> Job {
    Job::new(
        "update.firmware",
        json!({}),
        "update-firmware",
        "The firmware update",
    )
    .sudo()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gpu_keeps_what_firmware_did_not_leave_blank() {
        let list = gpus(&json!({"gpus": [
            {"name": "  RTX 4070\n", "vendor": "NVIDIA", "pciId": "10de:2786", "driver": "nvidia"},
            {"name": "Not Specified", "pciId": "0000"},
            {"vendor": "AMD"},
            "junk",
        ]}));
        assert_eq!(list.len(), 1);
        assert_eq!(
            gpu_summary(&list[0]),
            "NVIDIA RTX 4070. Driver nvidia. 10de:2786."
        );
        assert_eq!(clean_text(Some(&json!("a\r\n\tb")), 10), "a b");
        assert_eq!(clean_text(Some(&json!("FFFF")), 10), "");
        assert_eq!(clean_text(Some(&json!("abcdef")), 3), "abc");
    }

    #[test]
    fn the_stack_is_nvidia_before_vulkan() {
        let g = Graphics {
            nvidia: true,
            gsp: false,
            without_gsp: true,
            vulkan: true,
            hybrid: false,
            mode: String::new(),
        };
        assert_eq!(g.stack().unwrap().2, "NVIDIA without GSP");
        assert!(g.any(&[]));
    }
}
