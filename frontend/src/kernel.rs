//! The Kernel page's logic (`pages/system/KernelPage.qml`): the running kernel as
//! `Diagnostics.kernelSummary` makes it of the diagnostics inventory, the texts of Direct EFI
//! boot, which follow whether it is set up, and the setters its rows call.
//! `tests/catalog_custom.rs` runs the JavaScript under Node against this, and the setters through
//! `tests/catalog_setters.rs`.

use serde_json::Value;

use crate::catalog::control::{Field, When};
use crate::catalog::setter::{ANY, Args, Confirm, Sends, Setter, setter, task};

/// What the row says when the kernel cannot be read.
pub const UNREAD: &str = "Kernel version was not readable.";

/// `cleanLine`: one line, trimmed, at most `max` units, and nothing that names a secret.
pub fn clean_line(value: Option<&Value>, max: usize) -> String {
    let text = match value {
        None | Some(Value::Null) => String::new(),
        Some(v) => crate::js::string(v),
    };
    let mut out = String::new();
    let mut gap = false;
    for c in text.chars() {
        if matches!(c, '\r' | '\n' | '\t') {
            gap = true;
            continue;
        }
        if gap {
            out.push(' ');
            gap = false;
        }
        out.push(c);
    }
    if gap {
        out.push(' ');
    }
    let out = out.trim();
    let lower = out.to_lowercase();
    if out.is_empty()
        || ["password", "passwd", "secret", "token", "bearer "]
            .iter()
            .any(|w| lower.contains(w))
    {
        return String::new();
    }
    let mut units = 0;
    out.chars()
        .take_while(|c| {
            units += c.len_utf16();
            units <= max
        })
        .collect()
}

/// `kernelSummary(normalize(diagnostics))`: the kernel's name, release and machine.
pub fn kernel_summary(diagnostics: Option<&Value>) -> String {
    let kernel = diagnostics
        .and_then(|d| d.get("kernel"))
        .filter(|k| k.is_object());
    let part = |name: &str, max: usize| clean_line(kernel.and_then(|k| k.get(name)), max);
    [
        part("sysname", 32),
        part("release", 64),
        part("machine", 32),
    ]
    .into_iter()
    .filter(|p| !p.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

/// Whether direct boot is set up, and whether this machine can have it.
pub const DIRECT: Field = Field::Key("directBoot");
pub const AVAILABLE: Field = Field::Key("directBootAvailable");

/// The button and confirm while direct boot is not set up, then while it is.
pub const SET_UP: (&str, Confirm) = (
    "Set up…",
    Confirm {
        title: "Set up direct boot",
        message: "Firmware will get an Omarchy entry that loads the UKI in /boot/EFI/Linux. The next start skips Limine and goes to disk unlock. To boot a snapshot after that, open the firmware boot menu, pick Limine, then the snapshot.",
        action: "Set up",
    },
);
pub const REMOVE: (&str, Confirm) = (
    "Remove…",
    Confirm {
        title: "Remove direct boot",
        message: "Delete the Omarchy firmware entry. The next start uses Limine again, so snapshot picking is in the bootloader menu.",
        action: "Remove",
    },
);

/// When the Direct EFI boot button can be used.
pub const DIRECT_ENABLED: When = When::All(&[When::Idle, When::On(AVAILABLE)]);

pub const SETUP_DIRECT_BOOT: Setter = setter(
    "setupDirectBoot",
    Args::Nothing,
    ANY.needs(AVAILABLE),
    Sends::Job(task("direct-boot.setup", "direct-boot", "The direct boot setup").sudo()),
);

/// `setTweak("swappiness", on)`, for the switch and for Reset.
pub const SET_SWAPPINESS: Setter = setter(
    "setTweak",
    Args::Lead("swappiness"),
    ANY.boolean(),
    Sends::Setting("tweaks.swappiness"),
);

/// Whether the lower swappiness tweak is on: `!!(Omarchy.tweaks && Omarchy.tweaks.swappiness)`.
pub fn swappiness(tweaks: Option<&Value>) -> bool {
    crate::js::truthy(
        tweaks
            .filter(|t| t.is_object())
            .and_then(|t| t.get("swappiness")),
    )
}

/// The Lower swappiness row's description.
pub const SWAPPINESS_ON: &str = "vm.swappiness is 10 via /etc/sysctl.d/99-atmos-swappiness.conf.";
pub const SWAPPINESS_OFF: &str = "Write vm.swappiness=10. Reset removes the drop-in.";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_summary_is_what_can_be_read_of_the_kernel() {
        let d = json!({"kernel": {"sysname": "Linux", "release": "6.17.1-arch1-1\n", "machine": "x86_64"}});
        assert_eq!(kernel_summary(Some(&d)), "Linux 6.17.1-arch1-1 x86_64");
        assert_eq!(kernel_summary(Some(&json!({"kernel": "x"}))), "");
        assert_eq!(clean_line(Some(&json!("my token")), 32), "");
    }
}
