import QtQuick
import "../components"
import "../services"
import "../services/Favorites.js" as FavJs

PrefsPage {
  id: root
  title: "Favorites"
  description: "Settings you starred. Open jumps to the hub. The star on a row adds or removes it."
  hubId: "favorites"

  property var navigator: null

  readonly property var groups: FavJs.groups(Omarchy.favoriteItems)
  readonly property bool hasFavorites: root.groups.length > 0

  Repeater {
    model: root.groups

    PrefsGroup {
      required property var modelData
      title: (modelData && (modelData.title || modelData.hub)) || ""
      query: root.query
      detail: "Starred rows from this hub."

      Repeater {
        model: modelData && modelData.items ? modelData.items : []

        SettingRow {
          required property var modelData
          favoriteHub: (modelData && modelData.hub) || ""
          label: (modelData && modelData.label) || ""
          description: (modelData && modelData.description) || ""
          query: root.query
          keywords: ["favorite", "star", "pin"]

          PrefsButton {
            text: "Open…"
            onClicked: {
              if (root.navigator && root.navigator.go)
                root.navigator.go((modelData && modelData.hub) || "")
            }
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "Favorites"
    query: root.hasFavorites ? "." : root.query
    detail: "Star a setting on any hub. It shows up here."

    SettingRow {
      catalog: false
      label: "Favorites"
      description: "No favorites yet. Star a setting on any page."
      query: root.query
      keywords: ["favorite", "star", "empty"]
    }
  }
}
