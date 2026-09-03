# Shawk 2.0

A spiritual successor to [syshawk](https://github.com/JohannesThoren/syshawk):
a probe + server + dashboard for monitoring your own servers, with a
remote terminal and dashboard login gated by a host Linux group.

## Components

- `crates/common` — shared Rust types for probe<->server communication.
- `crates/probe` — collects host metrics (CPU, RAM, disk, network, top
  processes) and pushes them to the server over HTTPS. Also holds a
  persistent control WebSocket back to the server so it can open an
  interactive shell (PTY) on request. Runs as a systemd service on each
  monitored machine.
- `crates/server` — receives snapshots, stores them in Postgres/TimescaleDB,
  detects when a probe goes silent, authenticates dashboard logins against
  host Linux accounts (PAM) gated by group membership, relays terminal
  sessions between the dashboard and a probe, and serves a REST + WebSocket
  API.
- `dashboard/` — Next.js dashboard: login screen, live server list,
  per-server metrics with sparkline history, top-processes table, and an
  in-browser terminal (xterm.js) per server.

## Why push, not poll

Probes push snapshots to the server rather than the server polling them.
Monitored machines only need outbound HTTPS - no inbound ports to open.
A missed heartbeat (no snapshot for 30s) marks a probe offline.

The remote terminal follows the same push model: the probe holds one
persistent outbound control connection to the server. To open a terminal,
the server tells the probe (over that existing connection) to dial back
with a dedicated PTY socket, which the server then relays byte-for-byte to
the dashboard. The monitored machine never accepts an inbound connection.

## Dashboard access control

Dashboard login authenticates against the host's own Linux accounts via
PAM (the same accounts used for SSH/console login on the machine running
`shawk-server`), and additionally requires the user to be a member of a
specific group - `syshawk` by default, overridable via `DASHBOARD_GROUP`.

This means `shawk-server` needs permission to read shadow entries, which
in practice means running it as root or granting it the `shadow` group
(e.g. via a systemd `SupplementaryGroups=shadow` directive) - same
approach as the docker-manager app.

To grant someone dashboard access on the host running the server:

```bash
sudo groupadd syshawk        # first time only
sudo usermod -aG syshawk someuser
```

**If running via Docker Compose, restart the `server` container after any
change to `/etc/passwd`, `/etc/group`, or `/etc/shadow`** - `groupadd`/
`usermod` write a new file and atomically swap it in rather than editing
in place, and Docker's per-file bind mount keeps pointing at the old
file's inode from when the container started. Until you restart, the
container won't see the change (you'll see `group does not exist on
this host` in the server logs if this happens):

```bash
docker compose restart server
```

Sessions are in-memory and cookie-based (12h expiry); restarting
`shawk-server` logs everyone out.

## Remote terminal

Each online server gets an "Open terminal" button in the dashboard,
opening a real shell (`$SHELL`, falling back to `/bin/bash`) on that
machine via the probe's PTY relay. Terminal access goes through the same
session-cookie auth as the rest of the dashboard - if you can see the
server, you can open a shell on it. There's currently no separate
per-server permission tier; anyone in the `syshawk` group can shell into
any monitored machine.

## Running it with Docker Compose

This is the easiest way to run the server + dashboard together, and is
close to how this is meant to be deployed on a real box.

```bash
cp .env.example .env
# edit .env: set POSTGRES_PASSWORD, ADMIN_TOKEN (openssl rand -hex 32),
# DASHBOARD_GROUP, and PUBLIC_API_URL (see the comments in .env.example -
# PUBLIC_API_URL must be reachable from the *browser*, not just the
# compose network)

docker compose up -d --build
```

This starts three containers: `db` (Postgres + TimescaleDB, migrations
run automatically), `server`, and `dashboard`. The dashboard is on
`:3000`, the API on `:8080` (both bound to `127.0.0.1` by default - put
your reverse proxy in front for real access).

**Dashboard login requires the `server` container to read the host's
`/etc/passwd`/`/etc/shadow`/`/etc/group`/`/etc/pam.d`**, which
`docker-compose.yml` already mounts in read-only, plus adds the container
to the host's `shadow` group so it can actually read shadow entries. This
means `docker compose` needs to run as a user that can add that group
mapping (root, or rootless Docker configured accordingly) - same
constraint as running `shawk-server` directly, just via a bind mount
instead of `SupplementaryGroups=`.

Register a probe the same way as below, against `http://<host>:8080`.

## Running it without Docker

### 1. Postgres + TimescaleDB

```bash
docker run -d --name shawk-db \
  -e POSTGRES_USER=shawk -e POSTGRES_PASSWORD=shawk -e POSTGRES_DB=shawk \
  -p 5432:5432 timescale/timescaledb:latest-pg16
```

### 2. Server

```bash
export DATABASE_URL="postgres://shawk:shawk@localhost:5432/shawk"
export ADMIN_TOKEN="pick-a-secret"      # gates probe registration
export DASHBOARD_GROUP="syshawk"        # host group allowed to log in
export BIND_ADDR="0.0.0.0:8080"
cargo run -p shawk-server
```

Migrations (hypertable + 30-day retention policy) run automatically.
Note: needs shadow-read access for login to work - see above.

### 3. Register a probe

```bash
curl -X POST http://localhost:8080/api/probes \
  -H "X-Admin-Token: pick-a-secret" \
  -H "Content-Type: application/json" \
  -d '{"name": "tellus"}'
```

Copy the returned `token` into `probe.toml` (see
`crates/probe/probe.toml.example`) alongside `server_url`.

### 4. Probe

```bash
cargo run -p shawk-probe -- crates/probe/probe
```

For a real deployment, build a release binary and install it with
`deploy/shawk-probe.service` (systemd).

### 5. Dashboard

```bash
cd dashboard
cp .env.local.example .env.local   # point NEXT_PUBLIC_API_URL at the server
npm install
npm run dev
```

**Important for local dev:** access the dashboard via the same hostname
your `NEXT_PUBLIC_API_URL` uses (e.g. both `localhost`, not one on
`localhost` and one on `127.0.0.1`) — browsers treat those as different
sites, which breaks the session cookie. In a real deployment, put the
dashboard and API behind the same public hostname via your reverse proxy
(e.g. `/` -> dashboard, `/api/*` -> shawk-server) so this isn't a concern
and cookies stay first-party.

Log in with any account on the server's host that's a member of the
`syshawk` group.

## Not yet built

- Alerting (thresholds, notifications) — the dashboard colors metrics by
  severity; nothing pages you yet.
- Per-server terminal permissions — currently any `syshawk` member can
  open a shell on any monitored server.
- Terminal session audit log (who opened a shell on what, when).
- Admin UI for registering probes (it's a raw curl call right now).
