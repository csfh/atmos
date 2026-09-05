import QtQuick
import QtQuick.Dialogs
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi
import "network" as Net

PrefsPage {
  id: root
  title: "Network"
  description: "What you are connected to, plus DNS. Radios sit on this page. Networks, pairing, and a speed test open their own pages."

  property var stack: null
  property var navigator: null

  function openSubpage(id) {
    if (root.navigator && root.navigator.go) {
      root.navigator.go("network/" + id)
      return
    }
    if (!stack) return
    if (id === "speedtest") stack.push(speedtestPage)
    else if (id === "wifi") stack.push(wifiPage)
    else if (id === "bluetooth") stack.push(bluetoothPage)
  }

  Component { id: speedtestPage; Net.SpeedtestPage {} }
  Component { id: wifiPage; Net.WifiPage {} }
  Component { id: bluetoothPage; Net.BluetoothPage {} }

  readonly property var dnsOptions: [
    { value: "Cloudflare", label: "Cloudflare" },
    { value: "Google", label: "Google" },
    { value: "DHCP", label: "DHCP" },
    { value: "Custom", label: "Custom" }
  ]
  property string dnsError: ""
  property bool dnsReady: false
  property string tailscaleMachine: ""
  property bool shareFileMode: true

  function applyCustomDns(value) {
    var parsed = RichUi.parseDnsServers(value)
    dnsError = parsed.ok ? "" : parsed.error
    dnsReady = parsed.ok
    if (!parsed.ok) return
    Omarchy.setCustomDns(parsed.servers.join(" "))
  }

  function linkLabel() {
    if (Omarchy.netKind === "ethernet") return "Ethernet"
    if (Omarchy.netKind === "wifi") return Omarchy.netSsid.length ? Omarchy.netSsid : "Wi-Fi"
    return "Offline"
  }

  function linkDescription() {
    if (Omarchy.netKind === "ethernet") {
      var wired = ["Wired default route"]
      if (Omarchy.netIface) wired.push(Omarchy.netIface)
      if (Omarchy.netSpeed) wired.push(Omarchy.netSpeed + " Mb/s")
      return wired.join(" · ") + "."
    }
    if (Omarchy.netKind === "wifi") {
      var wifi = ["Wi-Fi default route"]
      if (Omarchy.netIface) wifi.push(Omarchy.netIface)
      if (Omarchy.netSignal) wifi.push(Omarchy.netSignal + "% signal")
      return wifi.join(" · ") + "."
    }
    return "No default route. Refresh after you plug in a cable or join Wi-Fi."
  }

  function wifiHubDescription() {
    if (!Omarchy.wifiHw)
      return "No Wi-Fi adapter."
    if (!Omarchy.wifiRadio)
      return "Scan and join nearby networks."
    if (Omarchy.netKind === "wifi" && Omarchy.netSsid.length)
      return "Connected to " + Omarchy.netSsid + "."
    return "Scanning for access points."
  }

  function bluetoothHubDescription() {
    if (!Omarchy.bluetooth)
      return "Pair a headset or mouse."
    var n = Omarchy.bluetoothDevices.length
    if (n === 1)
      return "One paired device."
    if (n > 1)
      return n + " paired devices."
    return "No paired devices."
  }

  PrefsGroup {
    title: "Connection"
    query: root.query
    detail: "What the machine is using to reach the internet right now. Ethernet shows the interface and link speed. Wi-Fi shows the network name and signal."
    hint: "omarchy network status"

    SettingRow {
      label: root.linkLabel()
      description: root.linkDescription()
      hint: "omarchy network status"
      query: root.query
      keywords: ["ethernet", "wifi", "online", "offline", "ip", "iface", "status"]

      PrefsButton {
        text: "Refresh"
        onClicked: Omarchy.refresh()
      }
    }

    SettingRow {
      available: Omarchy.netIp.length > 0
      label: "Address"
      description: Omarchy.netIp
      hint: "omarchy network status"
      query: root.query
      keywords: ["ip", "ipv4", "address", "copy"]

      PrefsButton {
        text: "Copy"
        enabled: Omarchy.netIp.length > 0
        onClicked: Omarchy.copyText(Omarchy.netIp)
      }
    }
  }

  PrefsGroup {
    title: "Connectivity"
    query: root.query
    detail: "Radios sit here. Nearby networks, pairing, and the speed test open their own pages."
    hint: "omarchy network"

    SettingRow {
      label: "Wi-Fi"
      description: root.wifiHubDescription()
      hint: "nmcli radio wifi"
      query: root.query
      keywords: ["wlan", "ssid", "scan", "join", "psk", "qr", "rfkill"]

      Row {
        spacing: Theme.space
        PrefsToggle {
          checked: Omarchy.wifiHw && Omarchy.wifiRadio
          enabled: Omarchy.wifiHw
          onToggled: Omarchy.setWifiRadio(!Omarchy.wifiRadio)
        }
        PrefsButton {
          text: "Manage…"
          enabled: Omarchy.wifiHw
          onClicked: root.openSubpage("wifi")
        }
      }
    }

    SettingRow {
      label: "Bluetooth"
      description: root.bluetoothHubDescription()
      hint: "omarchy bluetooth power"
      query: root.query
      keywords: ["bt", "pair", "headset", "scan", "forget", "radio"]

      Row {
        spacing: Theme.space
        PrefsToggle {
          checked: Omarchy.bluetooth
          onToggled: Omarchy.setBluetooth(!Omarchy.bluetooth)
        }
        PrefsButton {
          text: "Manage…"
          onClicked: root.openSubpage("bluetooth")
        }
      }
    }

    SettingRow {
      label: "Speed test"
      description: "A short download, then an upload, on whatever you are connected to now."
      hint: "omarchy network speedtest"
      query: root.query
      keywords: ["bandwidth", "ping", "speedtest"]

      PrefsButton {
        text: "Test…"
        onClicked: root.openSubpage("speedtest")
      }
    }
  }

  PrefsGroup {
    title: "DNS"
    query: root.query
    detail: "This writes NetworkManager and systemd-resolved so lookups go through the same resolvers. Cloudflare is 1.1.1.1 and 1.0.0.1. Google is 8.8.8.8 and 8.8.4.4. DHCP keeps whatever the link already handed out. Custom uses the servers field below. A VPN or a per-connection DNS setting in NetworkManager can still win for that connection."
    hint: "omarchy dns"

    SettingRow {
      label: "DNS provider"
      description: "Who answers name lookups for this machine."
      hint: "omarchy dns"
      detail: "This writes NetworkManager and systemd-resolved so lookups go through the same resolvers. Cloudflare is 1.1.1.1 and 1.0.0.1. Google is 8.8.8.8 and 8.8.4.4. DHCP keeps whatever the link already handed out. Custom uses the servers field below. Picking a named provider replaces that list. A VPN or a per-connection DNS setting in NetworkManager can still win for that connection."
      query: root.query
      keywords: ["cloudflare", "google", "dhcp", "resolver"]

      PrefsSelect {
        value: Omarchy.dns
        options: root.dnsOptions
        enabled: !Omarchy.jobBusy && Omarchy.dns.length > 0
        onChanged: function(value) {
          if (value === "Custom" || value === Omarchy.dns) return
          Omarchy.setDns(value)
        }
      }
    }

    SettingRow {
      stretchControl: true
      label: "Custom servers"
      description: root.dnsError.length > 0
        ? root.dnsError
        : "Type one or more IPv4 or IPv6 resolvers, separated by spaces. That switches the provider to Custom."
      hint: "omarchy dns Custom"
      query: root.query
      keywords: ["custom", "nameserver", "resolver"]

      Row {
        width: parent.width
        spacing: Theme.space
        PrefsField {
          id: dnsField
          width: parent.width - dnsSetBtn.width - parent.spacing
          placeholder: "1.1.1.1 8.8.8.8"
          horizontalAlignment: TextInput.AlignHCenter
          enabled: !Omarchy.jobBusy
          invalid: root.dnsError.length > 0
          onEdited: function(value) {
            var status = RichUi.dnsInputStatus(value)
            if (status.formatted !== value) dnsField.replaceText(status.formatted)
            root.dnsError = status.error
            root.dnsReady = status.ok
          }
          onSubmitted: function(value) { root.applyCustomDns(value) }
        }
        PrefsButton {
          id: dnsSetBtn
          text: "Set"
          primary: true
          enabled: !Omarchy.jobBusy && root.dnsReady
          onClicked: root.applyCustomDns(dnsField.currentText())
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Tailscale is a mesh VPN Omarchy can install. LocalSend and Taildrop are actions, not extra config."

    SettingRow {
      label: "Tailscale"
      description: Omarchy.tailscaleRunning
        ? "tailscaled is running."
        : (Omarchy.tailscaleInstalled
          ? "Tailscale is installed but the daemon is not running."
          : "Install Tailscale if you want this machine on a tailnet.")
      hint: "omarchy install service tailscale"
      query: root.query
      keywords: ["tailscale", "vpn", "tailnet", "mesh"]

      Row {
        spacing: Theme.space
        PrefsButton {
          visible: !Omarchy.tailscaleInstalled
          text: "Install…"
          primary: true
          enabled: !Omarchy.jobBusy && !Omarchy.tailscaleInstalled
          onClicked: tailscaleInstallConfirm.ask()
        }
        PrefsButton {
          visible: Omarchy.tailscaleInstalled
          text: "Remove…"
          danger: true
          enabled: !Omarchy.jobBusy && Omarchy.tailscaleInstalled
          onClicked: tailscaleRemoveConfirm.ask()
        }
      }
    }

    SettingRow {
      available: Omarchy.extras && Omarchy.extras.localsend === true
      label: "LocalSend clipboard"
      description: "Send whatever is on the clipboard to a nearby machine."
      hint: "omarchy share clipboard"
      query: root.query
      keywords: ["localsend", "share", "clipboard"]

      PrefsButton {
        text: "Share clipboard"
        enabled: Omarchy.extras && Omarchy.extras.localsend === true
        onClicked: Omarchy.shareClipboard()
      }
    }

    SettingRow {
      available: Omarchy.extras && Omarchy.extras.localsend === true
      label: "LocalSend file"
      description: "Pick a file and send it with LocalSend."
      hint: "omarchy share file"
      query: root.query
      keywords: ["localsend", "share", "file"]

      PrefsButton {
        text: "Share file…"
        enabled: Omarchy.extras && Omarchy.extras.localsend === true
        onClicked: {
          root.shareFileMode = true
          shareFileDialog.open()
        }
      }
    }

    SettingRow {
      available: Omarchy.extras && Omarchy.extras.localsend === true
      label: "LocalSend folder"
      description: "Pick a folder and send it with LocalSend."
      hint: "omarchy share folder"
      query: root.query
      keywords: ["localsend", "share", "folder"]

      PrefsButton {
        text: "Share folder…"
        enabled: Omarchy.extras && Omarchy.extras.localsend === true
        onClicked: shareFolderDialog.open()
      }
    }

    SettingRow {
      available: Omarchy.tailscaleInstalled
      label: "Taildrop send"
      description: "Send a file to a machine on your tailnet. Type the short name, then pick a file."
      hint: "omarchy tailscale send"
      query: root.query
      keywords: ["taildrop", "tailscale", "send", "share"]

      Row {
        spacing: Theme.space
        PrefsField {
          width: 140
          placeholder: "machine"
          enabled: Omarchy.tailscaleInstalled
          onEdited: function(value) { root.tailscaleMachine = value }
          onSubmitted: function(value) { root.tailscaleMachine = value }
        }
        PrefsButton {
          text: "Send file…"
          enabled: Omarchy.tailscaleInstalled && root.tailscaleMachine.length > 0
          onClicked: {
            root.shareFileMode = false
            shareFileDialog.open()
          }
        }
      }
    }

    SettingRow {
      available: Omarchy.tailscaleInstalled
      label: "Taildrop receive"
      description: "Wait for one incoming file and drop it in Downloads."
      hint: "omarchy tailscale receive --once"
      query: root.query
      keywords: ["taildrop", "tailscale", "receive"]

      PrefsButton {
        text: "Receive once"
        enabled: Omarchy.tailscaleInstalled
        onClicked: Omarchy.tailscaleReceive()
      }
    }
  }

  FileDialog {
    id: shareFileDialog
    title: root.shareFileMode ? "Share a file" : "Send a file"
    onAccepted: {
      var path = RichUi.pathFromUrl(selectedFile)
      if (root.shareFileMode) Omarchy.shareFile(path)
      else Omarchy.tailscaleSend(root.tailscaleMachine, path)
    }
  }

  FolderDialog {
    id: shareFolderDialog
    title: "Share a folder"
    onAccepted: Omarchy.shareFolder(RichUi.pathFromUrl(selectedFolder))
  }

  PrefsConfirm {
    id: tailscaleInstallConfirm
    title: "Install Tailscale"
    message: "Install Tailscale and start the daemon. You will still need to log in with tailscale up."
    confirmText: "Install"
    onConfirmed: Omarchy.installTailscale()
  }

  PrefsConfirm {
    id: tailscaleRemoveConfirm
    title: "Remove Tailscale"
    message: "Remove Tailscale and its bar plugin."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeTailscale()
  }

  Component.onCompleted: {
    tailscaleInstallConfirm.parent = root.prefsOverlay
    tailscaleRemoveConfirm.parent = root.prefsOverlay
  }
}

