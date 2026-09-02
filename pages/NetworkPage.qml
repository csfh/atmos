import QtQuick
import QtQuick.Dialogs
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi
import "network" as Net

PrefsPage {
  id: root
  title: "Network"
  description: "What you are connected to, plus DNS. Wi-Fi and Bluetooth open their own pages. There is a speed test as well."

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

  function connectionSummary() {
    if (Omarchy.netKind === "ethernet") {
      var wired = "Ethernet"
      if (Omarchy.netIface) wired += " on " + Omarchy.netIface
      if (Omarchy.netIp) wired += ". " + Omarchy.netIp
      if (Omarchy.netSpeed) wired += ". " + Omarchy.netSpeed + " Mb/s"
      return wired + "."
    }
    if (Omarchy.netKind === "wifi") {
      var wifi = Omarchy.netSsid.length ? Omarchy.netSsid : "Wi-Fi"
      if (Omarchy.netSignal) wifi += ". Signal " + Omarchy.netSignal + "%"
      if (Omarchy.netIp) wifi += ". " + Omarchy.netIp
      return wifi + "."
    }
    return "Nothing is online on the default route."
  }

  function wifiSummary() {
    if (!Omarchy.wifiRadio) return "Radio is off."
    if (Omarchy.netKind === "wifi" && Omarchy.netSsid.length)
      return "Connected to " + Omarchy.netSsid + "."
    return "On. Open Wi-Fi to pick a network."
  }

  function bluetoothSummary() {
    if (!Omarchy.bluetooth) return "Radio is off."
    var n = Omarchy.bluetoothDevices.length
    if (n === 1) return "On. One paired device."
    if (n > 1) return "On. " + n + " paired devices."
    return "On. Open Bluetooth to pair a device."
  }

  PrefsGroup {
    title: "Connection"
    query: root.query
    detail: "What the machine is using to reach the internet right now. Ethernet shows the interface and link speed. Wi-Fi shows the network name and signal."
    hint: "omarchy network status"

    PrefsRow {
      label: "Status"
      description: root.connectionSummary()
      hint: "omarchy network status"
      query: root.query
      keywords: ["ethernet", "wifi", "online", "offline", "ip", "iface"]
    }

  }

  PrefsGroup {
    title: "Connectivity"
    query: root.query
    detail: "These open their own pages. Adapter power and pairing live there too."
    hint: "omarchy network"

    PrefsLink {
      available: Omarchy.wifiHw
      label: "Wi-Fi"
      description: "Nearby and saved networks, band, and a QR code you can share."
      hint: "nmcli radio wifi"
      query: root.query
      keywords: ["wlan", "ssid", "scan", "join", "psk", "qr", "rfkill"]
      valueText: root.wifiSummary()
      onClicked: root.openSubpage("wifi")
    }

    PrefsLink {
      label: "Bluetooth"
      description: "Paired devices, a scan for new ones, and the adapter power switch."
      hint: "omarchy bluetooth power"
      query: root.query
      keywords: ["bt", "pair", "headset", "scan", "forget", "radio"]
      valueText: root.bluetoothSummary()
      onClicked: root.openSubpage("bluetooth")
    }

    PrefsLink {
      label: "Speed test"
      description: "A short download, then an upload, on whatever you are connected to now."
      hint: "omarchy network speedtest"
      query: root.query
      keywords: ["bandwidth", "ping", "speedtest"]
      onClicked: root.openSubpage("speedtest")
    }
  }

  PrefsGroup {
    title: "DNS"
    query: root.query
    detail: "This writes NetworkManager and systemd-resolved so lookups go through the same resolvers. Cloudflare is 1.1.1.1 and 1.0.0.1. Google is 8.8.8.8 and 8.8.4.4. DHCP keeps whatever the link already handed out. Custom uses the servers field below. A VPN or a per-connection DNS setting in NetworkManager can still win for that connection."
    hint: "omarchy dns"

    PrefsRow {
      label: "DNS provider"
      description: "Who answers name lookups for this machine."
      hint: "omarchy dns"
      detail: "This writes NetworkManager and systemd-resolved so lookups go through the same resolvers. Cloudflare is 1.1.1.1 and 1.0.0.1. Google is 8.8.8.8 and 8.8.4.4. DHCP keeps whatever the link already handed out. Custom uses the servers field below. Picking a named provider replaces that list. A VPN or a per-connection DNS setting in NetworkManager can still win for that connection."
      query: root.query
      keywords: ["cloudflare", "google", "dhcp", "resolver"]

      PrefsSelect {
        value: Omarchy.dns
        options: root.dnsOptions
        enabled: !Omarchy.busy && !Omarchy.jobBusy && Omarchy.dns.length > 0
        onChanged: function(value) {
          if (value === "Custom" || value === Omarchy.dns) return
          Omarchy.setDns(value)
        }
      }
    }

    PrefsRow {
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
          enabled: !Omarchy.busy && !Omarchy.jobBusy
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
          enabled: !Omarchy.busy && !Omarchy.jobBusy && root.dnsReady
          onClicked: root.applyCustomDns(dnsField.currentText())
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Tailscale is a mesh VPN Omarchy can install. LocalSend and Taildrop are actions, not extra config."

    PrefsRow {
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
        spacing: 8
        PrefsButton {
          visible: !Omarchy.tailscaleInstalled
          text: "Install…"
          primary: true
          enabled: !Omarchy.busy && !Omarchy.jobBusy && !Omarchy.tailscaleInstalled
          onClicked: tailscaleInstallConfirm.ask()
        }
        PrefsButton {
          visible: Omarchy.tailscaleInstalled
          text: "Remove…"
          danger: true
          enabled: !Omarchy.busy && !Omarchy.jobBusy && Omarchy.tailscaleInstalled
          onClicked: tailscaleRemoveConfirm.ask()
        }
      }
    }

    PrefsRow {
      available: Omarchy.extras && Omarchy.extras.localsend === true
      label: "LocalSend clipboard"
      description: "Send whatever is on the clipboard to a nearby machine."
      hint: "omarchy share clipboard"
      query: root.query
      keywords: ["localsend", "share", "clipboard"]

      PrefsButton {
        text: "Share clipboard"
        enabled: !Omarchy.busy && Omarchy.extras && Omarchy.extras.localsend === true
        onClicked: Omarchy.shareClipboard()
      }
    }

    PrefsRow {
      available: Omarchy.extras && Omarchy.extras.localsend === true
      label: "LocalSend file"
      description: "Pick a file and send it with LocalSend."
      hint: "omarchy share file"
      query: root.query
      keywords: ["localsend", "share", "file"]

      PrefsButton {
        text: "Share file…"
        enabled: !Omarchy.busy && Omarchy.extras && Omarchy.extras.localsend === true
        onClicked: {
          root.shareFileMode = true
          shareFileDialog.open()
        }
      }
    }

    PrefsRow {
      available: Omarchy.extras && Omarchy.extras.localsend === true
      label: "LocalSend folder"
      description: "Pick a folder and send it with LocalSend."
      hint: "omarchy share folder"
      query: root.query
      keywords: ["localsend", "share", "folder"]

      PrefsButton {
        text: "Share folder…"
        enabled: !Omarchy.busy && Omarchy.extras && Omarchy.extras.localsend === true
        onClicked: shareFolderDialog.open()
      }
    }

    PrefsRow {
      available: Omarchy.tailscaleInstalled
      label: "Taildrop send"
      description: "Send a file to a machine on your tailnet. Type the short name, then pick a file."
      hint: "omarchy tailscale send"
      query: root.query
      keywords: ["taildrop", "tailscale", "send", "share"]

      Row {
        spacing: 8
        PrefsField {
          width: 140
          placeholder: "machine"
          enabled: !Omarchy.busy && Omarchy.tailscaleInstalled
          onEdited: function(value) { root.tailscaleMachine = value }
          onSubmitted: function(value) { root.tailscaleMachine = value }
        }
        PrefsButton {
          text: "Send file…"
          enabled: !Omarchy.busy && Omarchy.tailscaleInstalled && root.tailscaleMachine.length > 0
          onClicked: {
            root.shareFileMode = false
            shareFileDialog.open()
          }
        }
      }
    }

    PrefsRow {
      available: Omarchy.tailscaleInstalled
      label: "Taildrop receive"
      description: "Wait for one incoming file and drop it in Downloads."
      hint: "omarchy tailscale receive --once"
      query: root.query
      keywords: ["taildrop", "tailscale", "receive"]

      PrefsButton {
        text: "Receive once"
        enabled: !Omarchy.busy && Omarchy.tailscaleInstalled
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

