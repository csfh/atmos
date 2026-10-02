import QtQuick
import "../services"

// An empty or unavailable state. A SettingRow so search, Simple and the
// group split logic treat it like any other row, with a muted icon in the
// lead slot so it reads as "nothing here" rather than as a setting. Put a
// PrefsButton inside for a way out.
SettingRow {
  id: root

  property string icon: "information-line"

  sectionHelp: false
  keywords: ["empty"]

  leading: PrefsIcon {
    name: root.icon
    size: Theme.navIconSize
    color: Theme.muted
  }
}
