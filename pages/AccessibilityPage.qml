import QtQuick
import "../components"
import "../services"
import "rows"

PrefsPage {
  id: root
  title: "Accessibility"
  description: "Motion, type size, the pointer, and touch. These use the same writers as Windows, Appearance, and Displays."

  PrefsGroup {
    title: "Motion and type"
    query: root.query
    detail: "Animations write the look sentinel. Text size is the same slider as Appearance."

    AnimationsRow { query: root.query }

    TextSizeRow { query: root.query }
  }

  PrefsGroup {
    title: "Pointer"
    query: root.query
    detail: "Cursor hide is the same look key as Windows. Size is a new look-sentinel field."

    HideCursorRow { query: root.query }

    CursorSizeRow { query: root.query }
  }

  PrefsGroup {
    title: "Touch"
    query: root.query
    detail: "This is the same touchscreen switch as Displays."

    TouchscreenRow { query: root.query }
  }

  PrefsGroup {
    title: "Tools"
    query: root.query
    detail: "Herdr is a screen reader Omarchy can launch in a terminal when the package is present."

    SettingRow {
      label: "Herdr"
      description: Omarchy.extras && Omarchy.extras.herdr === true
        ? "Launch the Herdr screen reader in a terminal."
        : "Not installed. Launch stays disabled until you install Herdr on Software."
      hint: "omarchy launch terminal herdr"
      query: root.query
      keywords: ["herdr", "screen reader", "a11y", "tts"]

      PrefsButton {
        text: "Launch"
        enabled: Omarchy.extras && Omarchy.extras.herdr === true
        onClicked: Omarchy.launchHerdr()
      }
    }
  }
}
