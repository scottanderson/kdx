# Run the lab on native Linux and non-Debian systems

Status: needs-triage

The lab has run only on Ubuntu 24.04 under WSL2.

- Native Ubuntu 23.10 and later restrict unprivileged user namespaces
  through AppArmor (`kernel.apparmor_restrict_unprivileged_userns`), which
  may stop `kdx-lab up` (`unshare --user`) or `bwrap`. Find the least
  invasive fix, such as an AppArmor profile, and document it.
- `kdx-lab setup` unpacks libstdc++5 with `dpkg-deb`. Other distributions
  need another way, e.g. `ar` and `tar`.
- Give package names for Fedora and Arch in `lab/README.md`.
