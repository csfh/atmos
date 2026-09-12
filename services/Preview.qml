pragma Singleton
import QtQuick

// Preview mode: show a write instead of making it.
//
// Off by default and never persisted. A settings app that silently came back
// refusing to save would be indistinguishable from a bug. Each Atmos window
// is its own process, so this flag stays in that window's memory.
QtObject {
  property bool active: false
}
