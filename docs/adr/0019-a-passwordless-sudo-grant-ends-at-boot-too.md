# 0019. A passwordless sudo grant ends at boot too

Status: Accepted. Date: 2026-10-26.

## Context

Turning passwordless sudo on (`atmosd root sudo-on`) writes `/etc/sudoers.d/99-omarchy-nopasswd-<user>` and arms a transient `systemd-run` timer that removes it. A transient timer does not survive a reboot, the drop-in does, and the window reads "on" from that timer alone: `/etc/sudoers.d` is `root:root` mode 0750, so the user cannot look for the file. After a reboot the grant could stay on with no end, and the window would show it off.

Omarchy's own `omarchy-sudo-passwordless` meets this with `/etc/tmpfiles.d/omarchy-nopasswd-sudo.conf` (`r! /etc/sudoers.d/99-omarchy-nopasswd-*`) and tells the user that a restart removes the rule. Atmos writes the grant itself, because that command asks a question on a terminal, so it cannot rely on a file that another package put there, which an older or changed Omarchy may not have.

## Decision

`sudo-on` writes its own boot rule before it writes the grant, `/etc/tmpfiles.d/atmos-nopasswd-<user>.conf`, mode 0644:

```
r! /etc/sudoers.d/99-omarchy-nopasswd-<user>
r! /etc/tmpfiles.d/atmos-nopasswd-<user>.conf
```

- `r!` lines run only when `systemd-tmpfiles` is given `--boot`, which `systemd-tmpfiles-setup.service` does at early boot (`--create --remove --boot`), so a later `--remove` cannot cut a live grant short. The second line removes the rule itself. Both were run for real with systemd 262 in a root of their own: a run without `--boot` leaves the grant, a run with it removes the grant and the rule.
- The timer removes the rule with the grant (`rm -f -- <grant> <rule>`), and `sudo-off` removes both.
- If the grant or the timer cannot be put in place, the grant and the rule are removed and the task fails: a grant with no end is not left on.
- The rule names the exact file, not a pattern, so it does not repeat Omarchy's line; two files that name the same path would be a conflict in tmpfiles.d, and these do not.
- The probe is unchanged. It still reads the timer in `waiting` state (ADR 0016's rule that a reading wins over a stored value stands): the rule file cannot say whether the grant is still there, because `omarchy-sudo-passwordless` removes the grant and leaves a rule behind.

## Consequences

- A reboot ends every grant that Atmos made, with or without Omarchy's rule.
- `/etc/tmpfiles.d` gets one small file while a grant is on, and none after it ends. A rule left by a crash is removed at the next boot by its own second line.
- The grant that survives a lost timer without a reboot (the timer stopped by hand) is still shown as off. That is not fixed here; the window cannot see the file.
- Not seen on a real reboot; the boot behaviour is `systemd-tmpfiles` run with `--boot` in a throwaway root.

## Alternatives considered

- **Rely on Omarchy's `omarchy-nopasswd-sudo.conf`.** It is there on this machine and not in Atmos's control.
- **A persistent timer unit or a boot service that removes the grant.** It adds a unit to install and remove; tmpfiles is the mechanism systemd offers for removing a file at boot.
- **Show "on" when the rule file exists.** The rule outlives a grant ended by `omarchy-sudo-passwordless`, so the window would show "on" for a grant that is gone.
- **A sudoers expiry inside the drop-in.** `NOTAFTER` works only for sudoers kept in LDAP.

## Enforced by

- `backend/src/tasks/root.rs` writes the rule first and tests it with the real `systemd-tmpfiles` (`systemd_tmpfiles_removes_the_grant_at_boot_and_not_before`).
