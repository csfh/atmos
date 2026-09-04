import QtQuick
import Quickshell.Bluetooth
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Bluetooth"
  description: "Paired devices, a scan for new ones, and the adapter power switch."

  property bool scanningBt: false
  property var discoveredBt: []
  property var pairedBt: []
  readonly property var adapter: Bluetooth.defaultAdapter
  readonly property var bluetoothDeviceObjects: Bluetooth.devices ? Bluetooth.devices.values : []
  readonly property bool wantDiscover: root.visible && Omarchy.bluetooth && root.scanningBt

  function rebuildBt() {
    var lists = RichUi.bluetoothLists(root.bluetoothDeviceObjects)
    var livePaired = lists.paired || []
    if (livePaired.length > 0)
      pairedBt = livePaired
    else if (Omarchy.bluetoothDevices && Omarchy.bluetoothDevices.length)
      pairedBt = Omarchy.bluetoothDevices
    else
      pairedBt = []
    discoveredBt = lists.discovered || []
  }

  function syncDiscovering() {
    if (!root.adapter) return
    root.adapter.discovering = root.wantDiscover
  }

  Component.onCompleted: rebuildBt()
  Component.onDestruction: {
    if (root.adapter) root.adapter.discovering = false
  }
  onWantDiscoverChanged: syncDiscovering()
  onAdapterChanged: syncDiscovering()
  onBluetoothDeviceObjectsChanged: rebuildBt()

  Timer {
    interval: 900
    running: root.visible && (root.scanningBt || Omarchy.bluetooth)
    repeat: true
    onTriggered: root.rebuildBt()
  }

  Connections {
    target: Bluetooth.devices
    function onValuesChanged() {
      root.rebuildBt()
    }
  }

  Connections {
    target: Omarchy
    function onBluetoothDevicesChanged() {
      root.rebuildBt()
    }
    function onBluetoothChanged() {
      root.syncDiscovering()
      root.rebuildBt()
    }
  }

  PrefsConfirm {
    id: forgetBtConfirm
    title: "Forget device"
    message: "Forget this pairing? You will need to pair the device again the next time you want it."
    confirmText: "Forget"
    onConfirmed: {
      if (forgetBtConfirm.payload)
        Omarchy.forgetBluetoothDevice(forgetBtConfirm.payload)
    }
    property string payload: ""
  }

  PrefsGroup {
    title: "Adapter"
    query: root.query
    detail: "Powers the adapter and remembers the choice across reboots. Restart unblocks rfkill and brings BlueZ back up."
    hint: "omarchy bluetooth power"

    PrefsRow {
      label: "Bluetooth radio"
      description: Omarchy.bluetooth
        ? "Paired devices show up below. Scan further down for something new."
        : "The adapter is powered down. Turn it on to pair a headset or mouse."
      hint: "omarchy bluetooth power"
      query: root.query
      keywords: ["bt", "radio", "wireless"]

      PrefsToggle {
        checked: Omarchy.bluetooth
        onToggled: Omarchy.setBluetooth(!Omarchy.bluetooth)
      }
    }

    PrefsRow {
      label: "Restart Bluetooth"
      description: "Unblock rfkill and restart BlueZ. Try this if the adapter looks stuck."
      hint: "omarchy restart bluetooth"
      query: root.query
      keywords: ["rfkill", "bluez", "adapter"]

      PrefsButton {
        text: "Restart"
        onClicked: Omarchy.restartBluetooth()
      }
    }
  }

  PrefsGroup {
    title: "Paired"
    query: root.query
    detail: "Connect, disconnect, or forget a paired device. The adapter stays on until you turn it off above."
    hint: "omarchy bluetooth device"

    PrefsRow {
      available: root.pairedBt.length === 0
      sectionHelp: false
      label: "Paired devices"
      description: Omarchy.bluetooth
        ? "Nothing is paired yet. Scan nearby for a headset or mouse."
        : "Turn the radio on above, then scan for a nearby device."
      query: root.query
      keywords: ["empty"]

      PrefsButton {
        text: "Scan"
        enabled: Omarchy.bluetooth
        onClicked: root.scanningBt = true
      }
    }

    Repeater {
      model: root.pairedBt

      PrefsRow {
        required property var modelData
        available: true
        sectionHelp: false
        label: modelData && modelData.name ? modelData.name : "Bluetooth device"
        description: modelData && modelData.connected ? "Connected and ready." : "Paired. Connect when you want to use it."
        hint: "omarchy bluetooth device"
        query: root.query
        keywords: ["bt", "headset", "mouse", "keyboard", "forget"]

        Row {
          spacing: 8
          PrefsButton {
            text: modelData && modelData.connected ? "Disconnect" : "Connect"
            primary: !(modelData && modelData.connected)
            enabled: modelData && modelData.address
            onClicked: {
              if (modelData.connected) Omarchy.disconnectBluetoothDevice(modelData.address)
              else Omarchy.connectBluetoothDevice(modelData.address)
            }
          }
          PrefsButton {
            text: "Forget"
            danger: true
            enabled: modelData && modelData.address
            onClicked: {
              forgetBtConfirm.payload = modelData.address
              forgetBtConfirm.ask()
            }
          }
        }
      }
    }
  }

  PrefsGroup {
    title: "Nearby"
    query: root.query
    detail: "Scan looks for unpaired devices around you. Pair adds one to the list above."

    PrefsRow {
      label: "Scan"
      description: !Omarchy.bluetooth
        ? "Turn the radio on above. The switch stays off until the adapter is powered."
        : (root.scanningBt ? "Looking for unpaired devices nearby." : "Look for unpaired devices nearby.")
      hint: "bluetoothctl scan"
      query: root.query
      keywords: ["discover", "pair", "headset"]

      PrefsToggle {
        checked: Omarchy.bluetooth && root.scanningBt
        enabled: Omarchy.bluetooth
        onToggled: root.scanningBt = !root.scanningBt
      }
    }

    Repeater {
      model: root.discoveredBt

      PrefsRow {
        required property var modelData
        available: Omarchy.bluetooth && root.scanningBt
        sectionHelp: false
        label: modelData && modelData.name ? modelData.name : "Device"
        description: modelData && modelData.address ? modelData.address : ""
        hint: "omarchy bluetooth device pair"
        query: root.query
        keywords: ["pair", "discover"]

        PrefsButton {
          text: "Pair"
          primary: true
          enabled: modelData && modelData.address
          onClicked: Omarchy.pairBluetoothDevice(modelData.address)
        }
      }
    }
  }

}
