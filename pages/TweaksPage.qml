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

  function currentOn(tweak) {
    if (!tweak) return false
    if (tweak.kind === "hypr-input") {
      if (tweak.id === "accelFlat") return Omarchy.hyprInput.accelProfile === "flat"
      if (tweak.id === "naturalScroll") return Omarchy.hyprInput.naturalScroll === true
    }
    var tweaks = Omarchy.tweaks && typeof Omarchy.tweaks === "object" ? Omarchy.tweaks : {}
    if (tweak.id === "middlePaste") return tweaks.middlePaste === true
    if (tweak.id === "electronWayland") return tweaks.electronWayland !== false
    if (tweak.id === "forceZeroScaling") return tweaks.forceZeroScaling !== false
    if (tweak.id === "swappiness") return tweaks.swappiness === true
    return false
  }

  function setTweak(tweak, on) {
    if (!tweak) return
    if (tweak.kind === "hypr-input") {
      if (tweak.id === "accelFlat") Omarchy.writeHyprInput({ accelProfile: on ? "flat" : "" })
      else if (tweak.id === "naturalScroll") Omarchy.writeHyprInput({ naturalScroll: on })
      return
    }
    Omarchy.setTweak(tweak.id, on)
  }

  function resetTweak(tweak) {
    root.setTweak(tweak, tweak && tweak.offValue === true)
    if (tweak && tweak.offValue === "") root.setTweak(tweak, false)
    if (tweak && tweak.offValue === false) root.setTweak(tweak, false)
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
