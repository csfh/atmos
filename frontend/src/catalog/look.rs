//! The Desktop pages' groups and rows, as the QML pages have them in order. Search reads
//! the same words (`tests/catalog_parity.rs` holds them to `search.query`).

use super::control::Control::{Acts, Bare, Calls, Choice, Custom, Go, Number, Press, Switch, Text};
use super::control::Field::{Key, Look};
use super::control::Options::Listed;
use super::control::Shape::{Button, Count, Pick, Toggle};
use super::control::When::{All, Differs, Has, Is, Off, On};
use super::control::{act, range};
use super::custom::Custom::{
    AddRule, AddScratch, AddTheme, Bar, BarCount, BootReset, Choose, Display, ExtraThemes,
    LoginWorkspace, NewReminder, NightSchedule, NoExtraThemes, NoRules, RuleCenter, Themes,
    UseSchedule, WorkspaceSwitch,
};
use super::setter::Args::{Given, Lead, Nothing};
use super::setter::Sends::{Setting, Write, WriteBool};
use super::setter::{ANY, setter};
use super::{Group, Page, line};

#[rustfmt::skip]
pub const PAGES: &[Page] = &[
    Page {
        id: "appearance",
        file: "AppearancePage.qml",
        description: "How the desktop looks. The theme sets colors for the shell and themed apps. Wallpaper and the boot screen open from Wallpaper and boot.",
        groups: &[
            Group { title: "Theme", detail: "A theme is a named palette plus the templates Omarchy writes into the shell, terminals, and a few related apps. Switching themes rewrites those configs from the theme's files. Stock themes live in the Omarchy package. If you edit them in place, the next update puts the packaged copies back.", rows: &[
                line("Theme files", "The files behind the current theme. Open the folder if you want to tweak colors or templates by hand.", "omarchy theme dir", &["folder", "directory", "files", "path"]).with(Calls(Key("theme"), Button("Open folder"), setter("openThemeFolder", Nothing, ANY.non_empty(), Write("theme.folder", Some("theme"))))).enabled(Has(Key("theme"))),
                line("Refresh", "Rewrite the current theme from its templates. Handy after you edit theme files.", "omarchy theme refresh", &["reload", "reapply", "templates"]).with(Press("Refresh", setter("refreshTheme", Nothing, ANY, Write("theme.refresh", None)))),
                line("Themes", "The palette in use right now, and every theme you can switch to. The shell and themed apps follow the current one. Hover a card to preview its colors on this window; click to paint the desktop.", "omarchy theme set", &["appearance", "current", "theme", "gallery", "swatch", "preview", "color", "style", "palette"]).with(Custom(Themes)),
            ] },
            Group { title: "Additional themes", detail: "Extra themes are git clones in ~/.config/omarchy/themes. Add clones a repository and switches to it. Update all pulls the latest commit on each one. Remove deletes a theme you installed.", rows: &[
                line("Installed themes", "No extra themes installed.", "omarchy theme extras", &["git", "extra", "clone", "empty"]).with(Custom(NoExtraThemes)),
                line("Installed themes", "", "omarchy theme extras · omarchy theme update · omarchy theme remove", &["git", "extra", "clone", "uninstall", "delete", "pull"]).with(Custom(ExtraThemes)),
                line("Add a theme", "", "omarchy theme install", &["git", "extra", "clone", "download", "install"]).with(Custom(AddTheme)),
            ] },
            Group { title: "Wallpaper and boot", detail: "Background is the desktop picture for this theme. Boot screen is the Plymouth unlock animation and logo you see before you log in.", rows: &[
                line("Background", "", "omarchy theme bg", &["wallpaper", "image", "file", "aether", "palette", "cache"]).with(Go("Choose…", "appearance/background")).words(super::words::Words::Background),
                line("Boot screen", "", "omarchy plymouth", &["plymouth", "sddm", "login", "unlock", "logo", "png"]).with(Go("Configure…", "appearance/boot")).words(super::words::Words::Boot),
            ] },
            Group { title: "Text", detail: "Font and size apply together to the shell, GTK apps, and terminals. Reset puts size back to 12 pixels.", rows: &[
                line("Font", "The monospace face used by the shell and terminals.", "omarchy font set", &["typeface", "monospace"]).with(Calls(Key("font"), Pick(Listed(Key("fonts"), &[])), setter("setFont", Given, ANY.row_same().non_empty().same(), Setting("font")))).enabled(Has(Key("fonts"))),
                line("Text size", "How large type is in the shell, GTK apps, and terminals. You can pick 9 to 20 pixels.", "omarchy display text size", &["scale", "size", "type", "font", "a11y"]).with(Number(Key("textSize"), range(9.0, 20.0, 1.0, 0))),
                line("Reset text size", "Put type back to 12 pixels everywhere Omarchy sets it.", "omarchy display text size reset", &["default", "scale", "12"]).with(Press("Reset", setter("resetTextSize", Nothing, ANY, Write("display.text-size-reset", None)))).enabled(Differs(Key("textSize"), 12)),
            ] },
            Group { title: "Display", detail: "Night light shifts the screen toward amber. Warmth in Kelvin is under Advanced.", rows: &[
                line("Night light", "", "omarchy toggle nightlight", &["nightlight", "warmth", "temperature", "blue light", "kelvin"]).with(Switch(Key("nightlight"))).words(super::words::Words::NightLight),
            ] },
            Group { title: "Advanced", detail: "Warmth is Kelvin. 6500 is daylight. Lower numbers go amber. This talks to hyprsunset the same way the toggle does.", rows: &[
                line("Night light warmth", "", "hyprctl hyprsunset temperature · ~/.config/hypr/hyprsunset.conf", &["kelvin", "warmth", "temperature", "amber", "blue light"]).with(Calls(Key("nightlightTemperature"), Count(range(3000.0, 6500.0, 100.0, 0).unset(4000.0)), setter("setNightlightTemperature", Given, ANY.row_same().range(3000.0, 6500.0).same(), Setting("nightlightTemperature")))).words(super::words::Words::NightWarmth),
                line("Night light schedule", "", "~/.config/hypr/hyprsunset.conf", &["nightlight", "schedule", "hyprsunset", "sunset", "sunrise", "time"]).with(Custom(NightSchedule)),
                line("Use schedule", "", "~/.config/hypr/hyprsunset.conf", &["nightlight", "schedule", "automatic", "hyprsunset", "enable"]).with(Custom(UseSchedule)),
            ] },
        ],
    },
    Page {
        id: "appearance/background",
        file: "appearance/BackgroundPage.qml",
        description: "The picture behind your windows. These controls apply to the current theme.",
        groups: &[
            Group { title: "Wallpaper", detail: "These pictures belong to the current theme. Next walks the set. Choose an image if you want a file from disk. Extra files in the theme folder join the cycle.", rows: &[
                line("Next wallpaper", "Move to the next image this theme ships. Keep pressing if you want to browse the set.", "omarchy theme bg next", &["wallpaper", "cycle"]).with(Acts(&[act("Next", "theme.bg-next", &[], "nextBackground")])),
                line("Choose an image", "Use a JPG, PNG, WebP, or similar file from disk as the wallpaper.", "omarchy theme bg set", &["wallpaper", "file", "image", "path"]).with(Custom(Choose("background"))),
                line("Theme wallpaper folder", "Drop extra images in this theme's folder if you want them in the cycle.", "omarchy theme bg install", &["wallpaper", "folder", "add", "user"]).with(Press("Open folder", setter("openBackgroundFolder", Nothing, ANY, Write("theme.bg-folder", None)))),
                line("Rebuild thumbnails", "Regenerate the small previews the wallpaper switcher shows for this theme.", "omarchy theme bg cache", &["cache", "thumbnail", "preview", "wallpaper"]).with(Press("Rebuild", setter("cacheBackgrounds", Nothing, ANY, Write("theme.bg-cache", None)))),
                line("Aether", "Open Aether if you want to pull a palette from the current wallpaper.", "aether", &["palette", "extract"]).with(Press("Open Aether", setter("openAether", Nothing, ANY, Write("aether", None)))).shown(On(Key("hasAether"))),
            ] },
        ],
    },
    Page {
        id: "appearance/boot",
        file: "appearance/BootPage.qml",
        description: "What you see while the machine unlocks, including the logo on the Plymouth screen.",
        groups: &[
            Group { title: "Plymouth", detail: "The unlock animation before the desktop. A custom logo is a PNG tinted with the current theme. Preview renders it without applying it.", rows: &[
                line("Unlock theme", "The Plymouth theme used at unlock. Default is the stock Omarchy logo.", "omarchy plymouth set by theme", &["plymouth", "sddm", "login", "unlock"]).with(Calls(Key("plymouth"), Pick(Listed(Key("plymouthThemes"), &[])), setter("setPlymouth", Given, ANY.row_same().non_empty().refuse(&["default"]).same(), Setting("plymouth")))).enabled(Has(Key("plymouthThemes"))),
                line("Custom logo", "Pick a PNG for the unlock screen. Omarchy tints it with the current theme colors.", "omarchy plymouth set", &["logo", "png", "custom", "unlock"]).with(Custom(Choose("plymouth"))),
                line("Preview", "Render an unlock screen from a PNG so you can see it before you apply it.", "omarchy plymouth preview", &["preview", "unlock", "plymouth", "logo"]).with(Custom(Choose("plymouth-preview"))),
                line("Reset boot screen", "Put the stock Omarchy unlock theme and login screen back.", "omarchy plymouth reset", &["default", "unlock", "sddm", "login"]).with(Custom(BootReset)),
            ] },
        ],
    },
    Page {
        id: "display",
        file: "DisplaysPage.qml",
        description: "Each monitor keeps its own resolution and refresh rate. Scale and brightness apply to the one you are looking at. On a laptop you also get the built-in panel and its input devices. GPU switching is on Drivers.",
        groups: &[
            Group { title: "", detail: "", rows: &[
                line("Resolution", "", "~/.config/hypr/monitors.lua", &["monitor", "display", "hdmi", "dp", "edp", "resolution", "refresh"]).with(Custom(Display("listed"))),
                line("Refresh rate", "How many frames this panel draws per second. Only the rates the resolution above supports are listed.", "~/.config/hypr/monitors.lua", &["monitor", "display", "refresh", "hertz", "hz", "fps", "highrr"]).with(Custom(Display("listed"))),
                line("Rotation", "How this panel is turned.", "hl.monitor transform", &["rotate", "transform", "portrait"]).with(Custom(Display("listed"))),
                line("Disable this display", "", "hl.monitor disabled", &["disable", "off", "lid"]).with(Custom(Display("listed"))),
                line("Variable refresh", "VRR when this output supports it. Fullscreen-only is the safer game setting.", "hl.monitor vrr", &["vrr", "freesync", "g-sync"]).with(Custom(Display("listed"))),
                line("Bit depth", "10-bit when the panel and cable can do it.", "hl.monitor bitdepth", &["bitdepth", "10-bit", "hdr"]).with(Custom(Display("listed"))),
                line("Color", "", "hl.monitor cm", &["hdr", "color", "srgb", "wide"]).with(Custom(Display("listed"))),
                line("Scale", "How large the interface looks on this monitor. The focused output uses omarchy hyprland monitor scaling. Other outputs write a monitor rule.", "omarchy hyprland monitor scaling", &["scale", "hidpi", "fractional", "dpi"]).with(Custom(Display("listed"))),
                line("Brightness", "How bright this panel is. Works on the built-in display and on some external monitors.", "omarchy brightness display", &["backlight", "ddc", "luminance"]).with(Custom(Display("listed"))),
            ] },
            Group { title: "Displays", detail: "Hyprland did not report any outputs. Refresh after a display is connected. Modes live in ~/.config/hypr/monitors.lua.", rows: &[
            ] },
            Group { title: "Layouts", detail: "Desk keeps every output on. Laptop keeps the built-in panel and needs one. Docked turns the built-in panel off and needs an external monitor. Each write is a monitor rule in ~/.config/hypr/monitors.lua.", rows: &[
                line("Apply a layout", "Uses the outputs Hyprland sees right now. Layouts that would leave no display on stay off.", "hl.monitor", &["desk", "laptop", "docked", "layout", "profile"]).with(Custom(Display("layout"))),
            ] },
            Group { title: "Laptop", detail: "These only show up on a laptop. The built-in screen can turn off while an external monitor is plugged in. Touchpad and touchscreen stay off across a Hyprland reload.", rows: &[
                line("Laptop screen", "", "omarchy hyprland monitor internal", &["lid", "edp", "laptop", "clamshell"]).with(Calls(Key("internalEnabled"), Toggle, setter("setInternalDisplay", Given, ANY.same().boolean(), Write("monitor.internal", Some("on"))))).shown(On(Key("internalPresent"))).enabled(All(&[On(Key("internalPresent")), On(Key("externalPresent"))])).varies(On(Key("externalPresent")), "Keep the built-in panel on.", "This is the only display, so it has to stay on."),
                line("Mirror to the first external", "The same picture on the laptop screen and the first external monitor.", "omarchy hyprland monitor internal mirror", &["mirror", "clone", "duplicate"]).with(Calls(Key("mirroring"), Toggle, setter("setInternalMirror", Given, ANY.same().boolean(), Write("monitor.mirror", Some("on"))))).shown(All(&[On(Key("internalPresent")), On(Key("externalPresent"))])).enabled(All(&[On(Key("internalPresent")), On(Key("externalPresent"))])),
                line("Touchpad", "Finger and pointer input on the trackpad. The choice survives a Hyprland reload.", "omarchy toggle touchpad", &["trackpad", "touchpad", "pointer", "mouse", "input"]).with(Custom(Display("touchpad"))),
                line("Touchscreen", "", "omarchy toggle touchscreen", &["touch", "touchscreen", "tablet", "digitizer"]).varies(On(Key("touchscreenPresent")), "Finger input on the display. The choice survives a Hyprland reload.", "Hyprland has not reported a touch device.").with(Switch(Key("touchscreenEnabled"))).shown(On(Key("touchscreenPresent"))).enabled(On(Key("touchscreenPresent"))),
                line("Keyboard backlight", "", "omarchy brightness keyboard", &["kbd", "backlight", "leds"]).with(Custom(Display("backlight"))),
            ] },
        ],
    },
    Page {
        id: "windows",
        file: "WindowsPage.qml",
        description: "Gaps, corners, and the tiling layout. Keybindings and window rules have their own pages. Reset only removes the block Atmos wrote.",
        groups: &[
            Group { title: "Shortcuts and rules", detail: "Keybindings lists what Hyprland is running and lets you add a personal override. Window rules float, tile, or pin a class without rewriting hyprland.lua.", rows: &[
                line("Keybindings", "", "~/.config/hypr/bindings.lua", &["keybinding", "hotkey", "shortcut", "bind", "unbind", "chord"]).with(Go("Configure…", "keybindings")).words(super::words::Words::Overrides),
                line("Window rules", "", "~/.config/hypr/atmos.lua", &["window", "rule", "float", "tile", "class", "regex"]).with(Go("Configure…", "windows/rules")).words(super::words::Words::Rules),
            ] },
            Group { title: "Look", detail: "These write a managed block at the end of ~/.config/hypr/looknfeel.lua. Tight windows overrides the sliders until you turn it off.", rows: &[
                line("Inner gaps", "Space between windows. Tight windows sets this to zero.", "~/.config/hypr/looknfeel.lua · general.gaps_in", &["gap", "padding", "space", "tile"]).with(Number(Look("gapsIn"), range(0.0, 32.0, 1.0, 0))).enabled(Off(Key("hyprNoGaps"))),
                line("Outer gaps", "Space between windows and the edge of the screen.", "~/.config/hypr/looknfeel.lua · general.gaps_out", &["gap", "margin", "edge"]).with(Number(Look("gapsOut"), range(0.0, 48.0, 1.0, 0))).enabled(Off(Key("hyprNoGaps"))),
                line("Border", "Thickness of the window outline.", "~/.config/hypr/looknfeel.lua · general.border_size", &["border", "outline", "frame"]).with(Number(Look("borderSize"), range(0.0, 8.0, 1.0, 0))).enabled(Off(Key("hyprNoGaps"))),
                line("Corners", "How round the window corners are.", "~/.config/hypr/looknfeel.lua · decoration.rounding", &["rounding", "radius", "corners"]).with(Number(Look("rounding"), range(0.0, 24.0, 1.0, 0))).enabled(Off(Key("hyprNoGaps"))),
                line("Blur", "Blur what sits behind a transparent window.", "~/.config/hypr/looknfeel.lua · decoration.blur", &["blur", "glass", "transparent"]).with(Switch(Look("blur"))),
                line("Shadow", "A shadow under each window.", "~/.config/hypr/looknfeel.lua · decoration.shadow", &["shadow", "drop"]).with(Switch(Look("shadow"))),
                line("Tiling", "Dwindle splits the screen in two. Scrolling walks columns sideways, like niri.", "~/.config/hypr/looknfeel.lua · general.layout", &["dwindle", "scrolling", "niri", "layout", "tile"]).with(Choice(Look("layout"), &[("dwindle", "Dwindle"), ("scrolling", "Scrolling")])),
                line("Column width", "How wide each scrolling column is. Near 1 shows one column. Near 0.5 shows two.", "~/.config/hypr/looknfeel.lua · scrolling.column_width", &["column", "scrolling", "width"]).with(Number(Look("columnWidth"), range(0.3, 1.0, 0.01, 2))).shown(Is(Look("layout"), "scrolling")).enabled(Is(Look("layout"), "scrolling")),
                line("Reset look", "Remove the block Atmos wrote. Hyprland goes back to the rest of looknfeel.lua and the Omarchy defaults.", "~/.config/hypr/looknfeel.lua", &["reset", "default", "looknfeel"]).with(Acts(&[act("Reset", "hypr.reset", &[("group", "look")], "resetHyprLook")])).enabled(On(Key("hyprLookManaged"))),
            ] },
            Group { title: "Behavior", detail: "Tight windows and square aspect are permanent Hyprland flags. Transparency and tiled fullscreen apply to the window that is focused right now.", rows: &[
                line("Tight windows", "No gaps, borders, or rounding. The sliders above stay disabled while this is on.", "omarchy hyprland toggle window-no-gaps", &["gaps", "borderless", "tight", "no gaps"]).with(Switch(Key("hyprNoGaps"))),
                line("Square single window", "When one window is on the screen, keep it from stretching across an ultrawide.", "omarchy hyprland toggle single-window-aspect-ratio", &["aspect", "square", "ultrawide", "single"]).with(Switch(Key("hyprSquareAspect"))),
                line("Opaque windows", "Turn off Omarchy's default window transparency, so unfocused windows stop looking faded.", "~/.config/hypr/atmos.lua · tag default-opacity", &["opacity", "transparent", "transparency", "inactive", "faded", "dim"]).with(Switch(Key("windowOpaque"))),
                line("This workspace", "", "omarchy hyprland workspace layout toggle", &["workspace", "layout", "dwindle", "scrolling"]).with(Acts(&[act("Switch layout", "hypr.workspace-layout-toggle", &[], "toggleWorkspaceLayout")])).varies(Is(Key("hyprWorkspaceLayout"), "scrolling"), "This workspace is scrolling. Switch layout turns it to dwindle.", "This workspace is dwindle. Switch layout turns it to scrolling."),
                line("This window", "Switch transparency or tiled fullscreen on the focused window. That change lasts for this window only.", "omarchy hyprland window transparency toggle", &["opacity", "transparent", "fullscreen", "tiled"]).with(Acts(&[act("Switch transparency", "window.transparency-toggle", &[], "toggleWindowTransparency"), act("Switch tiled full", "window.tiled-fullscreen-toggle", &[], "toggleTiledFullscreen")])),
            ] },
            Group { title: "Advanced", detail: "Dim, animations, cursor, and tearing. These go in the same looknfeel.lua block as the sliders above.", rows: &[
                line("Dim others", "Darken windows that are not focused.", "~/.config/hypr/looknfeel.lua · decoration.dim_inactive", &["dim", "inactive", "focus"]).with(Switch(Look("dimInactive"))),
                line("Dim strength", "How far unfocused windows go toward black.", "~/.config/hypr/looknfeel.lua · decoration.dim_strength", &["dim", "strength", "inactive"]).with(Number(Look("dimStrength"), range(0.0, 1.0, 0.05, 2))).shown(On(Look("dimInactive"))).enabled(On(Look("dimInactive"))),
                line("Animations", "Window open, close, and fade motion.", "~/.config/hypr/looknfeel.lua · animations.enabled", &["animation", "motion", "reduce", "a11y"]).with(Switch(Look("animations"))),
                line("Hide cursor while typing", "The pointer disappears when you start typing.", "~/.config/hypr/looknfeel.lua · cursor.hide_on_key_press", &["cursor", "pointer", "hide", "type"]).with(Switch(Look("cursorHideOnKey"))),
                line("Warp cursor on workspace", "The pointer jumps when you change workspaces.", "~/.config/hypr/looknfeel.lua · cursor.warp_on_change_workspace", &["cursor", "warp", "workspace"]).with(Switch(Look("cursorWarp"))),
                line("Resize on border", "Drag a window edge to resize it, without the modifier key.", "~/.config/hypr/looknfeel.lua · general.resize_on_border", &["resize", "border", "drag"]).with(Switch(Look("resizeOnBorder"))),
                line("Allow tearing", "Games and other windows may tear if they ask. That can cut input lag.", "~/.config/hypr/looknfeel.lua · general.allow_tearing", &["tearing", "vrr", "latency", "game"]).with(Switch(Look("allowTearing"))),
                line("Cursor size", "How large the pointer is.", "~/.config/hypr/looknfeel.lua · HYPRCURSOR_SIZE", &["cursor", "pointer", "size", "a11y"]).with(Number(Look("cursorSize"), range(8.0, 64.0, 2.0, 0))),
                line("Active opacity", "How solid a focused window is.", "~/.config/hypr/looknfeel.lua · decoration.active_opacity", &["opacity", "transparency", "alpha"]).with(Number(Look("activeOpacity"), range(0.2, 1.0, 0.05, 2))),
                line("Inactive opacity", "How solid unfocused windows are.", "~/.config/hypr/looknfeel.lua · decoration.inactive_opacity", &["opacity", "transparency", "alpha", "inactive"]).with(Number(Look("inactiveOpacity"), range(0.2, 1.0, 0.05, 2))),
                line("Preserve split", "Keep the dwindle split after the last window in a branch closes.", "~/.config/hypr/looknfeel.lua · dwindle.preserve_split", &["dwindle", "split", "tile"]).with(Switch(Look("preserveSplit"))),
                line("Focus on activate", "Focus a window when another client asks Hyprland to activate it.", "~/.config/hypr/looknfeel.lua · misc.focus_on_activate", &["focus", "activate", "urgent"]).with(Switch(Look("focusOnActivate"))),
                line("Swallow terminals", "A terminal that launches a GUI app is swallowed into that window.", "~/.config/hypr/looknfeel.lua · misc.enable_swallow", &["swallow", "terminal"]).with(Switch(Look("enableSwallow"))),
                line("Swallow regex", "Which terminal classes Hyprland swallows. Empty keeps the Hyprland default.", "~/.config/hypr/looknfeel.lua · misc.swallow_regex", &["swallow", "regex"]).with(Text(Look("swallowRegex"), "kitty|alacritty")).shown(On(Look("enableSwallow"))),
                line("Focus under fullscreen", "What happens when focus moves to a window under a fullscreen one. 0 ignores it, 1 takes over, 2 stays underneath.", "~/.config/hypr/looknfeel.lua · misc.on_focus_under_fullscreen", &["fullscreen", "focus"]).with(Choice(Look("onFocusUnderFullscreen"), &[("0", "Ignore"), ("1", "Take over"), ("2", "Stay under")])),
            ] },
        ],
    },
    Page {
        id: "windows/rules",
        file: "windows/RulesPage.qml",
        description: "These write a managed block at the end of ~/.config/hypr/atmos.lua. The Atmos window rules stay above that block. Lines in hyprland.lua stay there too.",
        groups: &[
            Group { title: "", detail: "", rows: &[
                line("Center", "A floating window sits in the middle of the screen.", "", &[]).with(Custom(RuleCenter)),
            ] },
            Group { title: "Your rules", detail: "Match is a window class, or a regex Hyprland applies to class. Use focused copies the class of the window that is focused right now.", rows: &[
                line("Add a rule", "Float, tile, center, size, or pin a class to a workspace.", "~/.config/hypr/atmos.lua", &["window", "rule", "float", "tile", "class"]).with(Custom(AddRule)),
                line("Rules", "No personal window rules.", "", &["empty", "rules"]).with(Custom(NoRules)),
            ] },
        ],
    },
    Page {
        id: "workspaces",
        file: "WorkspacesPage.qml",
        description: "How many numbered workspaces stay around, what the bar calls them, and which monitor they live on.",
        groups: &[
            Group { title: "Count and switching", detail: "Hyprland keeps numbered workspaces 1–10 persistent, so Super+N still has somewhere to go. The slider is how many of those the bar always paints. Workspaces past that with windows still show, like stock Hypr.", rows: &[
                line("Number of workspaces", "How many numbered workspaces the bar always shows. Workspaces past that with windows still appear. Hyprland still keeps 1–10.", "~/.config/omarchy/shell.json", &["count", "number", "ten", "bar"]).with(Custom(BarCount)),
                line("Wrap switching", "The last workspace wraps to the first.", "workspace e+1", &["wrap", "cycle", "switch"]).with(Custom(WorkspaceSwitch("workspaceWrapSwitch"))),
                line("Super + mouse wheel", "The mouse wheel moves between workspaces while Super is held. Atmos writes that as an override.", "SUPER + mouse_down", &["wheel", "scroll", "switch"]).with(Custom(WorkspaceSwitch("workspaceWheelSwitch"))),
                line("Show names in the bar", "Omarchy's stock widget paints numbers. This clone paints the names you set below.", "~/.config/omarchy/plugins", &["bar", "name", "plugin", "clone", "label"]).with(Calls(Key("workspaceBarNames"), Toggle, setter("dispatchSetting", Lead("workspaceBarNames"), ANY, Setting("workspaceBarNames")))),
                line("Open on login", "Which numbered workspace Hyprland focuses after you log in.", "hl.workspace_rule · default", &["default", "login", "start"]).with(Custom(LoginWorkspace)),
            ] },
            Group { title: "Names and monitors", detail: "A name is stored on the Hyprland workspace. Turn on Show names in the bar to paint it on the strip. A monitor pins that workspace to one output. Leave either blank.", rows: &[
            ] },
            Group { title: "Scratch workspaces", detail: "A special workspace stays off the numbered strip until you toggle it.", rows: &[
                line("Add a scratch workspace", "", "special:name", &["scratch", "special", "toggle"]).with(Custom(AddScratch)),
            ] },
        ],
    },
    Page {
        id: "bar",
        file: "BarPage.qml",
        description: "Where the bar sits and what it shows.",
        groups: &[
            Group { title: "Layout", detail: "The bar can sit on any edge. Transparent lets wallpaper show through. Hiding the bar leaves the rest of the shell running.", rows: &[
                line("Position", "Which edge of the screen the bar sits on. Top is the usual place.", "omarchy bar position", &["top", "bottom", "left", "right", "menu bar"]).with(Choice(Key("barPosition"), &[("top", "Top"), ("bottom", "Bottom"), ("left", "Left"), ("right", "Right")])),
                line("Transparent bar", "The wallpaper shows through the bar.", "omarchy bar transparent", &["opacity", "see-through"]).with(Switch(Key("barTransparent"))),
                line("Show bar", "Keep the bar visible. Turn this off to hide it.", "omarchy toggle bar", &["hide", "visible", "autohide"]).with(Switch(Key("barVisible"))),
            ] },
            Group { title: "Spacer", detail: "A blank gap you can put between widgets. Add inserts omarchy.spacer. Remove takes it off the bar. Width is in pixels.", rows: &[
                line("Spacer", "Put a blank gap between widgets. You can set the width after it is there.", "omarchy bar put omarchy.spacer", &["gap", "space", "padding", "layout", "missing", "add"]).with(Press("Add", setter("addSpacer", Nothing, ANY.blocked(Key("spacerPresent")), WriteBool("bar.spacer", "on", true)))).shown(Off(Key("spacerPresent"))),
                line("Width", "How wide the blank gap is between the widgets on either side.", "omarchy bar set omarchy.spacer size", &["gap", "space", "padding", "layout"]).with(Calls(Key("spacerSize"), Count(range(0.0, 64.0, 1.0, 0)), setter("setSpacerSize", Given, ANY.row_same().range(0.0, 64.0).same(), Setting("spacerSize")))).shown(On(Key("spacerPresent"))).enabled(On(Key("spacerPresent"))),
                line("Remove", "Take the blank gap off the bar.", "omarchy plugin disable omarchy.spacer", &["gap", "space", "padding", "layout", "delete", "remove"]).with(Press("Remove", setter("removeSpacer", Nothing, ANY.needs(Key("spacerPresent")), WriteBool("bar.spacer", "on", false)))).shown(On(Key("spacerPresent"))).enabled(On(Key("spacerPresent"))),
            ] },
            Group { title: "Tray", detail: "Hidden icons stay listed so you can show them again. Pinned icons stay visible even when the tray tucks extras away.", rows: &[
                line("Hidden icons", "No hidden tray icons.", "omarchy bar set omarchy.tray hidden", &["system tray", "sni", "unhide", "show", "icons", "empty"]).with(Custom(Bar("hiddenNone"))),
                line("Hidden icons", "Hidden right now: ", "omarchy bar set omarchy.tray hidden", &["system tray", "sni", "unhide", "show", "icons"]).with(Custom(Bar("hidden"))),
                line("Pinned icons", "No pinned tray icons.", "omarchy bar set omarchy.tray pinned", &["system tray", "sni", "pin", "unpin", "always visible", "empty"]).with(Custom(Bar("pinnedNone"))),
                line("Pinned icons", "Always visible: ", "omarchy bar set omarchy.tray pinned", &["system tray", "sni", "pin", "unpin", "always visible"]).with(Custom(Bar("pinned"))),
            ] },
            Group { title: "Indicators", detail: "Little status icons for things like dictation, recording, night light, and stay awake. Always show keeps them visible when they are idle.", rows: &[
                line("Always show", "Keep these status icons on the bar even when they are idle.", "omarchy bar set omarchy.indicators alwaysShow", &["status", "icons", "stay awake", "night light", "dnd"]).with(Switch(Key("indicatorsAlwaysShow"))).shown(On(Key("indicatorsPresent"))).enabled(On(Key("indicatorsPresent"))),
                line("Shown indicators", "Which status icons this widget may show. Leave them all on if you want the full set.", "omarchy bar set omarchy.indicators items", &["dictation", "recording", "reminder", "night light", "dnd", "stay awake"]).with(Custom(Bar("indicators"))),
            ] },
            Group { title: "Agents", detail: "Usage for coding agents on this machine. Sync writes a snapshot into a folder you share with other machines.", rows: &[
                line("Usage refresh", "How often the widget rebuilds usage numbers for the agents on this machine.", "omarchy bar set omarchy.agents refreshIntervalSec", &["claude", "codex", "grok", "interval", "usage"]).with(Custom(Bar("refresh"))),
                line("Sync usage", "This machine's usage goes into the sync folder, and snapshots from other machines fold in.", "omarchy bar set omarchy.agents syncMode", &["syncthing", "dropbox", "aggregate", "share"]).with(Switch(Key("agentsSync"))).shown(On(Key("agentsPresent"))).enabled(On(Key("agentsPresent"))),
                line("Sync folder", "A folder you already sync with Syncthing, Dropbox, or rsync. Needed when usage sync is on.", "omarchy bar set omarchy.agents syncDir", &["syncthing", "dropbox", "rsync", "folder", "path"]).with(Custom(Bar("syncdir"))),
                line("Snapshot file", "The filename this machine writes in the sync folder. Leave it blank to use hostname.json.", "omarchy bar set omarchy.agents syncFileName", &["snapshot", "filename", "hostname", "json"]).with(Custom(Bar("syncfile"))),
                line("Device id", "How this machine is named inside the synced snapshots. Leave it blank to use the hostname.", "omarchy bar set omarchy.agents syncDeviceId", &["device", "hostname", "machine", "id"]).with(Custom(Bar("device"))),
            ] },
            Group { title: "Clock", detail: "How the bar clock looks. Right-click cycles the same formats. Birth year is optional and draws a life bar in the calendar popup.", rows: &[
                line("Clock format", "How the clock reads on the bar. Right-clicking the clock walks through these same presets.", "omarchy bar set omarchy.clock format", &["time", "24-hour", "am pm", "date"]).with(Custom(Bar("clock"))),
                line("Alternate format", "A second style in the clock's right-click cycle. Handy if you sometimes want the date too.", "omarchy bar set omarchy.clock formatAlt", &["date", "week", "cycle", "secondary"]).with(Custom(Bar("clockAlt"))),
                line("Week starts on", "First day of the week in the calendar popup. Locale default follows the system language.", "omarchy bar set omarchy.clock weekStartDay", &["calendar", "week", "sunday", "monday", "start"]).with(Custom(Bar("week"))),
                line("Birth year", "", "omarchy bar set omarchy.clock birthYear", &["age", "life", "calendar", "memento"]).with(Custom(Bar("birth"))),
                line("Life expectancy", "How far the calendar life bar runs. Leave it blank to use 90 years.", "omarchy bar set omarchy.clock lifeExpectancy", &["age", "life", "span", "memento", "years"]).with(Custom(Bar("life"))),
            ] },
            Group { title: "Advanced", detail: "Shell plugins Omarchy discovered. The bar itself cannot be disabled. Other widgets and services can.", rows: &[
                line("Plugins", "No plugins listed.", "omarchy plugin list", &["plugin", "widget", "shell", "empty"]).with(Custom(Bar("pluginsNone"))),
            ] },
        ],
    },
    Page {
        id: "notifications",
        file: "NotificationsPage.qml",
        description: "Do not disturb, a test toast, and reminders that already live in Omarchy.",
        groups: &[
            Group { title: "Quiet", detail: "Do not disturb hides ordinary toasts. Critical alerts still get through.", rows: &[
                line("Do not disturb", "Hide ordinary notification toasts. Critical alerts still get through.", "omarchy toggle notification silencing", &["dnd", "silent", "mute", "quiet"]).with(Switch(Key("doNotDisturb"))),
            ] },
            Group { title: "Send", detail: "These fire an Omarchy toast right now. Use Test if you want to see that notifications still land.", rows: &[
                line("Test toast", "Send a sample notification from Atmos.", "omarchy notification send", &["test", "toast", "notify", "send"]).with(Acts(&[act("Send test", "notification.test", &[], "sendTestNotification")])),
                line("Time", "Show the current time and date as a toast.", "omarchy notification time", &["time", "clock", "date", "toast"]).with(Acts(&[act("Show time", "notification", &[("kind", "time")], "sendTimeNotification")])),
                line("Battery", "Show the current battery toast.", "omarchy notification battery", &["battery", "charge", "toast"]).with(Acts(&[act("Show battery", "notification", &[("kind", "battery")], "showBatteryNotification")])).shown(On(Key("batteryPresent"))).enabled(On(Key("batteryPresent"))),
                line("Weather", "Show the current forecast toast.", "omarchy notification weather", &["weather", "forecast", "toast"]).with(Acts(&[act("Show weather", "notification", &[("kind", "weather")], "sendWeatherNotification")])).shown(On(Key("weatherPresent"))).enabled(On(Key("weatherPresent"))),
            ] },
            Group { title: "Reminders", detail: "A reminder is a systemd timer that sends an Omarchy toast. Clear drops every outstanding one.", rows: &[
                line("New reminder", "Minutes from now, and an optional message.", "omarchy reminder", &["reminder", "timer", "later", "minutes"]).with(Custom(NewReminder)),
                line("Clear reminders", "", "omarchy reminder clear", &["reminder", "clear", "cancel"]).with(Acts(&[act("Show", "reminder.show", &[], "showReminders"), act("Clear", "reminder.clear", &[], "clearReminders")])).shown(On(Key("reminderActive"))).enabled(On(Key("reminderActive"))).words(super::words::Words::Reminders),
                line("Reminders", "No reminders waiting.", "", &["reminder", "empty"]).with(Bare).shown(Off(Key("reminderActive"))),
            ] },
        ],
    },
    Page {
        id: "profiles",
        file: "ProfilesPage.qml",
        description: "A profile is a named bundle of Settings keys. Apply goes through the same writers as an Omafile. Coding, Gaming, and Battery ship with Atmos. Save your own as an Omafile.",
        groups: &[
            Group { title: "Built-in", detail: "Each bundle writes power, idle, notifications, and window look through Settings.commandFor.", rows: &[
            ] },
            Group { title: "Your own", detail: "Write an Omafile, then apply it later. An Omafile is a Markdown document of an Omarchy system, not a second prefs store.", rows: &[
                line("Omafile", "Write this Omarchy system to a file, or apply one you already have.", "atmos export", &["omafile", "import", "export", "markdown"]).with(Go("Open…", "export")),
            ] },
        ],
    },
];
