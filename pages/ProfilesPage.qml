import QtQuick
import "../components"
import "../services"
import "../services/Profiles.js" as ProfilesJs

PrefsPage {
  id: root
  title: "Profiles"
  description: "A profile is a named bundle of Settings keys. Apply goes through the same writers as an Omafile. Coding, Gaming, and Battery ship with Atmos. Save your own as an Omafile."

  property var navigator: null

  readonly property var builtins: ProfilesJs.builtins()

  function applyProfile(id) {
    var profile = ProfilesJs.byId(id)
    if (!profile) return
    Omarchy.applyProfileValues(profile.values)
  }

  PrefsGroup {
    title: "Built-in"
    query: root.query
    detail: "Each bundle writes power, idle, notifications, and window look through Settings.commandFor."

    Repeater {
      model: root.builtins

      SettingRow {
        required property var modelData
        label: modelData && modelData.title ? modelData.title : "Profile"
        description: modelData && modelData.description ? modelData.description : ""
        hint: "Settings import"
        query: root.query
        keywords: ["profile", "coding", "gaming", "battery"]

        PrefsButton {
          text: "Apply"
          primary: true
          onClicked: root.applyProfile(modelData.id)
        }
      }
    }
  }

  PrefsGroup {
    title: "Your own"
    query: root.query
    detail: "Write an Omafile, then apply it later. An Omafile is a Markdown document of an Omarchy system, not a second prefs store."

    SettingRow {
      label: "Omafile"
      description: "Write this Omarchy system to a file, or apply one you already have."
      hint: "atmos export"
      query: root.query
      keywords: ["omafile", "import", "export", "markdown"]

      PrefsButton {
        text: "Open…"
        onClicked: {
          if (root.navigator && root.navigator.go) root.navigator.go("export")
        }
      }
    }
  }
}
