import QtQuick
import "../components"
import "../services"
import "../services/RichUi.js" as RichUi

PrefsPage {
  id: root
  title: "Security"
  description: "Fingerprint, a security key, and whether this machine accepts SSH. Passwordless sudo is under Advanced."

  property string sshKeyDraft: ""
  readonly property string sshKeyParsed: RichUi.parseSshPublicKey(root.sshKeyDraft)
  readonly property bool sshKeyValid: root.sshKeyParsed.length > 0

  PrefsConfirm {
    id: fingerprintSetupConfirm
    title: "Set up fingerprint"
    message: "Enroll a print for sudo, polkit, and the lock screen. You will need to touch the reader during setup."
    confirmText: "Set up"
    onConfirmed: Omarchy.setupFingerprint()
  }

  PrefsConfirm {
    id: fingerprintRemoveConfirm
    title: "Remove fingerprint"
    message: "Stop using the fingerprint reader for sudo, polkit, and the lock screen."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeFingerprint()
  }

  PrefsConfirm {
    id: fido2SetupConfirm
    title: "Set up a security key"
    message: "Register a FIDO2 key for sudo and polkit. Have the key ready."
    confirmText: "Set up"
    onConfirmed: Omarchy.setupFido2()
  }

  PrefsConfirm {
    id: fido2RemoveConfirm
    title: "Remove security key"
    message: "Stop using FIDO2 keys for sudo and polkit."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeFido2()
  }

  PrefsConfirm {
    id: sshdDisableConfirm
    title: "Turn off SSH"
    message: "Stop the OpenSSH server and close the firewall port. Authorized keys on this account stay."
    confirmText: "Turn off"
    onConfirmed: Omarchy.disableSshd()
  }

  PrefsConfirm {
    id: passwordlessOffConfirm
    title: "Turn off passwordless sudo"
    message: "Sudo will ask for a password again."
    confirmText: "Turn off"
    onConfirmed: Omarchy.disablePasswordlessSudo()
  }

  PrefsConfirm {
    id: dockerOnConfirm
    title: "Sudoless Docker"
    message: "Add this account to the docker group so Docker runs without sudo. You may need to log in again."
    confirmText: "Set up"
    onConfirmed: Omarchy.setupSudolessDocker()
  }

  PrefsConfirm {
    id: dockerOffConfirm
    title: "Remove sudoless Docker"
    message: "Take this account out of the docker group. Docker will need sudo again."
    confirmText: "Remove"
    onConfirmed: Omarchy.removeSudolessDocker()
  }

  PrefsDialog {
    id: sshdDialog
    title: "Turn on SSH"

    PrefsText {
      width: parent.width
      text: Omarchy.jobKind === "security-sshd" && Omarchy.jobBusy
        ? "Installing and starting OpenSSH…"
        : (root.sshKeyDraft.length > 0 && !root.sshKeyValid
          ? "Paste one public key line (ssh-ed25519 or ssh-rsa). Turn on stays off until it looks valid."
          : "Paste a public key. Omarchy starts sshd, opens the firewall, and adds that key to authorized_keys.")
      color: Theme.muted
      font.family: Theme.fontFamily
      font.pixelSize: Theme.captionSize
    }

    PrefsField {
      id: sshKeyField
      width: parent.width
      value: root.sshKeyDraft
      placeholder: "ssh-ed25519 AAAA… comment"
      enabled: !Omarchy.jobBusy
      invalid: root.sshKeyDraft.length > 0 && !root.sshKeyValid
      onEdited: function(value) { root.sshKeyDraft = value }
      onSubmitted: function(value) {
        root.sshKeyDraft = value
        if (!root.sshKeyValid || Omarchy.jobBusy) return
        Omarchy.setupSshd(root.sshKeyParsed)
        sshdDialog.close()
      }
    }

    PrefsButton {
      text: "Turn on"
      primary: true
      enabled: !Omarchy.jobBusy && root.sshKeyValid
      onClicked: {
        Omarchy.setupSshd(root.sshKeyParsed)
        sshdDialog.close()
      }
    }
  }

  Component.onCompleted: {
    fingerprintSetupConfirm.parent = root.prefsOverlay
    fingerprintRemoveConfirm.parent = root.prefsOverlay
    fido2SetupConfirm.parent = root.prefsOverlay
    fido2RemoveConfirm.parent = root.prefsOverlay
    sshdDisableConfirm.parent = root.prefsOverlay
    passwordlessOffConfirm.parent = root.prefsOverlay
    dockerOnConfirm.parent = root.prefsOverlay
    dockerOffConfirm.parent = root.prefsOverlay
    sshdDialog.parent = root.prefsOverlay
  }

  PrefsGroup {
    title: "Login"
    query: root.query
    detail: "Fingerprint and FIDO2 change PAM for sudo, polkit, and the lock screen. Set up talks to the reader or key."

    SettingRow {
      available: Omarchy.fingerprintAvailable
      label: "Fingerprint"
      description: Omarchy.fingerprintConfigured
        ? "The reader can unlock sudo, polkit, and the lock screen."
        : "This machine has a reader. Set up enrolls a print and turns it on for sudo, polkit, and lock."
      hint: "omarchy setup security fingerprint"
      query: root.query
      keywords: ["fingerprint", "fprintd", "biometric", "lock"]

      Row {
        spacing: Theme.space
        PrefsButton {
          visible: !Omarchy.fingerprintConfigured
          text: "Set up…"
          primary: true
          enabled: !Omarchy.jobBusy && Omarchy.fingerprintAvailable && !Omarchy.fingerprintConfigured
          onClicked: fingerprintSetupConfirm.ask()
        }
        PrefsButton {
          visible: Omarchy.fingerprintConfigured
          text: "Remove…"
          danger: true
          enabled: !Omarchy.jobBusy && Omarchy.fingerprintConfigured
          onClicked: fingerprintRemoveConfirm.ask()
        }
      }
    }

    SettingRow {
      label: "Security key"
      description: Omarchy.fido2Configured
        ? "A FIDO2 key can stand in for a password on sudo and polkit."
        : "Register a FIDO2 key for sudo and polkit."
      hint: "omarchy setup security fido2"
      query: root.query
      keywords: ["fido2", "yubikey", "webauthn", "u2f"]

      Row {
        spacing: Theme.space
        PrefsButton {
          visible: !Omarchy.fido2Configured
          text: "Set up…"
          primary: true
          enabled: !Omarchy.jobBusy && !Omarchy.fido2Configured
          onClicked: fido2SetupConfirm.ask()
        }
        PrefsButton {
          visible: Omarchy.fido2Configured
          text: "Remove…"
          danger: true
          enabled: !Omarchy.jobBusy && Omarchy.fido2Configured
          onClicked: fido2RemoveConfirm.ask()
        }
      }
    }
  }

  PrefsGroup {
    title: "Remote"
    query: root.query
    detail: "OpenSSH on this machine. Turning it on needs a public key. Turning it off leaves authorized_keys alone."

    SettingRow {
      label: "SSH server"
      description: Omarchy.sshdActive
        ? "sshd is running. Other machines can log in with an authorized key."
        : (Omarchy.sshdEnabled
          ? "sshd is enabled but not running right now."
          : "The OpenSSH server is stopped. Turn it on with a public key if you want remote login.")
      hint: "omarchy setup security sshd"
      query: root.query
      keywords: ["ssh", "sshd", "openssh", "remote"]

      Row {
        spacing: Theme.space
        PrefsButton {
          visible: !Omarchy.sshdEnabled && !Omarchy.sshdActive
          text: "Turn on…"
          primary: true
          enabled: !Omarchy.jobBusy && !Omarchy.sshdEnabled && !Omarchy.sshdActive
          onClicked: {
            root.sshKeyDraft = ""
            sshKeyField.clear()
            sshdDialog.open()
          }
        }
        PrefsButton {
          visible: Omarchy.sshdEnabled || Omarchy.sshdActive
          text: "Turn off…"
          danger: true
          enabled: !Omarchy.jobBusy && (Omarchy.sshdEnabled || Omarchy.sshdActive)
          onClicked: sshdDisableConfirm.ask()
        }
      }
    }
  }

  PrefsGroup {
    title: "Advanced"
    query: root.query
    detail: "Passwordless sudo is timed. Sudoless Docker puts this account in the docker group."

    SettingRow {
      stretchControl: true
      label: "Passwordless minutes"
      description: "How long passwordless sudo stays on when you enable it."
      hint: "omarchy sudo passwordless"
      query: root.query
      keywords: ["sudo", "nopasswd", "passwordless", "minutes"]

      PrefsSlider {
        width: parent.width
        from: 5
        to: 120
        stepSize: 5
        value: Omarchy.sudoMinutes
        valueText: Omarchy.sudoMinutes + " min"
        onChanged: function(value) { Omarchy.sudoMinutes = Math.round(value) }
      }
    }

    SettingRow {
      label: "Passwordless sudo"
      description: Omarchy.passwordlessSudo
        ? "Sudo is not asking for a password right now. Turn it off early if you are done."
        : "Let this account run sudo without a password for the minutes above. Agents use this. So can anything else running as you."
      hint: "omarchy sudo passwordless"
      query: root.query
      keywords: ["sudo", "nopasswd", "passwordless", "agent"]

      Row {
        spacing: Theme.space
        PrefsButton {
          visible: !Omarchy.passwordlessSudo
          text: "Turn on…"
          enabled: !Omarchy.jobBusy && !Omarchy.passwordlessSudo
          onClicked: Omarchy.requestSudoMode()
        }
        PrefsButton {
          visible: Omarchy.passwordlessSudo
          text: "Turn off…"
          danger: true
          enabled: !Omarchy.jobBusy && Omarchy.passwordlessSudo
          onClicked: passwordlessOffConfirm.ask()
        }
      }
    }

    SettingRow {
      label: "Sudoless Docker"
      description: Omarchy.sudolessDocker
        ? "This account is in the docker group."
        : "Add this account to the docker group so Docker runs without sudo."
      hint: "omarchy setup security sudoless docker"
      query: root.query
      keywords: ["docker", "group", "sudoless"]

      Row {
        spacing: Theme.space
        PrefsButton {
          visible: !Omarchy.sudolessDocker
          text: "Set up…"
          enabled: !Omarchy.jobBusy && !Omarchy.sudolessDocker
          onClicked: dockerOnConfirm.ask()
        }
        PrefsButton {
          visible: Omarchy.sudolessDocker
          text: "Remove…"
          danger: true
          enabled: !Omarchy.jobBusy && Omarchy.sudolessDocker
          onClicked: dockerOffConfirm.ask()
        }
      }
    }
  }
}
