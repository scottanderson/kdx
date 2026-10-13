# KDX lab

Runs the original Haxial binaries (see `vendor/MANIFEST.md`) and captures their
traffic. Docker isn't needed. The isolation comes from two layers:

- **Network.** `kdx-lab up` creates a private user and network namespace that
  only has `lo`. Nothing inside it can reach the host network or the internet.
  `dumpcap` records everything on that loopback interface to
  `captures/<scenario>.pcapng`.
- **Filesystem.** Each program runs in its own `bwrap` sandbox. It sees a
  read-only `/usr` and `/etc`, a private `/tmp`, and one writable state
  directory mounted at `/kdx` (`lab/run/<component>/`). The binary is copied
  fresh into that directory on every start.

The GUI clients also get the WSLg X socket (`/tmp/.X11-unix/X0`). X11 offers no
isolation between its clients, so don't type anything sensitive into other
windows while a client is running.

Requires: `bwrap`, `unshare`/`nsenter`, `dumpcap`/`tshark`, 32-bit Wine (for
the Windows client), and unprivileged user namespaces. On Ubuntu 24.04:

```sh
sudo dpkg --add-architecture i386 && sudo apt update
sudo apt install bubblewrap tshark unar curl wine wine32:i386
```

The lab has been tested only on Ubuntu 24.04 under WSL2 with WSLg. `client`
needs the WSLg X socket, and native Ubuntu may block unprivileged user
namespaces through AppArmor.

The server and tracker run natively. Haxial released Client 1.600, the
reference client so far, only for Windows and Mac, so it needs Wine. The
Linux client (`client linux`) is Client 1.620 beta 5: its about string reads
"KDX 1.620", and it was built in December 2004, after 1.600. Its strings show
the same features as 1.600, without the Windows tray options. It has not yet
been run in the lab.

Fetch the binaries first with `vendor/fetch` (see `vendor/MANIFEST.md`).

## Usage

```sh
lab/kdx-lab setup            # once: fetch + checksum the i386 libstdc++.so.5 from archive.debian.org
lab/kdx-lab up login-guest   # start the network and capture to captures/login-guest.pcapng
lab/kdx-lab server           # foreground; console accepts: exit, quit, about, info, setport
lab/kdx-lab server -d        # or detached, logging to lab/run/server.log
lab/kdx-lab server win -d    # KDX Server 1.600 for Windows under Wine, logging to lab/run/server-win.log
lab/kdx-lab client           # KDX Client 1.600 under Wine (or: client linux, for 1.620 beta 5)
lab/kdx-lab shell            # shell inside the lab network (nc, tshark, ss, python3 -I ...)
lab/kdx-lab down             # stop capture, kill everything in the lab network
```

In the client, connect to `127.0.0.1` (or `0`). The default port is 10700.

`up` also writes `captures/<scenario>.md`. Fill in its steps and observations,
because a capture without its GUI steps is hard to interpret later. Use one
scenario per feature you're exercising, and keep scenarios small.

## Server first run

The first `kdx-lab server` (foreground) runs the console setup:
press Enter, then give the admin login, the password twice, and the guest
access answer (`yes`/`no`). The lab convention is login `admin`, password
`lab-only-pw!1`, guest access `yes`. The state persists in `lab/run/server/`
(`Accounts.dat`, `KDXServer.stg`, `News.dat`, `Bases/Default/`). Use
`kdx-lab reset server` to start over.

## Server upgrade

`kdx-lab server` copies `KDXUpgradeHelper.lexe` next to the server, and the
sandbox stays up after the server exits until the helper has exited too, so
175 Upgrade Server Software can relaunch the server. The new server goes in
`lab/run/server/Bases/KDXServer.lexe` (Haxial's `Documentation.pdf`,
"Upgrade Server Software"); the helper moves it over the server's program.
The next `kdx-lab server` puts back 1.620.

## Windows server

`kdx-lab server win` runs Server 1.600 for Windows under Wine, with its state
in `lab/run/server-win/`. It is a GUI program, so it needs the X socket like
the clients. Use it for what the Linux server can't do, such as view display:
Wine gives it a black screen as large as the host's display. The first run
shows a dialog for the admin login, password, and guest access; it can stay
on screen after Save, and saving it twice makes the server report that it
can't listen on the port, although the first listener keeps running.

## Tracker first run

The first `kdx-lab tracker` (foreground) runs a similar console setup: press
Enter, then give the admin login and the password twice. There is no guest
question; guests can always browse. The lab convention is login `admin`,
password `lab-tracker-pw!1`. The state persists in
`lab/run/tracker/KDXTrackerServ.stg`. Client 1.620 beta 5 can't change the
admin password remotely (`docs/protocol/tracker.md`, 712), so use `kdx-lab
reset tracker` to change it.

## Things to know

- The client speaks first. The server sends nothing until the client's first
  packet. If it can't parse that packet, it logs
  `The network protocol is unknown. (ID 4/113 @ TRN941)` and disconnects.
  These `ID x/y @ TRNnnn` codes look like internal error-site identifiers and
  should help with static analysis.
- Wine also probes CUPS (`127.0.0.1:631`). Filter captures with
  `tcp.port == 10700`.
- On first launch, the Windows client shows Haxial's license agreement.
  Accepting it is the operator's decision. Clause 3 prohibits reverse
  engineering the software "or its transmission protocols (except where
  permitted by law)".
