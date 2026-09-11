pragma Singleton
import QtQuick

// Preview mode: show a write instead of making it.
//
// Off by default and never persisted. A settings app that silently came back
// refusing to save would be indistinguishable from a bug.
QtObject {
  property bool active: false
}
