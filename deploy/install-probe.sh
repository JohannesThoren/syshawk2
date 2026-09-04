#!/usr/bin/env bash
# Installs shawk-probe on this machine: builds the binary from source,
# creates a dedicated system user, writes the config, and installs +
# starts the systemd service.
#
# Usage: sudo ./deploy/install-probe.sh <server_url> <token> [interval_secs]
#
#   server_url     Base URL of shawk-server, e.g. https://syshawk.lgjt.xyz
#   token          Token returned when you registered this probe
#                  (POST /api/probes with X-Admin-Token)
#   interval_secs  Optional, defaults to 5
#
# Run this from inside a clone/copy of the syshawk2 repo.

set -euo pipefail

SERVER_URL="${1:?Usage: $0 <server_url> <token> [interval_secs]}"
TOKEN="${2:?Usage: $0 <server_url> <token> [interval_secs]}"
INTERVAL="${3:-5}"
INSTALL_DIR=/opt/shawk

if [ "$(id -u)" -ne 0 ]; then
  echo "Run this as root (sudo)." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f Cargo.toml ]; then
  echo "Couldn't find the syshawk2 repo root (no Cargo.toml next to deploy/). Run this from inside the repo." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "==> Rust not found, installing via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

echo "==> Building shawk-probe (release)..."
cargo build --release -p shawk-probe

echo "==> Creating 'shawk' system user..."
if ! id -u shawk >/dev/null 2>&1; then
  useradd --system --no-create-home --shell /usr/sbin/nologin shawk
fi

echo "==> Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
cp target/release/shawk-probe "$INSTALL_DIR/shawk-probe"

cat > "$INSTALL_DIR/probe.toml" <<CONF
server_url = "$SERVER_URL"
token = "$TOKEN"
interval_secs = $INTERVAL
top_processes = 15
CONF

chown -R shawk:shawk "$INSTALL_DIR"
chmod 700 "$INSTALL_DIR"
chmod 600 "$INSTALL_DIR/probe.toml"
chmod 755 "$INSTALL_DIR/shawk-probe"

echo "==> Installing systemd service..."
cp deploy/shawk-probe.service /etc/systemd/system/shawk-probe.service
systemctl daemon-reload
systemctl enable --now shawk-probe

echo
echo "Done. shawk-probe is running and reporting to $SERVER_URL"
echo "  status: systemctl status shawk-probe"
echo "  logs:   journalctl -u shawk-probe -f"
