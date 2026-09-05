import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "Sound"
  description: "Speaker and microphone levels. You can pick devices here too. If USB audio hangs, restart PipeWire from the bottom of the page."

  function deviceOptions(list) {
    var out = []
    if (!(list instanceof Array)) return out
    for (var i = 0; i < list.length; i++) {
      var item = list[i]
      if (!item || !item.name) continue
      out.push({ value: String(item.name), label: String(item.description || item.name) })
    }
    return out
  }

  PrefsGroup {
    title: "Output"
    query: root.query
    detail: "These talk to PipeWire through omarchy audio. Volume is the default sink. Switching the output moves playing streams with it."

    SettingRow {
      stretchControl: true
      label: "Volume"
      description: Omarchy.audioOutputMuted ? "Output is muted. Drag the slider to unmute and set a level." : "How loud speakers and headphones are. This is the default output."
      hint: "omarchy audio output volume"
      query: root.query
      keywords: ["speaker", "headphone", "loudness", "gain"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 100
        stepSize: 1
        live: true
        value: Omarchy.audioOutputVolume
        valueText: Omarchy.audioOutputVolume + "%"
        enabled: Omarchy.audioSinks.length > 0
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.audioOutputVolume || Omarchy.audioOutputMuted)
            Omarchy.setAudioOutputVolume(next)
        }
      }
    }

    SettingRow {
      label: "Mute output"
      description: "Silence speakers and headphones while this is on."
      hint: "omarchy audio output volume mute-toggle"
      query: root.query
      keywords: ["mute", "silent"]

      PrefsToggle {
        checked: Omarchy.audioOutputMuted
        enabled: Omarchy.audioSinks.length > 0
        onToggled: Omarchy.toggleAudioOutputMute()
      }
    }

    SettingRow {
      label: "Output device"
      description: "Where sound comes out. Playing audio follows this device."
      hint: "omarchy audio output set default"
      query: root.query
      keywords: ["sink", "speaker", "hdmi", "headphone", "usb"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.audioSink
        options: root.deviceOptions(Omarchy.audioSinks)
        enabled: Omarchy.audioSinks.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.audioSink) Omarchy.setAudioSink(value)
        }
      }
    }

    SettingRow {
      available: Omarchy.audioSinks.length > 1
      label: "Next output"
      description: "Jump to the next output, the same way the bar audio control does."
      hint: "omarchy audio output switch"
      query: root.query
      keywords: ["cycle", "switch"]

      PrefsButton {
        text: "Switch"
        enabled: Omarchy.audioSinks.length > 1
        onClicked: Omarchy.switchAudioOutput()
      }
    }
  }

  PrefsGroup {
    title: "Input"
    query: root.query
    detail: "Capture level and mute for the default microphone. Laptops with a mic-mute LED follow the mute toggle."

    SettingRow {
      stretchControl: true
      label: "Capture volume"
      description: Omarchy.audioInputMuted ? "The microphone is muted. Drag the slider to unmute and set a level." : "How loud the default microphone is."
      hint: "wpctl set-volume @DEFAULT_AUDIO_SOURCE@"
      query: root.query
      keywords: ["mic", "microphone", "capture", "gain"]

      PrefsSlider {
        width: parent.width
        from: 0
        to: 100
        stepSize: 1
        live: true
        value: Omarchy.audioInputVolume
        valueText: Omarchy.audioInputVolume + "%"
        enabled: Omarchy.audioSources.length > 0
        onChanged: function(value) {
          var next = Math.round(value)
          if (next !== Omarchy.audioInputVolume || Omarchy.audioInputMuted)
            Omarchy.setAudioInputVolume(next)
        }
      }
    }

    SettingRow {
      label: "Mute microphone"
      description: "Cut microphone input while this is on."
      hint: "omarchy audio input mute"
      query: root.query
      keywords: ["mic", "mute", "led"]

      PrefsToggle {
        checked: Omarchy.audioInputMuted
        enabled: Omarchy.audioSources.length > 0
        onToggled: Omarchy.toggleAudioInputMute()
      }
    }

    SettingRow {
      label: "Input device"
      description: "Which microphone or other capture device is the default."
      hint: "omarchy audio input set default"
      query: root.query
      keywords: ["source", "mic", "webcam", "seiren"]

      PrefsSelect {
        implicitWidth: 280
        value: Omarchy.audioSource
        options: root.deviceOptions(Omarchy.audioSources)
        enabled: Omarchy.audioSources.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.audioSource) Omarchy.setAudioSource(value)
        }
      }
    }
  }

  PrefsGroup {
    title: "Tuning"
    query: Omarchy.audioTuningMatch ? root.query : "."
    detail: "A filter Omarchy ships for this laptop's speakers when the built-in sound is thin or tinny."

    SettingRow {
      available: Omarchy.audioTuningMatch
      label: "Speaker tuning"
      description: "A filter Omarchy ships for this laptop's speakers."
      hint: "omarchy audio tuning"
      query: root.query
      keywords: ["eq", "dsp", "easyeffects", "laptop"]

      PrefsToggle {
        checked: Omarchy.audioTuningOn
        enabled: Omarchy.audioTuningMatch
        onToggled: Omarchy.setAudioTuning(!Omarchy.audioTuningOn)
      }
    }
  }

  PrefsGroup {
    title: "Recovery"
    query: root.query
    detail: "Restarts PipeWire and WirePlumber. Use this when a USB headset or DAC stops showing up."

    SettingRow {
      label: "Restart audio"
      description: "Restart PipeWire. That often brings a stuck USB headset or DAC back."
      hint: "omarchy restart audio"
      query: root.query
      keywords: ["pipewire", "wireplumber", "usb"]

      PrefsButton {
        text: "Restart"
        onClicked: Omarchy.restartAudio()
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Voxtype is Omarchy's dictation tool. Install pulls the package and a model. Hold F9 after it is ready."

    SettingRow {
      label: "Dictation"
      description: Omarchy.voxtypeInstalled
        ? "Voxtype is installed. Hold F9 to dictate, or Super+Ctrl+X to toggle it."
        : "Install Voxtype and a speech model if you want hold-to-talk dictation."
      hint: "omarchy voxtype install"
      query: root.query
      keywords: ["voxtype", "dictation", "speech", "whisper"]

      Row {
        spacing: Theme.space
        PrefsButton {
          visible: !Omarchy.voxtypeInstalled
          text: "Install…"
          primary: true
          enabled: !Omarchy.jobBusy && !Omarchy.voxtypeInstalled
          onClicked: voxtypeInstallConfirm.ask()
        }
        PrefsButton {
          visible: Omarchy.voxtypeInstalled
          text: "Remove…"
          danger: true
          enabled: !Omarchy.jobBusy && Omarchy.voxtypeInstalled
          onClicked: voxtypeRemoveConfirm.ask()
        }
      }
    }
  }

  PrefsConfirm {
    id: voxtypeInstallConfirm
    title: "Install dictation"
    message: "Install Voxtype and a speech model, about 150 MB. You will be asked for a password."
    confirmText: "Install"
    onConfirmed: Omarchy.installVoxtype()
  }

  PrefsConfirm {
    id: voxtypeRemoveConfirm
    title: "Remove dictation"
    message: "Remove Voxtype from this machine."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeVoxtype()
  }

  Component.onCompleted: {
    voxtypeInstallConfirm.parent = root.prefsOverlay
    voxtypeRemoveConfirm.parent = root.prefsOverlay
  }
}
