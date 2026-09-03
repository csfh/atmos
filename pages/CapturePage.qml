import QtQuick
import "../components"
import "../services"

PrefsPage {
  id: root
  title: "Capture"
  description: "Screenshots, a recording, and a couple of readers for text and QR codes. Pictures go to your Pictures folder. Recordings go to Videos."

  readonly property var screenshotModes: [
    { value: "smart", label: "Smart" },
    { value: "region", label: "Region" },
    { value: "windows", label: "Window" },
    { value: "fullscreen", label: "Fullscreen" }
  ]
  readonly property var screenshotDests: [
    { value: "slurp", label: "Copy and save" },
    { value: "copy", label: "Copy" },
    { value: "save", label: "Save" }
  ]
  readonly property var webcamSizes: [
    { value: "small", label: "Small" },
    { value: "medium", label: "Medium" },
    { value: "large", label: "Large" }
  ]

  property string shotMode: "smart"
  property string shotDest: "slurp"
  property bool recDesktopAudio: true
  property bool recMic: false
  property bool recWebcam: false
  property string recWebcamSize: "medium"
  property bool recFullscreen: false

  PrefsGroup {
    title: "Screenshot"
    query: root.query
    detail: "Smart picks a region or a window. Copy and save writes a PNG under Pictures and puts it on the clipboard."
    hint: "omarchy capture screenshot"

    PrefsRow {
      label: "Mode"
      description: "What the picker asks for."
      hint: "omarchy capture screenshot"
      query: root.query
      keywords: ["screenshot", "region", "window", "fullscreen", "smart"]

      PrefsSelect {
        value: root.shotMode
        options: root.screenshotModes
        onChanged: function(value) { root.shotMode = value }
      }
    }

    PrefsRow {
      label: "Where it goes"
      description: "Copy and save is the usual Omarchy path. Copy skips the file. Save skips the clipboard."
      hint: "omarchy capture screenshot"
      query: root.query
      keywords: ["screenshot", "clipboard", "save", "copy"]

      PrefsSelect {
        value: root.shotDest
        options: root.screenshotDests
        onChanged: function(value) { root.shotDest = value }
      }
    }

    PrefsRow {
      label: "Take screenshot"
      description: "The picker opens on the desktop."
      hint: "omarchy capture screenshot"
      query: root.query
      keywords: ["screenshot", "capture", "grim"]

      PrefsButton {
        text: "Capture"
        primary: true
        onClicked: Omarchy.captureScreenshot(root.shotMode, root.shotDest)
      }
    }
  }

  PrefsGroup {
    title: "Recording"
    query: root.query
    detail: "Start opens a region picker unless you ask for the whole screen. Stop finishes the file in Videos."
    hint: "omarchy capture screenrecording"

    PrefsRow {
      label: "Desktop audio"
      description: "Record what the speakers are playing."
      hint: "omarchy capture screenrecording --with-desktop-audio"
      query: root.query
      keywords: ["record", "audio", "speakers"]

      PrefsToggle {
        checked: root.recDesktopAudio
        enabled: !Omarchy.recordingActive
        onToggled: root.recDesktopAudio = !root.recDesktopAudio
      }
    }

    PrefsRow {
      label: "Microphone"
      description: "Record your voice with the picture."
      hint: "omarchy capture screenrecording --with-microphone-audio"
      query: root.query
      keywords: ["record", "mic", "microphone"]

      PrefsToggle {
        checked: root.recMic
        enabled: !Omarchy.recordingActive
        onToggled: root.recMic = !root.recMic
      }
    }

    PrefsRow {
      label: "Webcam"
      description: "Put a camera overlay on the recording."
      hint: "omarchy capture screenrecording --with-webcam"
      query: root.query
      keywords: ["record", "webcam", "camera"]

      PrefsToggle {
        checked: root.recWebcam
        enabled: !Omarchy.recordingActive
        onToggled: root.recWebcam = !root.recWebcam
      }
    }

    PrefsRow {
      available: root.recWebcam
      label: "Webcam size"
      description: "How large the overlay starts."
      hint: "omarchy capture screenrecording --webcam-size"
      query: root.query
      keywords: ["webcam", "size", "overlay"]

      PrefsSelect {
        value: root.recWebcamSize
        options: root.webcamSizes
        enabled: !Omarchy.recordingActive
        onChanged: function(value) { root.recWebcamSize = value }
      }
    }

    PrefsRow {
      label: "Whole screen"
      description: "Skip the region picker and record the monitor."
      hint: "omarchy capture screenrecording --fullscreen"
      query: root.query
      keywords: ["record", "fullscreen", "monitor"]

      PrefsToggle {
        checked: root.recFullscreen
        enabled: !Omarchy.recordingActive
        onToggled: root.recFullscreen = !root.recFullscreen
      }
    }

    PrefsRow {
      label: Omarchy.recordingActive ? "Recording" : "Start recording"
      description: Omarchy.recordingActive
        ? "gpu-screen-recorder is running. Stop writes the file under Videos."
        : "Start a recording with the switches above."
      hint: "omarchy capture screenrecording"
      query: root.query
      keywords: ["record", "start", "stop", "video"]

      Row {
        spacing: 8
        PrefsButton {
          visible: !Omarchy.recordingActive
          text: "Start"
          primary: true
          enabled: !Omarchy.recordingActive
          onClicked: Omarchy.startScreenrecording(root.recDesktopAudio, root.recMic, root.recWebcam, root.recWebcamSize, root.recFullscreen)
        }
        PrefsButton {
          visible: Omarchy.recordingActive
          text: "Stop"
          danger: true
          enabled: Omarchy.recordingActive
          onClicked: Omarchy.stopScreenrecording()
        }
      }
    }

    PrefsRow {
      available: Omarchy.webcamOverlay
      label: "Resize webcam"
      description: "Step the overlay smaller or larger while a recording is up."
      hint: "omarchy capture webcam resize"
      query: root.query
      keywords: ["webcam", "resize", "overlay"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Smaller"
          enabled: Omarchy.webcamOverlay
          onClicked: Omarchy.resizeWebcam("smaller")
        }
        PrefsButton {
          text: "Larger"
          enabled: Omarchy.webcamOverlay
          onClicked: Omarchy.resizeWebcam("larger")
        }
        PrefsButton {
          text: "Reset"
          enabled: Omarchy.webcamOverlay
          onClicked: Omarchy.resizeWebcam("reset")
        }
      }
    }
  }

  PrefsGroup {
    title: "Read"
    query: root.query
    detail: "Both open a region picker. Text copies what OCR finds. QR copies the decoded link or payload."

    PrefsRow {
      label: "Text from screen"
      description: "OCR a region and copy the words."
      hint: "omarchy capture text"
      query: root.query
      keywords: ["ocr", "text", "tesseract"]

      PrefsButton {
        text: "Read text"
        onClicked: Omarchy.captureText()
      }
    }

    PrefsRow {
      label: "QR code"
      description: "Decode a QR code from a region."
      hint: "omarchy capture qr"
      query: root.query
      keywords: ["qr", "code", "scan"]

      PrefsButton {
        text: "Read QR"
        onClicked: Omarchy.captureQr()
      }
    }
  }

  PrefsGroup {
    title: "Save locations"
    query: root.query
    detail: "Omarchy uses the XDG Pictures and Videos folders. Open the folder or copy the path. These are not changed here."

    PrefsRow {
      label: "Screenshots"
      description: Omarchy.picturesDir.length
        ? Omarchy.picturesDir
        : "XDG Pictures is not set. Open and Copy stay disabled until that folder exists."
      hint: "XDG_PICTURES_DIR"
      query: root.query
      keywords: ["pictures", "folder", "screenshot", "path"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Open"
          enabled: Omarchy.picturesDir.length > 0
          onClicked: Omarchy.openUserDir(Omarchy.picturesDir)
        }
        PrefsButton {
          text: "Copy"
          enabled: Omarchy.picturesDir.length > 0
          onClicked: Omarchy.copyText(Omarchy.picturesDir)
        }
      }
    }

    PrefsRow {
      label: "Recordings"
      description: Omarchy.videosDir.length
        ? Omarchy.videosDir
        : "XDG Videos is not set. Open and Copy stay disabled until that folder exists."
      hint: "XDG_VIDEOS_DIR"
      query: root.query
      keywords: ["videos", "folder", "recording", "path"]

      Row {
        spacing: 8
        PrefsButton {
          text: "Open"
          enabled: Omarchy.videosDir.length > 0
          onClicked: Omarchy.openUserDir(Omarchy.videosDir)
        }
        PrefsButton {
          text: "Copy"
          enabled: Omarchy.videosDir.length > 0
          onClicked: Omarchy.copyText(Omarchy.videosDir)
        }
      }
    }
  }
}
