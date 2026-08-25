# blackshark-linux

Linux userspace support for the Razer BlackShark V3 Pro, with tested support
for the Xbox edition on CachyOS/Arch, KDE Plasma, and Wayland.

This is a maintained fork of
[RiskRunner0/blackshark-linux](https://github.com/RiskRunner0/blackshark-linux).
It keeps the original daemon, D-Bus, CLI, tray, and GUI architecture while
adding Xbox-device support, reliable HID response matching, physical microphone
mute state, and wired USB transport detection.

## Supported devices

| Device | USB ID | Status |
| --- | --- | --- |
| BlackShark V3 Pro PC dongle | `1532:0577` | Supported by upstream; not independently tested in this fork |
| BlackShark V3 Pro Xbox dongle | `1532:0a55` | Supported and tested |
| BlackShark V3 Pro Xbox wired USB | `1532:0a4e` | USB audio handled by Linux; transport presence detected |

The daemon controls the headset through the wireless dongle. When the Xbox
headset is connected by cable, Linux exposes a separate USB-audio device and
the daemon reports the active transport as `usb`.

The wired HID interface does not answer the proprietary command protocol and
does not expose physical mute changes through HID or USB Audio. Wired physical
mute state is therefore reported as `unknown` rather than guessed.

## Features

- Battery percentage and charging state
- Physical microphone mute state over the wireless transport
- Wireless, wired USB, and disconnected transport state
- Sidetone level
- Nine EQ presets
- THX Spatial Audio toggle
- Active Noise Cancellation toggle and level
- Power-saving timeout
- Persistent configuration restored after reconnect
- D-Bus service with CLI, system tray, and Slint GUI clients
- Optional experimental PipeWire game/chat mix

## Requirements

- Linux with systemd user services
- Rust stable when building from source
- A user in the `users` group for HID access
- PipeWire or PulseAudio only for the optional game/chat mix feature

Firmware 1.3.x or newer is recommended for both headset and dongle. Older
firmware may expose the dongle without allowing the daemon to communicate with
the headset. See upstream [issue #1](https://github.com/RiskRunner0/blackshark-linux/issues/1).

## Install

Clone this fork and run the installer:

```bash
git clone https://github.com/Rafaelsk/blackshark-linux.git
cd blackshark-linux
./install.sh
```

The installer builds the workspace when prebuilt binaries are not present,
installs binaries under `~/.local/bin`, installs the user service, and installs
the udev rules using `sudo`.

If your user is not in the `users` group:

```bash
sudo usermod -aG users "$USER"
```

Log out and back in before reinstalling or reconnecting the dongle.

## Use

```bash
systemctl --user status blacksharkd
blackshark-ctl status
blackshark-ctl battery
blackshark-ctl sidetone <0-15>
blackshark-ctl eq <0-8>
blackshark-ctl thx <on|off>
blackshark-ctl anc <on|off> [level]
blackshark-ctl power-savings <0|15|30|45|60>
blackshark-ctl monitor
```

Run the desktop clients directly:

```bash
blackshark-tray &
blackshark-gui
```

The tray shows connection transport, microphone mute state, battery status,
and quick controls. The GUI exposes the full configuration and experimental
audio-routing tools.

Optional KDE application-launcher and tray-autostart entries are included:

```bash
install -Dm644 pkg/blackshark-gui.desktop \
  ~/.local/share/applications/blackshark-gui.desktop
install -Dm644 pkg/blackshark-tray.desktop \
  ~/.config/autostart/blackshark-tray.desktop
```

## Architecture

```text
blackshark-ctl  (CLI)  --+
blackshark-tray (tray) --+-- D-Bus: net.blackshark1
blackshark-gui  (GUI)  --+              |
                                    blacksharkd
                                         |
                                     /dev/hidraw*
                                         |
                              BlackShark wireless dongle
```

`blacksharkd` is the sole owner of the proprietary HID interface. Clients use
the session-bus service `net.blackshark1` at
`/net/blackshark1/Headset`; they do not access `hidraw` directly.

The wired Xbox USB interface is enumerated only to detect transport presence.
It is never opened as a proprietary command channel.

## Development

Run the same checks used by CI:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

This fork keeps `RiskRunner0/blackshark-linux` configured as the `upstream`
remote. New work should use feature branches and merge into this fork's
`master` after review and hardware testing.

## Credits

- Original project and architecture:
  [RiskRunner0/blackshark-linux](https://github.com/RiskRunner0/blackshark-linux)
- Xbox edition support:
  [callingmybluff](https://github.com/callingmybluff) via upstream
  [PR #11](https://github.com/RiskRunner0/blackshark-linux/pull/11)
- Maintained fork, HID fixes, mute state, and wired transport support: Rafael Robles

The Git history is intentionally preserved so authorship remains attached to
each contribution.

## License status

The upstream project currently has no explicit software license. Under default
copyright rules, a public repository should not be assumed to grant permission
for independent redistribution or derivative releases.

This repository remains a GitHub fork while licensing is clarified with the
upstream author and contributors. Do not publish independent release artifacts
from this fork until an explicit license is established.
