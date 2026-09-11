import QtQuick
import "../../components"
import "../../services"
import "../../services/History.js" as HistoryJs

// What Atmos changed, and what it is about to change.
//
// The question this answers is the one you ask at the worst moment: "what
// did I touch yesterday that broke my touchpad." Atmos is unusually well
// placed to answer it, because every mutation already funnels through one
// function and the import planner already knows how to describe a change
// before it happens.
//
// Preview is the same machinery pointed forwards. With it on, a write is
// held and shown rather than made, so you can flip a control and read the
// command it would run. Nothing held ever executes on its own -- Apply is
// the only path, and Discard is one click.

PrefsPage {
  id: root
  title: "History"
  description: "Every change Atmos made, newest first. Turn on Preview to see a change before it happens instead of after."

  readonly property var counts: HistoryJs.countsBySource(Omarchy.changeHistory)

  PrefsGroup {
    title: "Preview"
    query: root.query
    detail: "While Preview is on, Atmos shows you what a control would run and does not run it. Apply releases everything held; Discard throws it away."

    PrefsRow {
      label: "Preview changes"
      description: Preview.active
        ? "On. Controls will not write. Anything you change is held below."
        : "Off. Controls write immediately, as normal."
      query: root.query
      keywords: ["dry", "run", "diff", "safe", "preview"]

      PrefsToggle {
        checked: Preview.active
        onToggled: function (value) {
          Preview.active = value
          if (!value) Omarchy.discardHeld()
        }
      }
    }

    PrefsRow {
      label: "Held"
      description: Omarchy.heldChanges.length === 1
        ? "1 change is waiting. Nothing has been written."
        : Omarchy.heldChanges.length + " changes are waiting. Nothing has been written."
      query: root.query
      available: Omarchy.heldChanges.length > 0
      stretchControl: true

      Row {
        spacing: Theme.space
        PrefsButton {
          text: "Apply"
          primary: true
          onClicked: Omarchy.applyHeld()
        }
        PrefsButton {
          text: "Discard"
          onClicked: Omarchy.discardHeld()
        }
      }
    }

    Repeater {
      model: Omarchy.heldChanges

      delegate: PrefsRow {
        required property var modelData
        label: modelData && modelData.file ? modelData.file : (modelData ? modelData.key : "")
        description: modelData ? modelData.text : ""
        query: root.query
        valueText: "held"
      }
    }
  }

  PrefsGroup {
    title: "Changes"
    query: root.query
    detail: "Recorded in memory for this session. Atmos keeps the most recent 200."

    PrefsRow {
      label: "Nothing yet"
      description: "Change something and it will show up here with the command it ran."
      query: root.query
      available: Omarchy.changeHistory.length === 0
    }

    PrefsRow {
      label: "By you"
      description: "Changes made from a control in this window."
      query: root.query
      available: Omarchy.changeHistory.length > 0
      valueText: String(HistoryJs.countFor(root.counts, "you"))
    }

    Repeater {
      model: Omarchy.changeHistory

      delegate: PrefsRow {
        required property var modelData
        label: modelData && modelData.file
          ? modelData.file
          : (modelData && modelData.key ? modelData.key : "change")
        description: modelData ? modelData.text : ""
        query: root.query
        valueText: modelData ? HistoryJs.relativeTime(modelData.at) : ""
      }
    }
  }
}
