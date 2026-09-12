pragma Singleton
import QtQuick
import "Disclosure.js" as DisclosureJs

// Simple / Everything. Session-only; see Disclosure.js for the rules.
QtObject {
  id: root

  property bool simple: false
  property string revealedHub: ""
  property string revealedLabel: ""
  property bool pending: false

  function snapshot() {
    return {
      simple: root.simple,
      revealedHub: root.revealedHub,
      revealedLabel: root.revealedLabel,
      pending: root.pending
    }
  }

  function apply(next) {
    root.simple = next.simple
    root.revealedHub = next.revealedHub
    root.revealedLabel = next.revealedLabel
    root.pending = next.pending
  }

  function revealFromSearch(hub, label) {
    apply(DisclosureJs.revealFromSearch(snapshot(), hub, label))
  }

  function leaveHub(hub) {
    apply(DisclosureJs.leaveHub(snapshot(), hub))
  }

  function finishReveal(hub) {
    apply(DisclosureJs.finishReveal(snapshot(), hub))
  }
}
