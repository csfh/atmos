import QtQuick
import Quickshell.Bluetooth
import "../../components"
import "../../services"
import "../../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  hubId: "bluetooth"
  title: I18n.tr("Bluetooth")
  description: I18n.tr("Paired devices, a scan for new ones, and the adapter power switch.")

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
    title: I18n.tr("Forget device")
    message: "Forget this pairing? You will need to pair the device again the next time you want it."
    confirmText: "Forget"
    onConfirmed: {
      if (forgetBtConfirm.payload)
        Omarchy.forgetBluetoothDevice(forgetBtConfirm.payload)
    }
    property string payload: ""
  }

  PrefsGroup {
    title: I18n.tr("Adapter")
    query: root.query
    detail: "Powers the adapter and remembers the choice across reboots. Restart unblocks rfkill and brings BlueZ back up."
    hint: "omarchy bluetooth power"

    SettingRow {
      label: "Bluetooth radio"
      description: Omarchy.bluetooth
        ? "Paired devices show up below. Scan further down for something new."
        : "Paired devices and scanning use this adapter."
      hint: "omarchy bluetooth power"
      query: root.query
      keywords: ["bt", "radio", "wireless"]

      PrefsToggle {
        checked: Omarchy.bluetooth
        onToggled: Omarchy.set("bluetooth", !Omarchy.bluetooth)
      }
    }

    SettingRow {
      available: !!root.adapter
      label: "Discoverable"
      description: I18n.tr("Other devices can see this adapter while this is on.")
      hint: "adapter.discoverable"
      query: root.query
      keywords: ["discoverable", "visible", "pair"]

      PrefsToggle {
        checked: !!(root.adapter && root.adapter.discoverable)
        enabled: Omarchy.bluetooth && !!root.adapter
        onToggled: {
          if (root.adapter) root.adapter.discoverable = !root.adapter.discoverable
        }
      }
    }

    SettingRow {
      label: "Restart Bluetooth"
      description: I18n.tr("Unblock rfkill and restart BlueZ. Try this if the adapter looks stuck.")
      hint: "omarchy restart bluetooth"
      query: root.query
      keywords: ["rfkill", "bluez", "adapter"]

      PrefsButton {
        text: I18n.tr("Restart")
        onClicked: Omarchy.restartBluetooth()
      }
    }
  }

  PrefsGroup {
    framed: true
    title: I18n.tr("Paired")
    query: root.query
    detail: "Connect, disconnect, or forget a paired device. The adapter stays on until you turn it off above."
    hint: "omarchy bluetooth device"

    SettingRow {
      available: root.pairedBt.length === 0
      sectionHelp: false
      label: "Paired devices"
      description: I18n.tr("No paired devices.")
      query: root.query
      keywords: ["empty"]
    }

    Repeater {
      model: root.pairedBt

      SettingRow {
        required property var modelData
        available: true
        sectionHelp: false
        label: modelData && modelData.name ? modelData.name : "Bluetooth device"
        description: (modelData && modelData.connected ? I18n.tr("Connected. ") : I18n.tr("Paired. "))
        + (modelData && modelData.battery != null && modelData.battery !== "" ? I18n.tr("Battery {battery}. ", { battery: modelData.battery }) : "")
        + I18n.tr("Trust keeps it auto-connecting.")
        hint: "omarchy bluetooth device"
        query: root.query
        keywords: ["bt", "headset", "mouse", "keyboard", "forget"]

        Row {
          spacing: Theme.space
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
            text: I18n.tr("Trust")
            enabled: modelData && modelData.address
            onClicked: Omarchy.trustBluetoothDevice(modelData.address)
          }
          PrefsButton {
            text: I18n.tr("Forget…")
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
    framed: true
    title: I18n.tr("Nearby")
    query: root.query
    detail: "Scan looks for unpaired devices around you. Pair adds one to the list above."

    SettingRow {
      label: "Scan"
      description: !Omarchy.bluetooth
        ? "The adapter is off."
        : (root.scanningBt
          ? "Looking for unpaired devices nearby."
          : "Unpaired devices nearby appear below while this is on.")
      hint: "bluetoothctl scan"
      query: root.query
      keywords: ["discover", "pair", "headset"]

      PrefsToggle {
        checked: Omarchy.bluetooth && root.scanningBt
        enabled: Omarchy.bluetooth
        onToggled: root.scanningBt = !root.scanningBt
      }
    }

    SettingRow {
      available: Omarchy.bluetooth && root.scanningBt && root.discoveredBt.length === 0
      sectionHelp: false
      label: "Nearby devices"
      description: I18n.tr("No unpaired devices nearby.")
      query: root.query
      keywords: ["empty", "discover"]
    }

    Repeater {
      model: root.discoveredBt

      SettingRow {
        required property var modelData
        available: Omarchy.bluetooth && root.scanningBt
        sectionHelp: false
        label: modelData && modelData.name ? modelData.name : "Device"
        description: modelData && modelData.address ? modelData.address : ""
        hint: "omarchy bluetooth device pair"
        query: root.query
        keywords: ["pair", "discover"]

        PrefsButton {
          text: I18n.tr("Pair")
          primary: true
          enabled: modelData && modelData.address
          onClicked: Omarchy.pairBluetoothDevice(modelData.address)
        }
      }
    }
  }

}
