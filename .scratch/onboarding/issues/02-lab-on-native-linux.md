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

## Comments

- `kdx-lab setup` now falls back to `ar` and `tar` when `dpkg-deb` is
  missing; it handles `data.tar`, `.gz`, `.xz`, `.zst`, and `.bz2`. The
  download needs only `curl` and `sha256sum`. Tested against `dpkg-deb -x`
  on small hand-built `.deb` files (gzip, xz, none); not yet run on Fedora or
  Arch.
- `lab/README.md` now lists Fedora and Arch package names, and notes the
  AppArmor restriction on native Ubuntu 23.10 and later as untested.
- Remaining: the AppArmor fix, which needs a native Ubuntu host, and a real
  run of the lab on Fedora and Arch.
