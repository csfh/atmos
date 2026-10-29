//! The Machine pages' groups and rows, as the QML pages have them in order. Search reads
//! the same words (`tests/catalog_parity.rs` holds them to `search.query`).

use super::control::Control::{
    Acts, Buttons, Calls, Custom, Go, Picks, Press, Says, Shows, Switch,
};
use super::control::Field::Key;
use super::control::Options::Listed;
use super::control::Shape::{Count, Pick, Toggle};
use super::control::When::{All, Flag, Has, Idle, Off, On};
use super::control::{act, range};
use super::custom::Custom::{Bt, Disk, Hw, IdleLock, Logo, Net, Speed, Wifi};
use super::setter::Args::{Given, GivenAnd, Nothing};
use super::setter::Sends::{Job, Setting, Write};
use super::setter::{ANY, btn, setter, task};
use super::{Group, Page, line};

#[rustfmt::skip]
pub const PAGES: &[Page] = &[
    Page {
        id: "hardware",
        file: "HardwarePage.qml",
        description: "What this machine is made of. Processor, memory, chipset, firmware, NPU, and the rest of the units the kernel can see. Graphics drivers are on Drivers.",
        groups: &[
            Group { title: "Machine", detail: "Name and chassis from DMI. Refresh reads the machine again, including memory use.", rows: &[
                line("Identity", "", "/sys/class/dmi/id/product_serial", &["serial", "sku", "service tag"]).with(Custom(Hw("identity"))),
                line("Refresh", "Read the units again. Memory use and temperatures change while the machine runs.", "snapshot", &["reload", "rescan", "inventory"]).with(Custom(Hw("refresh"))),
            ] },
            Group { title: "Motherboard", detail: "The board DMI names. Chipset is the host bridge on that board, listed next.", rows: &[
            ] },
            Group { title: "Chipset", detail: "The PCI host bridge, and the ISA or LPC bridge when the kernel names one. That is the chipset the CPU talks to.", rows: &[
            ] },
            Group { title: "Firmware", detail: "BIOS or UEFI from DMI, plus TPM and Secure Boot when the firmware exposes them.", rows: &[
                line("BIOS", "", "/sys/class/dmi/id/bios_version", &["bios", "uefi", "firmware", "efi"]).with(Custom(Hw("bios"))),
                line("Secure Boot", "", "/sys/firmware/efi", &["secure boot", "efi", "mok"]).with(Custom(Hw("secureBoot"))),
                line("TPM", "", "/sys/class/tpm", &["tpm", "trusted platform"]).with(Custom(Hw("tpm"))),
            ] },
            Group { title: "Processor", detail: "Cores and threads from the kernel. Flags are the ones that matter for guests and SIMD.", rows: &[
            ] },
            Group { title: "Memory", detail: "Fitted memory from the inventory. Live use is on Monitor. Swap is on Disks. Modules are SMBIOS type 17 when the firmware table is readable without root.", rows: &[
                line("Installed", "How much memory is fitted.", "/proc/meminfo", &["ram", "memory", "dimm", "ddr"]).with(Custom(Hw("installed"))),
            ] },
            Group { title: "Graphics", detail: "GPUs, the bound DRM driver, and hybrid switching live on Drivers.", rows: &[
                line("Drivers", "GPUs, the bound DRM driver, and hybrid switching.", "atmos drivers", &["gpu", "graphics", "nvidia", "vulkan", "hybrid", "drm", "drivers"]).with(Go("Open…", "drivers")),
            ] },
            Group { title: "NPU", detail: "A neural processor on PCI, such as AMD XDNA, when one is present.", rows: &[
            ] },
            Group { title: "Network adapters", detail: "Physical interfaces the kernel registered. Virtual bridges and containers stay off this list.", rows: &[
            ] },
            Group { title: "Audio", detail: "Sound cards from ALSA. Volume and sinks stay on the Sound page.", rows: &[
            ] },
            Group { title: "USB", detail: "Devices on the USB buses that published a product name.", rows: &[
            ] },
            Group { title: "Battery", detail: "Charge from sysfs. Profiles and the bar percentage stay on the Power page.", rows: &[
            ] },
            Group { title: "Thermal", detail: "Zones the kernel exported. Refresh if you want a newer reading.", rows: &[
            ] },
            Group { title: "Virtualization", detail: "Whether this OS is a guest, and whether KVM can run guests here.", rows: &[
            ] },
        ],
    },
    Page {
        id: "disks",
        file: "DisksPage.qml",
        description: "Space on each drive, plus encryption if the disk is locked. Snapper snapshots and hibernation are further down.",
        groups: &[
            Group { title: "", detail: "", rows: &[
                line("Drive", "", "omarchy drive info", &["nvme", "ssd", "sata", "lsblk", "model"]).with(Custom(Disk("listed"))),
                line("Speed test", "", "omarchy disk speedtest", &["benchmark", "mbps", "performance"]).with(Custom(Disk("listed"))),
            ] },
            Group { title: "Encryption", detail: "LUKS volumes on this machine. Change the passphrase here. You will need the current one.", rows: &[
            ] },
            Group { title: "Snapshots", detail: "Each listed snapshot can become the new Snapper default. Rollback asks first. Reboot after it finishes.", rows: &[
                line("Snapper", "", "omarchy snapshot", &["btrfs", "limine", "restore", "rollback"]).with(Custom(Disk("snapper"))),
                line("Create snapshot", "", "omarchy snapshot create", &["snapper", "create"]).with(Buttons(&[btn("Create…", setter("createSnapshot", Nothing, ANY, Job(task("snapshot.create", "snapshot-create", "The snapshot").sudo()))).when(All(&[Idle, On(Key("snapperPresent"))])).confirm("Create snapshot", "Take a numbered Snapper snapshot of each config, then run cleanup. You can roll back to it later.", "Create")])).shown(On(Key("snapperPresent"))).words(super::words::Words::Snapshot),
                line("Snapshots", "No snapshots.", "", &["empty", "snapshot"]).with(Custom(Disk("nosnapshots"))),
            ] },
            Group { title: "Hibernation", detail: "Hibernation writes RAM to a swap file and resumes from it on the next boot. Set up needs sudo.", rows: &[
                line("Hibernation", "", "omarchy hibernation available", &["sleep", "swap", "resume"]).with(Custom(Disk("hibernation"))),
            ] },
            Group { title: "Advanced", detail: "How many Snapper snapshots to keep, whether hourly snapshots run, and weekly SSD TRIM.", rows: &[
                line("Keep snapshots", "How many numbered Snapper snapshots the root config keeps before cleanup.", "/etc/snapper/configs/root · NUMBER_LIMIT", &["snapper", "retention", "limit", "number"]).with(Custom(Disk("limit"))),
                line("Timeline snapshots", "Hourly snapshots, cleaned on a timer. That uses disk quickly.", "/etc/snapper/configs/root · TIMELINE_CREATE", &["snapper", "timeline", "hourly", "timer"]).with(Custom(Disk("timeline"))),
                line("Weekly TRIM", "The SSD reclaims unused blocks once a week.", "systemctl enable fstrim.timer", &["fstrim", "trim", "ssd", "discard"]).with(Custom(Disk("fstrim"))),
            ] },
            Group { title: "Swap", detail: "Swap devices this machine is using, including zram when the kernel has it loaded.", rows: &[
            ] },
        ],
    },
    Page {
        id: "network",
        file: "NetworkPage.qml",
        description: "What you are connected to, plus DNS. Nearby networks and a speed test open their own pages.",
        groups: &[
            Group { title: "Connection", detail: "What the machine is using to reach the internet right now. Ethernet shows the interface and link speed. Wi-Fi shows the network name and signal.", rows: &[
                line("Address", "", "omarchy network status", &["ip", "ipv4", "address", "copy"]).with(Custom(Net("address"))),
                line("Gateway", "", "nmcli IP4.GATEWAY", &["gateway", "route", "copy"]).with(Custom(Net("gateway"))),
                line("DNS", "", "nmcli IP4.DNS", &["dns", "resolver", "copy"]).with(Custom(Net("dns"))),
            ] },
            Group { title: "Connectivity", detail: "Wi-Fi radio sits here. Nearby networks and the speed test open their own pages.", rows: &[
                line("Wi-Fi", "", "nmcli radio wifi", &["wlan", "ssid", "scan", "join", "psk", "qr", "rfkill"]).with(Custom(Net("wifi"))),
                line("Bluetooth", "", "omarchy bluetooth power", &["bt", "pair", "headset", "scan", "forget", "radio"]).with(Go("Open…", "bluetooth")).words(super::words::Words::Bluetooth),
                line("Speed test", "A short download, then an upload, on whatever you are connected to now.", "omarchy network speedtest", &["bandwidth", "ping", "speedtest"]).with(Go("Test…", "network/speedtest")),
            ] },
            Group { title: "VPN and hotspot", detail: "WireGuard import uses nmcli. Hotspot needs a Wi-Fi adapter that can AP.", rows: &[
                line("WireGuard", "", "nmcli connection import type wireguard", &["vpn", "wireguard", "import"]).with(Custom(Net("wireguard"))),
                line("Hotspot", "", "nmcli device wifi hotspot", &["hotspot", "ap", "tether"]).with(Custom(Net("hotspot"))),
            ] },
            Group { title: "DNS", detail: "This writes NetworkManager and systemd-resolved so lookups go through the same resolvers. Cloudflare is 1.1.1.1 and 1.0.0.1. Google is 8.8.8.8 and 8.8.4.4. DHCP keeps whatever the link already handed out. Custom uses the servers field below. A VPN or a per-connection DNS setting in NetworkManager can still win for that connection.", rows: &[
                line("DNS provider", "Who answers name lookups for this machine.", "omarchy dns", &["cloudflare", "google", "dhcp", "resolver"]).with(Custom(Net("provider"))),
                line("Custom servers", "", "omarchy dns Custom", &["custom", "nameserver", "resolver"]).with(Custom(Net("servers"))),
            ] },
            Group { title: "Advanced", detail: "Tailscale is a mesh VPN Omarchy can install. LocalSend and Taildrop are actions, not extra config.", rows: &[
                line("Tailscale", "", "omarchy install service tailscale", &["tailscale", "vpn", "tailnet", "mesh"]).with(Buttons(&[btn("Install…", setter("installTailscale", Nothing, ANY, Job(task("tailscale.install", "tailscale-install", "The Tailscale install").sudo()))).when(All(&[Idle, Off(Key("tailscaleInstalled"))])).confirm("Install Tailscale", "Install Tailscale and start the daemon. You will still need to log in with tailscale up.", "Install"), btn("Remove…", setter("removeTailscale", Nothing, ANY.needs(Key("tailscaleInstalled")), Job(task("tailscale.remove", "tailscale-remove", "The Tailscale removal").sudo()))).when(All(&[Idle, On(Key("tailscaleInstalled"))])).confirm("Remove Tailscale", "Remove Tailscale and its bar plugin.", "Remove")])).words(super::words::Words::Tailscale),
                line("LocalSend clipboard", "Send whatever is on the clipboard to a nearby machine.", "omarchy share clipboard", &["localsend", "share", "clipboard"]).with(Acts(&[act("Share clipboard", "share.clipboard", &[], "shareClipboard")])).shown(Flag("extras", "localsend")).enabled(Flag("extras", "localsend")),
                line("LocalSend file", "Pick a file and send it with LocalSend.", "omarchy share file", &["localsend", "share", "file"]).with(Custom(Net("lsfile"))),
                line("LocalSend folder", "Pick a folder and send it with LocalSend.", "omarchy share folder", &["localsend", "share", "folder"]).with(Custom(Net("lsfolder"))),
                line("Taildrop send", "Send a file to a machine on your tailnet. Type the short name, then pick a file.", "omarchy tailscale send", &["taildrop", "tailscale", "send", "share"]).with(Custom(Net("taildrop"))),
                line("Taildrop receive", "Wait for one incoming file and drop it in Downloads.", "omarchy tailscale receive --once", &["taildrop", "tailscale", "receive"]).with(Press("Receive once", setter("tailscaleReceive", Nothing, ANY, Job(task("tailscale.receive", "tailscale-receive", "The Taildrop receive").interactive())))).shown(On(Key("tailscaleInstalled"))).enabled(On(Key("tailscaleInstalled"))),
            ] },
        ],
    },
    Page {
        id: "network/wifi",
        file: "network/WifiPage.qml",
        description: "Join a nearby network, pin the band, or share the one you are on with a QR code.",
        groups: &[
            Group { title: "Adapter", detail: "The radio NetworkManager uses to scan and join. Off is like airplane mode for Wi-Fi only.", rows: &[
                line("Wi-Fi radio", "", "nmcli radio wifi", &["wlan", "rfkill", "airplane", "radio"]).with(Switch(Key("wifiRadio"))).enabled(On(Key("wifiHw"))).words(super::words::Words::Radio),
            ] },
            Group { title: "Networks", detail: "Nearby and saved access points from a NetworkManager scan. The list refreshes while this page is open. Join a known network, or type a password. Enterprise networks also ask for an identity. Forget drops a saved connection.", rows: &[
                line("Nearby networks", "", "", &["scan", "ssid", "empty"]).with(Custom(Wifi("nearby"))),
                line("Could not join", "", "", &["failed"]).with(Custom(Wifi("joinerror"))),
            ] },
            Group { title: "Connection", detail: "Band pins the active network to 2.4, 5, or 6 GHz. The QR code is a scannable copy of the network you are on.", rows: &[
                line("Band", "", "omarchy network band", &["wifi", "wlan", "5ghz", "2.4", "6ghz"]).with(Calls(Key("wifiBandSelected"), Pick(Listed(Key("wifiBands"), &[("auto", "Auto"), ("2.4", "2.4 GHz"), ("5", "5 GHz"), ("6", "6 GHz")])), setter("setWifiBand", Given, ANY.row_same().one_of(&["auto", "2.4", "5", "6"]).same(), Write("wifi.band", Some("band"))))).shown(On(Key("wifiConnected"))).enabled(On(Key("wifiConnected"))).words(super::words::Words::Band),
                line("QR code", "", "omarchy network qr --meta", &["share", "ssid", "password", "qrcode", "wifi-qr"]).with(Custom(Wifi("qr"))),
                line("Restart Wi-Fi", "Unblock rfkill and restart NetworkManager's Wi-Fi. Try this if the radio looks stuck.", "omarchy restart wifi", &["rfkill", "reload", "wlan"]).with(Acts(&[act("Restart", "restart", &[("what", "wifi")], "restartWifi")])).enabled(On(Key("wifiHw"))),
            ] },
            Group { title: "Saved connections", detail: "Metered, priority, MAC randomization, and static IPv4 write through nmcli. Forget is still on the scan list.", rows: &[
                line("Saved networks", "No saved connections.", "", &["saved", "empty"]).with(Custom(Wifi("nosaved"))),
                line("Metered: auto", "", "", &["metered", "priority", "mac", "static", "ipv4"]).with(Custom(Wifi("listed"))),
            ] },
        ],
    },
    Page {
        id: "network/speedtest",
        file: "network/SpeedtestPage.qml",
        description: "A short download, then an upload, on whatever you are connected to now. Opening this page starts a run.",
        groups: &[
            Group { title: "Run", detail: "Runs a download sample, then an upload sample. Each direction is about five seconds. Results are megabits per second on the default route. Opening this page starts a run.", rows: &[
                line("Speed test", "", "omarchy network speedtest", &["bandwidth", "ping", "speedtest"]).with(Custom(Speed("run"))),
                line("Progress", "", "omarchy network speedtest", &["progress", "mbps"]).with(Custom(Speed("progress"))),
                line("Could not run", "", "", &["failed"]).with(Custom(Speed("error"))),
            ] },
            Group { title: "Results", detail: "Inbound and outbound rates from the last run on this page. They are megabits per second on the default route.", rows: &[
                line("Download", "", "omarchy network speedtest down", &["down", "mbps"]).with(Custom(Speed("down"))),
                line("Upload", "", "omarchy network speedtest up", &["up", "mbps"]).with(Custom(Speed("up"))),
            ] },
        ],
    },
    Page {
        id: "bluetooth",
        file: "network/BluetoothPage.qml",
        description: "Paired devices, a scan for new ones, and the adapter power switch.",
        groups: &[
            Group { title: "Adapter", detail: "Powers the adapter and remembers the choice across reboots. Restart unblocks rfkill and brings BlueZ back up.", rows: &[
                line("Bluetooth radio", "", "omarchy bluetooth power", &["bt", "radio", "wireless"]).with(Switch(Key("bluetooth"))).varies(On(Key("bluetooth")), "Paired devices show up below. Scan further down for something new.", "Paired devices and scanning use this adapter."),
                line("Discoverable", "Other devices can see this adapter while this is on.", "adapter.discoverable", &["discoverable", "visible", "pair"]).with(Custom(Bt("discoverable"))),
                line("Restart Bluetooth", "Unblock rfkill and restart BlueZ. Try this if the adapter looks stuck.", "omarchy restart bluetooth", &["rfkill", "bluez", "adapter"]).with(Acts(&[act("Restart", "restart", &[("what", "bluetooth")], "restartBluetooth")])),
            ] },
            Group { title: "Paired", detail: "Connect, disconnect, or forget a paired device. The adapter stays on until you turn it off above.", rows: &[
                line("Paired devices", "No paired devices.", "", &["empty"]).with(Custom(Bt("nopaired"))),
            ] },
            Group { title: "Nearby", detail: "Scan looks for unpaired devices around you. Pair adds one to the list above.", rows: &[
                line("Scan", "", "bluetoothctl scan", &["discover", "pair", "headset"]).with(Custom(Bt("scan"))),
                line("Nearby devices", "No unpaired devices nearby.", "", &["empty", "discover"]).with(Custom(Bt("nonearby"))),
            ] },
        ],
    },
    Page {
        id: "power",
        file: "PowerPage.qml",
        description: "",
        groups: &[
            Group { title: "Profile", detail: "These are power-profiles-daemon modes. Performance uses more power. Power saver stretches battery. On a laptop, AC and battery each remember their own pick.", rows: &[
                line("Power profile", "", "omarchy powerprofiles set", &["performance", "balanced", "power saver", "battery"]).with(Calls(Key("powerProfile"), Pick(Listed(Key("powerProfiles"), &[("power-saver", "Power saver"), ("balanced", "Balanced"), ("performance", "Performance")])), setter("setPowerProfile", Given, ANY.row_same().non_empty().same(), Write("power.profile", Some("name"))))).varies(On(Key("isLaptop")), "How hard the machine works right now. On a laptop this follows the AC or battery pick below.", "How hard the machine works right now."),
                line("On AC", "The profile to use while the charger is plugged in.", "omarchy powerprofiles set ac", &["plugged", "mains", "adapter", "charger"]).with(Picks(Key("powerProfileAc"), Listed(Key("powerProfiles"), &[("power-saver", "Power saver"), ("balanced", "Balanced"), ("performance", "Performance")]))).shown(On(Key("isLaptop"))),
                line("On battery", "The profile to use while you are on battery.", "omarchy powerprofiles set battery", &["unplugged", "discharging", "laptop"]).with(Picks(Key("powerProfileBattery"), Listed(Key("powerProfiles"), &[("power-saver", "Power saver"), ("balanced", "Balanced"), ("performance", "Performance")]))).shown(On(Key("isLaptop"))),
            ] },
            Group { title: "Presentation", detail: "Stay awake, silence notifications, and stop the screensaver. This is session state, not a saved prefs file.", rows: &[
                line("Presentation Mode", "", "~/.local/state/omarchy/atmos-presentation.json", &["presentation", "stay awake", "dnd", "caffeine"]).with(Calls(Key("presentationMode"), Toggle, setter("setPresentationMode", Given, ANY.boolean().same(), Setting("presentationMode")))).varies(On(Key("presentationMode")), "Idle, lock, and notifications are held off until you turn this off.", "A timed stay-awake plus do-not-disturb for talks and sharing a screen."),
            ] },
            Group { title: "Hardware", detail: "Governor and energy preference are what power-profiles-daemon is using. Atmos does not write CPU sysfs while PPD is running.", rows: &[
                line("CPU governor", "What power-profiles-daemon selected. Atmos does not write this.", "scaling_governor", &["governor", "cpu", "schedutil"]).with(Shows(Key("powerGovernor"))).shown(Has(Key("powerGovernor"))),
                line("Energy preference", "The energy_performance_preference power-profiles-daemon selected.", "energy_performance_preference", &["amd", "pstate", "epp"]).with(Shows(Key("amdPstate"))).shown(Has(Key("amdPstate"))),
                line("Charge limit", "Stop charging past this percent. Only on hardware that exposes a threshold.", "charge_control_end_threshold", &["charge", "limit", "battery"]).with(Calls(Key("chargeLimit"), Count(range(50.0, 100.0, 5.0, 0).unset(100.0)), setter("setChargeLimit", Given, ANY.row_same().range(50.0, 100.0).same(), Setting("chargeLimit")))).shown(On(Key("chargeLimitAvailable"))),
            ] },
            Group { title: "Battery", detail: "A one-shot notification with charge and draw. The percentage toggle is the number next to the bar's power icon.", rows: &[
                line("Battery status", "Pop a notification with the current charge and how much power you are drawing.", "omarchy notification battery", &["charge", "notify", "draw", "capacity"]).with(Acts(&[act("Show battery", "notification", &[("kind", "battery")], "showBatteryNotification")])).shown(On(Key("batteryPresent"))).enabled(On(Key("batteryPresent"))),
                line("Battery percentage", "The charge number next to the power icon on the bar.", "omarchy bar set omarchy.power showPercentage", &["battery", "percent", "charge", "laptop"]).with(Switch(Key("powerShowPercentage"))).shown(All(&[On(Key("powerPresent")), On(Key("isLaptop"))])).enabled(All(&[On(Key("powerPresent")), On(Key("isLaptop"))])),
            ] },
        ],
    },
    Page {
        id: "idle",
        file: "IdlePage.qml",
        description: "How long the machine waits before the screensaver and lock. You can also keep it awake or change the screensaver logo.",
        groups: &[
            Group { title: "Timings", detail: "Screensaver and lock are separate timers in ~/.config/omarchy/shell.json. Zero on a slider skips that step.", rows: &[
                line("Screensaver", "How long you can sit still before the screensaver starts. Zero skips the screensaver.", "~/.config/omarchy/shell.json · idle.screensaver", &["idle", "timeout", "sleep"]).with(Calls(Key("idleScreensaver"), Count(range(0.0, 1800.0, 60.0, 0)), setter("setIdle", GivenAnd(Key("idleLock")), ANY.row_same().whole(), Setting("idleScreensaver")))),
                line("Lock", "How long you can sit still before the session locks. Zero skips the lock.", "~/.config/omarchy/shell.json · idle.lock", &["screen lock", "security"]).with(Custom(IdleLock)),
            ] },
            Group { title: "Behavior", detail: "Stay awake skips the screensaver and lock timers. Screensaver and Suspend menu stay available while these switches are on.", rows: &[
                line("Stay awake", "Keep the screen awake and unlocked.", "omarchy toggle idle", &["caffeine", "inhibit", "awake", "sleep"]).with(Switch(Key("stayAwake"))),
                line("Screensaver", "The screensaver runs after the idle timeout.", "omarchy toggle screensaver-off", &["screensaver", "allow", "disable"]).with(Switch(Key("screensaverEnabled"))),
                line("Suspend menu", "Suspend stays in the system menu.", "omarchy toggle suspend-off", &["sleep", "power", "system menu", "allow", "suspend"]).with(Switch(Key("suspendEnabled"))),
            ] },
            Group { title: "Branding", detail: "ASCII art on the screensaver. Choose a picture to turn into that art. Edit opens the text file if you want to write it yourself. Reset puts the Omarchy logo back.", rows: &[
                line("Screensaver logo", "", "omarchy branding screensaver", &["ascii", "logo", "branding"]).varies(On(Key("screensaverBranded")), "You are using custom ASCII art on the screensaver.", "The stock Omarchy logo. Choose a picture to turn into ASCII. Edit opens the text file.").with(Custom(Logo("screensaver"))),
            ] },
            Group { title: "Advanced", detail: "Lid close already locks when the machine is undocked. Omarchy runs that from logind, not from Atmos.", rows: &[
                line("Lid close", "Closing the lid locks when undocked. A docked lid stays unlocked on the other screen. Change this in logind (omarchy-system-lid-close), not here.", "omarchy-system-lid-close", &["lid", "clamshell", "lock", "close"]).with(Says("On")).shown(On(Key("isLaptop"))),
            ] },
        ],
    },
];
