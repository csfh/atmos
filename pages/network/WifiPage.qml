import QtQuick
import Quickshell.Networking
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi
import "../../services/NetworkPrefs.js" as NetPrefs

PrefsPage {
  id: root
  hubId: "network/wifi"
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
  readonly property var qrRows: Omarchy.wifiQrRows
  readonly property int qrSize: Omarchy.wifiQrSize
  readonly property string qrSsid: Omarchy.wifiQrSsid
  readonly property string qrError: Omarchy.wifiQrError
  readonly property bool qrLoading: Omarchy.jobKind === "wifi-qr" && Omarchy.jobBusy
  property bool enterpriseBusy: Omarchy.jobKind === "wifi-enterprise"
  property bool wifiJoinBusy: Omarchy.jobKind === "wifi-join" || Omarchy.jobKind === "wifi-enterprise"
  property string staticUuid: ""
  property string staticError: ""
  readonly property var networkDevices: Networking.devices ? Networking.devices.values : []
  readonly property var wifiDevice: findWifiDevice()
  readonly property var wifiNetworkObjects: wifiDevice && wifiDevice.networks ? wifiDevice.networks.values : []
  readonly property bool wantScan: root.visible && Omarchy.wifiHw && Omarchy.wifiRadio

  function findWifiDevice() {
    var devices = RichUi.objectList(root.networkDevices)
    var fallback = null
    for (var i = 0; i < devices.length; i++) {
      var device = devices[i]
      if (!device || device.type !== DeviceType.Wifi) continue
      if (device.connected) return device
      if (!fallback) fallback = device
    }
    return fallback
  }

  function securityKind(sec) {
    if (sec === WifiSecurityType.Open) return "open"
    if (sec === WifiSecurityType.Owe) return "owe"
    if (sec === WifiSecurityType.Wpa2Eap || sec === WifiSecurityType.WpaEap) return "enterprise"
    return "psk"
  }

  function rebuildWifi() {
    var objects = RichUi.objectList(root.wifiNetworkObjects)
    var rows = []
    for (var i = 0; i < objects.length; i++) {
      var net = objects[i]
      if (!net || !net.name) continue
      rows.push(RichUi.wifiRow(net.name, (net.signalStrength || 0) * 100, securityKind(net.security), net.connected, net.known))
    }
    wifiRows = RichUi.sortWifiRows(rows)
  }

  function setScanner(on) {
    var next = on ? root.wifiDevice : null
    if (scannerDevice && scannerDevice !== next)
      scannerDevice.scannerEnabled = false
    scannerDevice = next
    if (scannerDevice)
      scannerDevice.scannerEnabled = true
  }

  function bounceScanner() {
    if (!scannerDevice) {
      setScanner(root.wantScan)
      rebuildWifi()
      return
    }
    scannerDevice.scannerEnabled = false
    scanRestart.restart()
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
    actionSsid = row.ssid
    actionKind = "connect"
    wifiError = ""
    Omarchy.joinWifi(row.ssid, "")
  }

  function submitWifiSecrets(row, secret, identity) {
    if (!row) return
    secret = String(secret || "")
    if (!secret) return
    wifiError = ""
    if (RichUi.isEnterprise(row.securityKind)) {
      identity = String(identity || "")
      if (!identity) return
      actionSsid = row.ssid
      actionKind = "enterprise"
      Omarchy.connectEnterpriseWifi(row.ssid, identity, secret)
      passwordSsid = ""
      return
    }
    actionSsid = row.ssid
    actionKind = "connect"
    Omarchy.joinWifi(row.ssid, secret)
    passwordSsid = ""
  }

  function startQr() {
    Omarchy.fetchWifiQr()
  }

  Component.onCompleted: {
    setScanner(root.wantScan)
    rebuildWifi()
  }
  Component.onDestruction: setScanner(false)
  onWantScanChanged: setScanner(root.wantScan)
  onWifiDeviceChanged: setScanner(root.wantScan)
  onWifiNetworkObjectsChanged: rebuildWifi()

  Timer {
    id: scanRestart
    interval: 80
    repeat: false
    onTriggered: {
      if (root.scannerDevice)
        root.scannerDevice.scannerEnabled = true
      root.rebuildWifi()
    }
  }

  Timer {
    interval: 900
    running: root.wantScan
    repeat: true
    onTriggered: root.rebuildWifi()
  }

  Connections {
    target: Networking.devices
    function onValuesChanged() {
      root.rebuildWifi()
    }
  }

  Connections {
    target: root.wifiDevice && root.wifiDevice.networks
    function onValuesChanged() {
      root.rebuildWifi()
    }
  }

  onEnterpriseBusyChanged: {
    if (enterpriseBusy) return
    if (root.actionKind !== "enterprise") return
    if (Omarchy.lastError) root.wifiError = "Enterprise join failed"
    root.actionKind = ""
    root.actionSsid = ""
    root.passwordSsid = ""
  }

  onWifiJoinBusyChanged: {
    if (wifiJoinBusy) return
    if (root.actionKind !== "connect" && root.actionKind !== "enterprise") return
    if (Omarchy.lastError) root.wifiError = root.actionKind === "enterprise" ? "Enterprise join failed" : "Could not join"
    root.actionKind = ""
    root.actionSsid = ""
    root.passwordSsid = ""
  }

  Connections {
    target: Omarchy
    function onNetSsidChanged() {
      if (root.actionKind === "connect" && Omarchy.netSsid === root.actionSsid) {
        root.actionKind = ""
        root.actionSsid = ""
      }
    }
    function onLastErrorChanged() {
      if (!root.actionKind || !Omarchy.lastError) return
      if (root.actionKind !== "connect" && root.actionKind !== "enterprise") return
      root.wifiError = root.actionKind === "enterprise" ? "Enterprise join failed" : "Could not join"
      root.actionKind = ""
      root.actionSsid = ""
      root.passwordSsid = ""
    }
  }

  PrefsConfirm {
    id: forgetWifiConfirm
    title: "Forget network"
    message: "Forget this saved Wi-Fi network? You will need the password again the next time you join."
    confirmText: "Forget"
    onConfirmed: Omarchy.forgetWifiSsid(forgetWifiConfirm.payload)
    property string payload: ""
  }

  PrefsGroup {
    title: "Adapter"
    query: root.query
    detail: "The radio NetworkManager uses to scan and join. Off is like airplane mode for Wi-Fi only."
    hint: "nmcli radio wifi"

    SettingRow {
      label: "Wi-Fi radio"
      description: !Omarchy.wifiHw
        ? "No Wi-Fi adapter."
        : (Omarchy.wifiRadio
          ? (Omarchy.netKind === "wifi" && Omarchy.netSsid.length
            ? "Connected to " + Omarchy.netSsid + ". Nearby networks stay below."
            : "Scanning for access points below.")
          : "Scan and join nearby networks.")
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
    framed: true
    title: "Networks"
    query: root.query
    detail: "Nearby and saved access points from a NetworkManager scan. The list refreshes while this page is open. Join a known network, or type a password. Enterprise networks also ask for an identity. Forget drops a saved connection."
    hint: "nmcli"

    SettingRow {
      available: root.wifiRows.length === 0
      sectionHelp: false
      label: "Nearby networks"
      description: !Omarchy.wifiHw
        ? "No Wi-Fi adapter."
        : (Omarchy.wifiRadio
          ? "No networks nearby."
          : "No networks. Turn the radio on above.")
      query: root.query
      keywords: ["scan", "ssid", "empty"]

      PrefsButton {
        text: "Refresh"
        enabled: Omarchy.wifiHw && Omarchy.wifiRadio
        onClicked: {
          Omarchy.refresh()
          root.bounceScanner()
        }
      }
    }

    Repeater {
      model: root.wifiRows

      SettingRow {
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
          spacing: Theme.space

          Row {
            spacing: Theme.space
            PrefsButton {
              text: modelData && modelData.connected ? "Disconnect" : (root.actionSsid === modelData.ssid ? "Joining…" : "Join")
              primary: !(modelData && modelData.connected)
              enabled: root.actionKind === "" && !root.wifiJoinBusy && modelData
              onClicked: {
                if (modelData.connected) Omarchy.deactivateWifiSsid(modelData.ssid)
                else root.connectWifi(modelData)
              }
            }
            PrefsButton {
              visible: !!(modelData && modelData.known && !modelData.connected)
              text: "Forget…"
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
            spacing: Theme.space

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

    SettingRow {
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

    SettingRow {
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

    SettingRow {
      available: Omarchy.wifiConnected
      stretchControl: true
      label: "QR code"
      description: root.qrSsid.length ? ("A scannable code for " + root.qrSsid + ".") : "A scannable code for the network you are on, so someone nearby can join."
      hint: "omarchy network qr --meta"
      query: root.query
      keywords: ["share", "ssid", "password", "qrcode", "wifi-qr"]

      Column {
        width: parent.width
        spacing: Theme.space

        Row {
          spacing: Theme.space
          PrefsButton {
            text: root.qrLoading ? "Building…" : "Show QR"
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

    SettingRow {
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

  PrefsDialog {
    id: staticDialog
    title: "Static IPv4"

    PrefsText {
      width: parent.width
      text: root.staticError.length ? root.staticError : "Address and prefix are required. Leave gateway or DNS blank to skip them."
      color: root.staticError.length ? Theme.urgent : Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField { id: staticAddr; width: parent.width; placeholder: "10.0.0.8" }
    PrefsField { id: staticPrefix; width: parent.width; placeholder: "24" }
    PrefsField { id: staticGateway; width: parent.width; placeholder: "10.0.0.1" }
    PrefsField { id: staticDns; width: parent.width; placeholder: "1.1.1.1" }

    Row {
      anchors.right: parent.right
      spacing: Theme.space
      PrefsButton {
        text: "Cancel"
        onClicked: staticDialog.close()
      }
      PrefsButton {
        text: "Apply"
        primary: true
        onClicked: {
          var spec = {
            method: "manual",
            address: staticAddr.currentText(),
            prefix: staticPrefix.currentText(),
            gateway: staticGateway.currentText(),
            dns: staticDns.currentText()
          }
          if (!NetPrefs.argvFor("ipv4", { uuid: root.staticUuid, method: "manual", address: spec.address, prefix: spec.prefix, gateway: spec.gateway, dns: spec.dns })) {
            root.staticError = "Check the address and prefix."
            return
          }
          root.staticError = ""
          Omarchy.setConnectionIpv4(root.staticUuid, spec)
          staticDialog.close()
        }
      }
    }
  }

  PrefsGroup {
    framed: true
    title: "Saved connections"
    query: root.query
    detail: "Metered, priority, MAC randomization, and static IPv4 write through nmcli. Forget is still on the scan list."
    hint: "nmcli connection modify"

    SettingRow {
      available: Omarchy.wifiConnections.length === 0
      label: "Saved networks"
      description: "No saved connections."
      query: root.query
      keywords: ["saved", "empty"]
    }

    Repeater {
      model: Omarchy.wifiConnections

      SettingRow {
        required property var modelData
        label: modelData && modelData.name ? modelData.name : "Connection"
        description: modelData && modelData.active ? "Active now." : "Saved."
        hint: modelData && modelData.uuid ? modelData.uuid : "nmcli"
        query: root.query
        keywords: ["metered", "priority", "mac", "static", "ipv4"]

        Column {
          spacing: Theme.space
          PrefsSelect {
            value: modelData && modelData.metered ? String(modelData.metered) : "unknown"
            options: [
              { value: "unknown", label: "Metered: auto" },
              { value: "yes", label: "Metered" },
              { value: "no", label: "Not metered" }
            ]
            onChanged: function(value) {
              if (modelData && modelData.uuid) Omarchy.setConnectionMetered(modelData.uuid, value)
            }
          }
          PrefsField {
            width: 80
            placeholder: "Priority"
            value: modelData && modelData.priority != null ? String(modelData.priority) : "0"
            onSubmitted: function(value) {
              if (modelData && modelData.uuid) Omarchy.setConnectionPriority(modelData.uuid, value)
            }
          }
          PrefsSelect {
            value: modelData && modelData.mac ? String(modelData.mac) : "default"
            options: [
              { value: "default", label: "MAC default" },
              { value: "random", label: "Random MAC" },
              { value: "stable", label: "Stable random" },
              { value: "permanent", label: "Permanent" },
              { value: "preserve", label: "Preserve" }
            ]
            onChanged: function(value) {
              if (modelData && modelData.uuid) Omarchy.setConnectionMac(modelData.uuid, value)
            }
          }
          Row {
            spacing: Theme.space
            PrefsButton {
              text: "DHCP"
              onClicked: {
                if (modelData && modelData.uuid) Omarchy.setConnectionIpv4(modelData.uuid, { method: "auto" })
              }
            }
            PrefsButton {
              text: "Static…"
              onClicked: {
                root.staticUuid = modelData.uuid
                root.staticError = ""
                staticDialog.open()
              }
            }
          }
        }
      }
    }
  }
}
