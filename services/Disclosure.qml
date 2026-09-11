pragma Singleton
import QtQuick

// Simple / Everything.
//
// The two loudest complaints about desktop settings apps are opposites:
// macOS takes options away and buries what is left; KDE shows you everything
// at once and drowns you. Both are the same complaint about a missing fold,
// so add the fold rather than pick a side.
//
// Simple never removes anything. A folded row is still found by search and
// unfolded by it, so an option is always one keystroke away and never a dead
// end -- which is exactly the part macOS gets wrong, since there a buried
// setting is unreachable unless you already know its name.
//
// Not persisted. A settings app that silently came back in a reduced mode
// you had forgotten choosing would be indistinguishable from a bug.
QtObject {
  property bool simple: false
}
