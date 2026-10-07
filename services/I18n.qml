pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "I18n.js" as I18nJs

QtObject {
  id: root

  readonly property string locale: I18nJs.effectiveLocale({
    ATMOS_LANG: Quickshell.env("ATMOS_LANG"),
    LANGUAGE: Quickshell.env("LANGUAGE"),
    LC_ALL: Quickshell.env("LC_ALL"),
    LC_MESSAGES: Quickshell.env("LC_MESSAGES"),
    LANG: Quickshell.env("LANG")
  }, Omarchy.locale)
  readonly property bool rtl: I18nJs.isRtl(locale)
  property int revision: 0
  property var catalog: ({})

  function tr(source, args) {
    var currentRevision = root.revision;
    return I18nJs.translate(root.catalog, root.locale, source, args);
  }

  function trPlural(forms, count, args) {
    return I18nJs.translatePlural(root.catalog, root.locale, forms, count, args);
  }

  property FileView catalogFile: FileView {
    path: Quickshell.shellDir + "/i18n.json"
    watchChanges: true
    onFileChanged: reload()
    onLoaded: root.reloadCatalog()
    Component.onCompleted: reload()
  }

  function reloadCatalog() {
    var parsed = I18nJs.catalogFromText(catalogFile.text(), root.catalog);
    if (parsed === root.catalog) return;
    root.catalog = parsed;
    root.revision += 1;
  }
}
