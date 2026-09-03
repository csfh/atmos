import QtQuick
import Quickshell.Io
import Quickshell.Networking
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Wi-Fi"
  description: "Join a nearby network, pin the band, or share the one you are on with a QR code."

  readonly property var bandLabels: ({
    "auto": "Auto",
    "2.4": "2.4 GHz",
    "5": "5 GHz",
    "6": "6 GHz"
  })
  readonly property var bandOptions: {
    var out = []
    var list = Omarchy.wifiBands || []
    for (var i = 0; i < list.length; i++) {
      var id = String(list[i])
      out.push({ value: id, label: root.bandLabels[id] || id })
    }
    return out
  }

  property var scannerDevice: null
  property var wifiRows: []
  property string passwordSsid: ""
  property string actionSsid: ""
  property string actionKind: ""
  property string wifiError: ""
  property var qrRows: []
  property int qrSize: 0
  property string qrSsid: ""
  property string qrError: ""
  property bool qrLoading: false

  function wifiDevice() {
    var list = Networking.devices ? Networking.devices.values : []
    for (var i = 0; i < list.length; i++) {
      if (list[i] && list[i].type === DeviceType.Wifi) return list[i]
    }
    return null
  }

  function securityKind(sec) {
    if (sec === WifiSecurityType.Open) return "open"
    if (sec === WifiSecurityType.Owe) return "owe"
    if (sec === WifiSecurityType.Wpa2Eap || sec === WifiSecurityType.WpaEap) return "enterprise"
    return "psk"
  }

  function rebuildWifi() {
    var device = wifiDevice()
    var objects = device && device.networks ? device.networks.values : []
    var rows = []
    for (var i = 0; i < objects.length; i++) {
      var net = objects[i]
      if (!net || !net.name) continue
      rows.push(RichUi.wifiRow(net.name, (net.signalStrength || 0) * 100, securityKind(net.security), net.connected, net.known))
    }
    wifiRows = RichUi.sortWifiRows(rows)
  }

  function networkForSsid(ssid) {
    var device = wifiDevice()
    var objects = device && device.networks ? device.networks.values : []
    for (var i = 0; i < objects.length; i++) {
      if (objects[i] && objects[i].name === ssid) return objects[i]
    }
    return null
  }

  function setScanner(on) {
    var next = on ? wifiDevice() : null
    if (scannerDevice && scannerDevice !== next)
      scannerDevice.scannerEnabled = false
    scannerDevice = next
    if (scannerDevice)
      scannerDevice.scannerEnabled = true
  }

  function connectWifi(row) {
    if (!row || actionKind !== "") return
    var kind = row.securityKind
    if (RichUi.isEnterprise(kind)) {
      passwordSsid = row.ssid
      return
    }
    if (RichUi.requiresCredentials(kind) && !row.known) {
      passwordSsid = row.ssid
      return
    }
    var net = networkForSsid(row.ssid)
    if (!net) return
    actionSsid = row.ssid
    actionKind = "connect"
    wifiError = ""
    net.connect()
  }

  function submitWifiSecrets(row, secret, identity) {
    if (!row) return
    secret = String(secret || "")
    if (!secret) return
    var net = networkForSsid(row.ssid)
    actionSsid = row.ssid
    actionKind = "connect"
    wifiError = ""
    if (RichUi.isEnterprise(row.securityKind)) {
      identity = String(identity || "")
      if (!identity) return
      enterpriseProc.secret = secret
      enterpriseProc.command = ["bash", Omarchy.enterpriseWifiScript, row.ssid, identity]
      enterpriseProc.running = true
      return
    }
    if (net) net.connectWithPsk(secret)
    passwordSsid = ""
  }

  function startQr() {
    qrLoading = true
    qrError = ""
    qrRows = []
    qrSize = 0
    qrSsid = ""
    if (Omarchy.wifiIface)
      qrProc.command = ["omarchy", "network", "qr", "--meta", Omarchy.wifiIface]
    else
      qrProc.command = ["omarchy", "network", "qr", "--meta"]
    qrProc.running = true
  }

  Component.onCompleted: {
    setScanner(Omarchy.wifiRadio)
    rebuildWifi()
  }
  Component.onDestruction: setScanner(false)

  Timer {
    interval: 900
    running: root.visible && Omarchy.wifiRadio
    repeat: true
    onTriggered: root.rebuildWifi()
  }

  Process {
    id: qrProc
    stdout: StdioCollector {
      id: qrOut
      waitForEnd: true
    }
    stderr: StdioCollector {
      id: qrErr
      waitForEnd: true
    }
    onExited: function(code) {
      root.qrLoading = false
      if (code !== 0) {
        root.qrError = String(qrErr.text || "Could not build a QR code").replace(/^\s+|\s+$/g, "")
        return
      }
      var parsed = RichUi.parseQrOutput(qrOut.text)
      if (!parsed.ok) {
        root.qrError = parsed.error
        return
      }
      root.qrSsid = parsed.ssid
      root.qrRows = parsed.rows
      root.qrSize = parsed.size
    }
  }

  Process {
    id: enterpriseProc
    property string secret: ""
    stdinEnabled: true
    onStarted: {
      write(secret + "\n")
      secret = ""
    }
    onExited: function(code) {
      root.actionKind = ""
      root.actionSsid = ""
      root.passwordSsid = ""
      if (code !== 0) root.wifiError = "Enterprise join failed"
      Omarchy.refresh()
    }
  }

  PrefsConfirm {
    id: forgetWifiConfirm
    title: "Forget network"
    message: "Forget this saved Wi-Fi network? You will need the password again the next time you join."
    confirmText: "Forget"
    onConfirmed: {
      var net = root.networkForSsid(forgetWifiConfirm.payload)
      if (net) net.forget()
    }
    property string payload: ""
  }

  PrefsGroup {
    title: "Adapter"
    query: root.query
    detail: "Turns the Wi-Fi radio on or off through NetworkManager. Off is like airplane mode for Wi-Fi only."
    hint: "nmcli radio wifi"

    PrefsRow {
      label: "Wi-Fi radio"
      description: !Omarchy.wifiHw
        ? "NetworkManager does not see a Wi-Fi adapter. The switch stays off until the kernel exposes one (driver or rfkill)."
        : (Omarchy.wifiRadio
          ? (Omarchy.netKind === "wifi" && Omarchy.netSsid.length
            ? "Connected to " + Omarchy.netSsid + ". Nearby networks stay below."
            : "Scanning for access points below.")
          : "The radio is stopped. Turn it on to scan nearby networks.")
      hint: "nmcli radio wifi"
      query: root.query
      keywords: ["wlan", "rfkill", "airplane", "radio"]

      PrefsToggle {
        checked: Omarchy.wifiHw && Omarchy.wifiRadio
        enabled: Omarchy.wifiHw
        onToggled: Omarchy.setWifiRadio(!Omarchy.wifiRadio)
      }
    }
  }

  PrefsGroup {
    title: "Networks"
    query: root.query
    detail: "Nearby and saved access points from a NetworkManager scan. The list refreshes while this page is open. Join a known network, or type a password. Enterprise networks also ask for an identity. Forget drops a saved connection."
    hint: "nmcli"

    PrefsRow {
      available: root.wifiRows.length === 0
      sectionHelp: false
      label: "Nearby networks"
      description: !Omarchy.wifiHw
        ? "No adapter. Refresh stays disabled until NetworkManager sees Wi-Fi hardware."
        : (Omarchy.wifiRadio
          ? "Looking for access points nearby."
          : "Turn the radio on above, then refresh to scan.")
      query: root.query
      keywords: ["scan", "ssid"]

      PrefsButton {
        text: "Refresh"
        enabled: Omarchy.wifiHw && Omarchy.wifiRadio
        onClicked: {
          Omarchy.refresh()
          root.rebuildWifi()
        }
      }
    }

    Repeater {
      model: root.wifiRows

      PrefsRow {
        required property var modelData
        available: Omarchy.wifiRadio
        sectionHelp: false
        label: modelData && modelData.ssid ? modelData.ssid : "Network"
        description: {
          var bits = []
          if (modelData && modelData.connected) bits.push("Connected")
          else if (modelData && modelData.known) bits.push("Saved")
          if (modelData) bits.push("Signal " + modelData.signal + "%")
          if (modelData && modelData.securityKind === "open") bits.push("Open")
          else if (modelData && modelData.securityKind === "enterprise") bits.push("Enterprise")
          return bits.join(". ") + "."
        }
        hint: "nmcli"
        query: root.query
        keywords: ["ssid", "scan", "join", "psk"]

        Column {
          spacing: 8

          Row {
            spacing: 8
            PrefsButton {
              text: modelData && modelData.connected ? "Disconnect" : (root.actionSsid === modelData.ssid ? "Joining…" : "Join")
              primary: !(modelData && modelData.connected)
              enabled: root.actionKind === "" && modelData
              onClicked: {
                if (modelData.connected) {
                  var net = root.networkForSsid(modelData.ssid)
                  if (net) net.disconnect()
                } else {
                  root.connectWifi(modelData)
                }
              }
            }
            PrefsButton {
              visible: !!(modelData && modelData.known && !modelData.connected)
              text: "Forget"
              danger: true
              enabled: modelData
              onClicked: {
                forgetWifiConfirm.payload = modelData.ssid
                forgetWifiConfirm.ask()
              }
            }
          }

          Column {
            visible: root.passwordSsid === (modelData ? modelData.ssid : "")
            spacing: 8

            PrefsField {
              id: identityField
              visible: modelData && RichUi.isEnterprise(modelData.securityKind)
              placeholder: "Identity"
              enabled: root.actionKind === ""
            }

            PrefsPassword {
              id: rowPassword
              placeholder: "Password"
              enabled: root.actionKind === ""
              onSubmitted: function(value) {
                root.submitWifiSecrets(modelData, value, identityField.currentText())
              }
            }

            PrefsButton {
              text: "Connect"
              enabled: root.actionKind === ""
              onClicked: root.submitWifiSecrets(modelData, rowPassword.currentText(), identityField.currentText())
            }
          }
        }
      }
    }

    PrefsRow {
      available: root.wifiError.length > 0
      sectionHelp: false
      label: "Could not join"
      description: root.wifiError
      query: root.query
      keywords: ["failed"]

      PrefsButton {
        text: "Dismiss"
        enabled: root.wifiError.length > 0
        onClicked: root.wifiError = ""
      }
    }
  }

  PrefsGroup {
    title: "Connection"
    query: root.query
    detail: "Band pins the active network to 2.4, 5, or 6 GHz. The QR code is a scannable copy of the network you are on."

    PrefsRow {
      available: Omarchy.wifiConnected
      label: "Band"
      description: Omarchy.wifiBand.length
        ? "Stay on " + Omarchy.wifiBand + " GHz for this network, or let it pick."
        : "Stay on one band for this network, or let it pick."
      hint: "omarchy network band"
      query: root.query
      keywords: ["wifi", "wlan", "5ghz", "2.4", "6ghz"]

      PrefsSelect {
        value: Omarchy.wifiBandSelected
        options: root.bandOptions
        enabled: Omarchy.wifiConnected && root.bandOptions.length > 0
        onChanged: function(value) {
          if (value !== Omarchy.wifiBandSelected) Omarchy.setWifiBand(value)
        }
      }
    }

    PrefsRow {
      available: Omarchy.wifiConnected
      stretchControl: true
      label: "QR code"
      description: root.qrSsid.length ? ("A scannable code for " + root.qrSsid + ".") : "A scannable code for the network you are on, so someone nearby can join."
      hint: "omarchy network qr --meta"
      query: root.query
      keywords: ["share", "ssid", "password", "qrcode", "wifi-qr"]

      Column {
        width: parent.width
        spacing: 8

        Row {
          spacing: 8
          PrefsButton {
            text: root.qrLoading ? "Building…" : "Show"
            enabled: !root.qrLoading && Omarchy.wifiConnected
            onClicked: root.startQr()
          }
          PrefsButton {
            text: "Copy password"
            enabled: Omarchy.wifiIface.length > 0
            onClicked: Omarchy.copyWifiPassword()
          }
        }

        Text {
          visible: root.qrError.length > 0
          text: root.qrError
          color: Theme.urgent
          font.family: Theme.fontFamily
          font.pixelSize: Theme.captionSize
        }

        Column {
          visible: root.qrSize > 0
          spacing: 0
          Repeater {
            model: root.qrRows
            Row {
              required property var modelData
              spacing: 0
              Repeater {
                model: modelData
                Rectangle {
                  required property var modelData
                  width: 5
                  height: 5
                  color: modelData === 1 ? "#111111" : "#f7f7f7"
                }
              }
            }
          }
        }
      }
    }

    PrefsRow {
      label: "Restart Wi-Fi"
      description: "Unblock rfkill and restart NetworkManager's Wi-Fi. Try this if the radio looks stuck."
      hint: "omarchy restart wifi"
      query: root.query
      keywords: ["rfkill", "reload", "wlan"]

      PrefsButton {
        text: "Restart"
        enabled: Omarchy.wifiHw
        onClicked: Omarchy.restartWifi()
      }
    }
  }
}
