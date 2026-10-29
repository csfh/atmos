//! The Controls pages' groups and rows, as the QML pages have them in order. Search reads
//! the same words (`tests/catalog_parity.rs` holds them to `search.query`).

use super::control::Control::{Acts, Buttons, Calls, Choice, Custom, Number, Press, Switch};
use super::control::Field::{Derived, Input, Key, Look};
use super::control::Options::Devices;
use super::control::Shape::{Count, Pick, Toggle};
use super::control::When::{All, Flag, Has, Idle, Many, Off, On};
use super::control::{act, range};
use super::custom::Custom::{Bind, CapturePick, Kb, RecSwitch, ResizeWebcam, SaveFolder, TakeShot};
use super::setter::Args::{Given, Nothing};
use super::setter::Sends::{Device, HyprInput, Job, Setting, Write};
use super::setter::{ANY, btn, setter, task};
use super::{Group, Page, line};

#[rustfmt::skip]
pub const PAGES: &[Page] = &[
    Page {
        id: "input",
        file: "InputPage.qml",
        description: "How the mouse, touchpad, and keyboard feel. Turning the laptop trackpad off is on Displays. The system layout picker is on System.",
        groups: &[
            Group { title: "Pointer", detail: "Sensitivity and acceleration for the mouse and trackpad. These write a managed block in ~/.config/hypr/input.lua.", rows: &[
                line("Sensitivity", "Pointer speed. Zero is the Hyprland default. Negative is slower.", "~/.config/hypr/input.lua · input.sensitivity", &["mouse", "pointer", "speed", "sensitivity"]).with(Number(Input("sensitivity"), range(-1.0, 1.0, 0.02, 2))),
                line("Acceleration", "Adaptive speeds up as you move. Flat keeps a steady ratio.", "~/.config/hypr/input.lua · input.accel_profile", &["accel", "acceleration", "flat", "adaptive"]).with(Choice(Input("accelProfile"), &[("", "Default"), ("adaptive", "Adaptive"), ("flat", "Flat")])),
                line("Scroll inertia", "How a high-resolution or free-spin mouse wheel is turned into scroll events. Smooth keeps the fine motion. Stepped turns it into clicks.", "~/.config/hypr/input.lua · input.emulate_discrete_scroll", &["inertia", "wheel", "high-res", "discrete", "smooth", "scroll"]).with(Choice(Input("emulateDiscreteScroll"), &[("0", "Smooth"), ("1", "Default"), ("2", "Stepped")])),
            ] },
            Group { title: "Touchpad", detail: "Feel for the trackpad. The on/off switch for the device itself is on Displays.", rows: &[
                line("Natural scroll", "Content moves with your fingers, the way a phone does.", "~/.config/hypr/input.lua · input.touchpad.natural_scroll", &["natural", "invert", "scroll", "direction"]).with(Switch(Input("naturalScroll"))),
                line("Scroll speed", "How far two-finger scroll moves.", "~/.config/hypr/input.lua · input.touchpad.scroll_factor", &["scroll", "factor", "speed"]).with(Number(Input("scrollFactor"), range(0.1, 2.0, 0.1, 1))),
                line("Two-finger click", "A two-finger tap is a right click.", "~/.config/hypr/input.lua · input.touchpad.clickfinger_behavior", &["clickfinger", "right click", "tap"]).with(Switch(Input("clickfinger"))),
                line("Ignore while typing", "The trackpad rests while you type, so a palm does not move the pointer.", "~/.config/hypr/input.lua · input.touchpad.disable_while_typing", &["disable while typing", "palm", "reject"]).with(Switch(Input("disableWhileTyping"))),
                line("Three-finger drag", "Three fingers down and moving drags, like a click-and-hold.", "~/.config/hypr/input.lua · input.touchpad.drag_3fg", &["three finger", "drag"]).with(Switch(Input("drag3fg"))),
            ] },
            Group { title: "Keyboard", detail: "Repeat and numlock for Hyprland. The console and login layout stay on System.", rows: &[
                line("Repeat rate", "How many times a held key repeats each second.", "~/.config/hypr/input.lua · input.repeat_rate", &["repeat", "rate", "hold"]).with(Number(Input("repeatRate"), range(10.0, 80.0, 1.0, 0))),
                line("Repeat delay", "How long you hold a key before it starts repeating.", "~/.config/hypr/input.lua · input.repeat_delay", &["repeat", "delay", "hold"]).with(Number(Input("repeatDelay"), range(150.0, 600.0, 10.0, 0))),
                line("Numlock on boot", "The number pad is on when Hyprland starts.", "~/.config/hypr/input.lua · input.numlock_by_default", &["numlock", "keypad"]).with(Switch(Input("numlock"))),
                line("Reset input", "Remove the block Atmos wrote. Hyprland goes back to the rest of input.lua and the Omarchy defaults.", "~/.config/hypr/input.lua", &["reset", "default", "input"]).with(Acts(&[act("Reset", "hypr.reset", &[("group", "input")], "resetHyprInput")])).enabled(On(Key("hyprInputManaged"))),
            ] },
            Group { title: "Advanced", detail: "Follow mouse, waking the screen, an optional Hyprland layout list, and a three-finger workspace swipe.", rows: &[
                line("Follow mouse", "How the pointer picks the focused window. 1 is the usual Omarchy setting.", "~/.config/hypr/input.lua · input.follow_mouse", &["follow", "focus", "mouse"]).with(Choice(Input("followMouse"), &[("0", "Click to focus"), ("1", "Follow"), ("2", "Follow, detached"), ("3", "Follow, loose")])),
                line("Wake on key", "A key press turns the screen back on after DPMS off.", "~/.config/hypr/input.lua · misc.key_press_enables_dpms", &["dpms", "wake", "key"]).with(Switch(Input("keyPressDpms"))),
                line("Wake on mouse", "Moving the pointer turns the screen back on after DPMS off.", "~/.config/hypr/input.lua · misc.mouse_move_enables_dpms", &["dpms", "wake", "mouse"]).with(Switch(Input("mouseMoveDpms"))),
                line("Hyprland layouts", "", "~/.config/hypr/input.lua · input.kb_layout", &["layout", "xkb", "multi", "us,dk"]).with(Custom(Kb("layouts"))),
                line("Variants", "", "~/.config/hypr/input.lua · input.kb_variant", &["variant", "intl", "nodeadkeys"]).with(Custom(Kb("variants"))),
                line("Alt+Alt layout switch", "Left Alt and Right Alt together cycle the Hyprland layouts above.", "~/.config/hypr/input.lua · input.kb_options", &["grp", "alts", "switch", "layout"]).with(Custom(Kb("toggle"))),
                line("Three-finger swipe", "", "~/.config/hypr/input.lua · hl.gesture", &["gesture", "swipe", "workspace", "three"]).with(Calls(Key("hyprWorkspaceGesture"), Toggle, setter("setHyprWorkspaceGesture", Given, ANY.blocked(Key("hyprWorkspaceGestureUnmanaged")).same(), HyprInput("workspaceGesture")))).enabled(Off(Key("hyprWorkspaceGestureUnmanaged"))).varies(On(Key("hyprWorkspaceGestureUnmanaged")), "This gesture is already in ~/.config/hypr/input.lua outside the Atmos block, so the switch stays on and Atmos will not change it. Comment that line out to let Atmos own it.", "Swipe sideways with three fingers to change workspace."),
            ] },
        ],
    },
    Page {
        id: "keybindings",
        file: "windows/BindingsPage.qml",
        description: "The list is what Hyprland is running now. Add writes a managed block at the end of ~/.config/hypr/bindings.lua. Lines you typed yourself stay. Remove only deletes a row Atmos added.",
        groups: &[
            Group { title: "", detail: "", rows: &[
                line("Unbind only", "The default chord is removed, with no replacement.", "", &[]).with(Custom(Bind("unbind"))),
            ] },
            Group { title: "Your overrides", detail: "These lines live in the Atmos block of bindings.lua. Adding a chord that is already taken writes hl.unbind first, then o.bind.", rows: &[
                line("Add a binding", "A chord, a short name, and the command to run. Unbind only turns a default off.", "~/.config/hypr/bindings.lua", &["bind", "unbind", "hotkey", "shortcut", "chord"]).with(Custom(Bind("add"))),
                line("Overrides", "No personal bindings.", "", &["empty", "bindings"]).with(Custom(Bind("none"))),
            ] },
            Group { title: "What is bound", detail: "This is omarchy menu keybindings --print. Filter if you want to find a chord before you override it.", rows: &[
                line("Bindings", "No bindings reported.", "omarchy menu keybindings --print", &["empty", "keybinding"]).with(Custom(Bind("empty"))),
                line("Filter", "", "omarchy menu keybindings --print", &["search", "filter", "list"]).with(Custom(Bind("filter"))),
                line("Bindings", "No matching bindings.", "", &["empty", "filter"]).with(Custom(Bind("nomatch"))),
                line("More bindings", "Showing ", "omarchy menu keybindings --print", &["more", "show", "all", "list"]).with(Custom(Bind("more"))),
            ] },
        ],
    },
    Page {
        id: "accessibility",
        file: "AccessibilityPage.qml",
        description: "Motion, type size, the pointer, and touch. These use the same writers as Windows, Appearance, and Displays.",
        groups: &[
            Group { title: "Motion and type", detail: "Animations write the look sentinel. Text size is the same slider as Appearance.", rows: &[
                line("Animations", "Window open, close, and fade motion.", "~/.config/hypr/looknfeel.lua · animations.enabled", &["animation", "motion", "reduce", "a11y"]).with(Switch(Look("animations"))),
                line("Text size", "How large type is in the shell, GTK apps, and terminals. You can pick 9 to 20 pixels.", "omarchy display text size", &["scale", "size", "type", "font", "a11y"]).with(Number(Key("textSize"), range(9.0, 20.0, 1.0, 0))),
            ] },
            Group { title: "Pointer", detail: "Cursor hide is the same look key as Windows. Size is a new look-sentinel field.", rows: &[
                line("Hide cursor while typing", "The pointer disappears when you start typing.", "~/.config/hypr/looknfeel.lua · cursor.hide_on_key_press", &["cursor", "pointer", "hide", "type"]).with(Switch(Look("cursorHideOnKey"))),
                line("Cursor size", "How large the pointer is.", "~/.config/hypr/looknfeel.lua · HYPRCURSOR_SIZE", &["cursor", "pointer", "size", "a11y"]).with(Number(Look("cursorSize"), range(8.0, 64.0, 2.0, 0))),
            ] },
            Group { title: "Touch", detail: "This is the same touchscreen switch as Displays.", rows: &[
                line("Touchscreen", "", "omarchy toggle touchscreen", &["touch", "touchscreen", "tablet", "digitizer"]).with(Switch(Key("touchscreenEnabled"))).enabled(On(Key("touchscreenPresent"))).varies(On(Key("touchscreenPresent")), "Finger input on the display. The choice survives a Hyprland reload.", "Hyprland has not reported a touch device."),
            ] },
            Group { title: "Tools", detail: "Herdr is a screen reader Omarchy can launch in a terminal when the package is present.", rows: &[
                line("Herdr", "", "omarchy launch terminal herdr", &["herdr", "screen reader", "a11y", "tts"]).with(Acts(&[act("Launch", "terminal.herdr", &[], "launchHerdr")])).enabled(Flag("extras", "herdr")).varies(Flag("extras", "herdr"), "Launch the Herdr screen reader in a terminal.", "Not installed. Launch stays disabled until you install Herdr on Software."),
            ] },
        ],
    },
    Page {
        id: "sound",
        file: "SoundPage.qml",
        description: "Speaker and microphone levels. You can pick devices here too. If USB audio hangs, restart PipeWire from the bottom of the page.",
        groups: &[
            Group { title: "Output", detail: "These talk to PipeWire through omarchy audio. Volume is the default sink. Switching the output moves playing streams with it.", rows: &[
                line("Volume", "", "omarchy audio output volume", &["speaker", "headphone", "loudness", "gain"]).with(Calls(Key("audioOutputVolume"), Count(range(0.0, 100.0, 1.0, 0)), setter("setAudioOutputVolume", Given, ANY.row_same().range(0.0, 100.0).same().unless(Key("audioOutputMuted")), Setting("audioOutputVolume")))).enabled(Has(Key("audioSinks"))).varies(On(Key("audioOutputMuted")), "Output is muted. Drag the slider to unmute and set a level.", "How loud speakers and headphones are. This is the default output."),
                line("Mute output", "Silence speakers and headphones while this is on.", "omarchy audio output volume mute-toggle", &["mute", "silent"]).with(Calls(Key("audioOutputMuted"), Toggle, setter("toggleAudioOutputMute", Nothing, ANY, Write("audio.output.mute-toggle", None)))).enabled(Has(Key("audioSinks"))),
                line("Output device", "Where sound comes out. Playing audio follows this device.", "omarchy audio output set default", &["sink", "speaker", "hdmi", "headphone", "usb"]).with(Calls(Derived("audioSinks", "audioSink"), Pick(Devices(Key("audioSinks"))), setter("setAudioSink", Given, ANY.row_same().non_empty().same(), Device("audio.output.default", Key("audioSinks"))))).enabled(Has(Key("audioSinks"))),
                line("Next output", "Jump to the next output, the same way the bar audio control does.", "omarchy audio output switch", &["cycle", "switch"]).with(Buttons(&[btn("Switch", setter("switchAudioOutput", Nothing, ANY, Write("audio.output-switch", None))).when(Many(Key("audioSinks")))])).shown(Many(Key("audioSinks"))),
            ] },
            Group { title: "Input", detail: "Capture level and mute for the default microphone. Laptops with a mic-mute LED follow the mute toggle.", rows: &[
                line("Capture volume", "", "omarchy audio input volume", &["mic", "microphone", "capture", "gain"]).with(Calls(Key("audioInputVolume"), Count(range(0.0, 100.0, 1.0, 0)), setter("setAudioInputVolume", Given, ANY.row_same().range(0.0, 100.0).same().unless(Key("audioInputMuted")), Setting("audioInputVolume")))).enabled(Has(Key("audioSources"))).varies(On(Key("audioInputMuted")), "The microphone is muted. Drag the slider to unmute and set a level.", "How loud the default microphone is."),
                line("Mute microphone", "Cut microphone input while this is on.", "omarchy audio input mute", &["mic", "mute", "led"]).with(Calls(Key("audioInputMuted"), Toggle, setter("toggleAudioInputMute", Nothing, ANY, Write("audio.input.mute-toggle", None)))).enabled(Has(Key("audioSources"))),
                line("Input device", "Which microphone or other capture device is the default.", "omarchy audio input set default", &["source", "mic", "webcam", "seiren"]).with(Calls(Derived("audioSources", "audioSource"), Pick(Devices(Key("audioSources"))), setter("setAudioSource", Given, ANY.row_same().non_empty().same(), Device("audio.input.default", Key("audioSources"))))).enabled(Has(Key("audioSources"))),
            ] },
            Group { title: "Tuning", detail: "A filter Omarchy ships for this laptop's speakers when the built-in sound is thin or tinny.", rows: &[
                line("Speaker tuning", "A filter Omarchy ships for this laptop's speakers.", "omarchy audio tuning", &["eq", "dsp", "easyeffects", "laptop"]).with(Switch(Key("audioTuningOn"))).shown(On(Key("audioTuningMatch"))).enabled(On(Key("audioTuningMatch"))),
            ] },
            Group { title: "Recovery", detail: "Restarts PipeWire and WirePlumber. Use this when a USB headset or DAC stops showing up.", rows: &[
                line("Restart audio", "Restart PipeWire. That often brings a stuck USB headset or DAC back.", "omarchy restart audio", &["pipewire", "wireplumber", "usb"]).with(Acts(&[act("Restart", "restart", &[("what", "audio")], "restartAudio")])),
            ] },
            Group { title: "Advanced", detail: "Voxtype is Omarchy's dictation tool. Install pulls the package and a model. Hold F9 after it is ready.", rows: &[
                line("Dictation", "", "omarchy voxtype install", &["voxtype", "dictation", "speech", "whisper"]).varies(On(Key("voxtypeInstalled")), "Voxtype is installed. Hold F9 to dictate, or Super+Ctrl+X to toggle it.", "Install Voxtype and a speech model if you want hold-to-talk dictation.").with(Buttons(&[btn("Install…", setter("installVoxtype", Nothing, ANY, Job(task("voxtype.install", "voxtype-install", "The dictation install").sudo()))).when(All(&[Idle, Off(Key("voxtypeInstalled"))])).confirm("Install dictation", "Install Voxtype and a speech model, about 150 MB. You will be asked for a password.", "Install"), btn("Remove…", setter("removeVoxtype", Nothing, ANY.needs(Key("voxtypeInstalled")), Job(task("voxtype.remove", "voxtype-remove", "The dictation removal").sudo()))).when(All(&[Idle, On(Key("voxtypeInstalled"))])).confirm("Remove dictation", "Remove Voxtype from this machine.", "Remove")])),
            ] },
        ],
    },
    Page {
        id: "capture",
        file: "CapturePage.qml",
        description: "Screenshots, a recording, and a couple of readers for text and QR codes. Pictures go to your Pictures folder. Recordings go to Videos.",
        groups: &[
            Group { title: "Screenshot", detail: "Smart picks a region or a window. Copy and save writes a PNG under Pictures and puts it on the clipboard.", rows: &[
                line("Mode", "What the picker asks for.", "omarchy capture screenshot", &["screenshot", "region", "window", "fullscreen", "smart"]).with(Custom(CapturePick("mode"))),
                line("Where it goes", "Copy and save is the usual Omarchy path. Copy skips the file. Save skips the clipboard.", "omarchy capture screenshot", &["screenshot", "clipboard", "save", "copy"]).with(Custom(CapturePick("dest"))),
                line("Take screenshot", "The picker opens on the desktop.", "omarchy capture screenshot", &["screenshot", "capture", "grim"]).with(Custom(TakeShot)),
            ] },
            Group { title: "Recording", detail: "Start opens a region picker unless you ask for the whole screen. Stop finishes the file in Videos.", rows: &[
                line("Desktop audio", "Record what the speakers are playing.", "omarchy capture screenrecording --with-desktop-audio", &["record", "audio", "speakers"]).with(Custom(RecSwitch("recDesktopAudio"))),
                line("Microphone", "Record your voice with the picture.", "omarchy capture screenrecording --with-microphone-audio", &["record", "mic", "microphone"]).with(Custom(RecSwitch("recMic"))),
                line("Webcam", "A camera overlay on the recording.", "omarchy capture screenrecording --with-webcam", &["record", "webcam", "camera"]).with(Custom(RecSwitch("recWebcam"))),
                line("Webcam size", "How large the overlay starts.", "omarchy capture screenrecording --webcam-size", &["webcam", "size", "overlay"]).with(Custom(CapturePick("size"))),
                line("Whole screen", "Record the whole monitor, without a region picker.", "omarchy capture screenrecording --fullscreen", &["record", "fullscreen", "monitor"]).with(Custom(RecSwitch("recFullscreen"))),
                line("Resize webcam", "Step the overlay smaller or larger while a recording is up.", "omarchy capture webcam resize", &["webcam", "resize", "overlay"]).with(Custom(ResizeWebcam)),
            ] },
            Group { title: "Read", detail: "Both open a region picker. Text copies what OCR finds. QR copies the decoded link or payload.", rows: &[
                line("Text from screen", "OCR a region and copy the words.", "omarchy capture text", &["ocr", "text", "tesseract"]).with(Press("Read text…", setter("captureText", Nothing, ANY, Job(task("capture.text", "capture-text", "The text capture").interactive())))),
                line("QR code", "Decode a QR code from a region.", "omarchy capture qr", &["qr", "code", "scan"]).with(Press("Read QR…", setter("captureQr", Nothing, ANY, Job(task("capture.qr", "capture-qr", "The QR capture").interactive())))),
            ] },
            Group { title: "Save locations", detail: "Omarchy uses the XDG Pictures and Videos folders. Open the folder or copy the path. These are not changed here.", rows: &[
                line("Screenshots", "", "XDG_PICTURES_DIR", &["pictures", "folder", "screenshot", "path"]).with(Custom(SaveFolder("picturesDir"))),
                line("Recordings", "", "XDG_VIDEOS_DIR", &["videos", "folder", "recording", "path"]).with(Custom(SaveFolder("videosDir"))),
            ] },
        ],
    },
];
