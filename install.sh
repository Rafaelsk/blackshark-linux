#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# blackshark-ctl installer
# If pre-built binaries are present alongside this script, installs them
# directly. Otherwise builds from source (requires Rust/cargo).
# ---------------------------------------------------------------------------

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
UDEV_DIR="/etc/udev/rules.d"
BINARIES=(blacksharkd blackshark-ctl blackshark-tray blackshark-gui)

# Colours
RED='\033[0;31m'
GREEN='\033[0;32m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${BOLD}==> $*${NC}"; }
ok()    { echo -e "${GREEN}    ok${NC}"; }
die()   { echo -e "${RED}error: $*${NC}" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Detect whether pre-built binaries are present
# ---------------------------------------------------------------------------

PREBUILT=true
BINARY_DIR="$REPO_DIR"

# A Git checkout should build the current source even if stale binaries happen
# to exist in the repository root. Release archives may include pre-built
# binaries alongside this script.
if [[ -e "${REPO_DIR}/.git" ]]; then
    PREBUILT=false
else
    for bin in "${BINARIES[@]}"; do
        if [[ ! -x "${REPO_DIR}/${bin}" ]]; then
            PREBUILT=false
            break
        fi
    done
fi

# ---------------------------------------------------------------------------
# Checks
# ---------------------------------------------------------------------------

info "Checking dependencies"

if [[ "$PREBUILT" == false ]]; then
    command -v cargo >/dev/null 2>&1 || die "cargo not found — install Rust from https://rustup.rs, or download a pre-built release"
fi
command -v systemctl >/dev/null 2>&1 || die "systemctl not found — this installer requires systemd"

# Warn if the user is not in the 'users' group (needed for the udev rule).
if ! id -nG "$USER" | grep -qw users; then
    echo -e "${RED}warning: you are not in the 'users' group.${NC}"
    echo "         The udev rule grants GROUP=users access to the HID device."
    echo "         Run: sudo usermod -aG users \$USER"
    echo "         Then log out and back in, and re-run this script."
    echo ""
fi

ok

# ---------------------------------------------------------------------------
# Build (only if no pre-built binaries found)
# ---------------------------------------------------------------------------

if [[ "$PREBUILT" == true ]]; then
    info "Using pre-built binaries"
    ok
else
    info "Building release binaries (this may take a minute)"
    cd "$REPO_DIR"
    cargo build --release -p blacksharkd -p blackshark-ctl -p blackshark-tray -p blackshark-gui
    BINARY_DIR="${REPO_DIR}/target/release"
    ok
fi

# ---------------------------------------------------------------------------
# Install binaries
# ---------------------------------------------------------------------------

info "Installing binaries to ${BIN_DIR}"
mkdir -p "$BIN_DIR"

for bin in "${BINARIES[@]}"; do
    install -m755 "${BINARY_DIR}/${bin}" "${BIN_DIR}/${bin}"
    echo "    ${BIN_DIR}/${bin}"
done
ok

# Make sure ~/.local/bin is on PATH (common omission on fresh systems).
if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo ""
    echo -e "${RED}warning: ${BIN_DIR} is not in your PATH.${NC}"
    echo "         Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo "           export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
fi

# ---------------------------------------------------------------------------
# Systemd user service
# ---------------------------------------------------------------------------

info "Installing systemd user service"
mkdir -p "$SYSTEMD_DIR"
install -m644 "${REPO_DIR}/pkg/blacksharkd.service" "${SYSTEMD_DIR}/blacksharkd.service"
echo "    ${SYSTEMD_DIR}/blacksharkd.service"

systemctl --user daemon-reload
systemctl --user enable blacksharkd
systemctl --user restart blacksharkd
echo "    enabled and started blacksharkd"
ok

# ---------------------------------------------------------------------------
# udev rule (requires sudo)
# ---------------------------------------------------------------------------

info "Installing udev rule (requires sudo)"
sudo install -m644 "${REPO_DIR}/pkg/99-blackshark.rules" "${UDEV_DIR}/99-blackshark.rules"
echo "    ${UDEV_DIR}/99-blackshark.rules"
sudo udevadm control --reload-rules
sudo udevadm trigger
echo "    udev rules reloaded"
ok

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------

echo ""
echo -e "${GREEN}${BOLD}Installation complete.${NC}"
echo ""
echo "  Next steps:"
echo ""
echo "  1. Plug in the USB dongle if it isn't already."
echo ""
echo "  2. Check the daemon is running:"
echo "       systemctl --user status blacksharkd"
echo ""
echo "  3. Verify the headset is detected:"
echo "       blackshark-ctl status"
echo ""
echo "  4. Start the system tray:"
echo "       blackshark-tray &"
echo ""
echo "  5. Open the settings GUI:"
echo "       blackshark-gui"
echo ""
echo "  To start the tray automatically on login, add it to your desktop"
echo "  autostart. On KDE: Settings -> Autostart -> Add Application."
echo ""
echo "  If the headset is not detected, replug the USB dongle after the"
echo "  udev rules have been applied."
