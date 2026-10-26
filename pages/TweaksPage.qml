import QtQuick
import "../components"
import "../services"
import "../services/Tweaks.js" as TweaksJs

PrefsPage {
  id: root
  hubId: "tweaks"
  title: "Tweaks"
  description: "Settings that do not deserve their own sidebar entry. Each row says what it writes. Reset puts that one tweak back."

  readonly property var groups: TweaksJs.groupedCatalog()

  // The logic is in Tweaks.js, which the Rust window's Tweaks page is held to
  // (frontend/tests/tweaks_parity.rs).
  function currentOn(tweak) {
    return TweaksJs.currentOn(tweak, Omarchy.hyprInput, Omarchy.tweaks)
  }

  function setTweak(tweak, on) {
    root.writeTweak(tweak, TweaksJs.writeFor(tweak, on))
  }

  // Reset writes the tweak's default, as Omarchy sets it (TweaksJs.resetValue), once.
  function resetTweak(tweak) {
    root.writeTweak(tweak, TweaksJs.resetWrite(tweak))
  }

  function writeTweak(tweak, write) {
    if (!write) return
    if (tweak.kind === "hypr-input") {
      var patch = {}
      patch[write.field] = write.value
      Omarchy.writeHyprInput(patch)
      return
    }
    Omarchy.setTweak(tweak.id, write.value)
  }

  Repeater {
    model: root.groups

    PrefsGroup {
      required property var modelData
      readonly property var group: modelData
      title: group && group.group ? group.group : "Tweak"
      query: root.query
      detail: "Each row names the file it writes."

      Repeater {
        model: group && group.items ? group.items : []

        SettingRow {
          required property var modelData
          label: modelData && modelData.label ? modelData.label : "Tweak"
          description: modelData && modelData.description ? modelData.description : ""
          hint: modelData && modelData.modifies ? modelData.modifies : ""
          query: root.query
          keywords: ["tweak", "reset"]

          Row {
            spacing: Theme.space
            PrefsToggle {
              checked: root.currentOn(modelData)
              onToggled: root.setTweak(modelData, !root.currentOn(modelData))
            }
            PrefsButton {
              text: "Reset"
              onClicked: root.resetTweak(modelData)
            }
          }
        }
      }
    }
  }
}
