# Reference service units for `ai-brainsd`

> **Honesty banner (T196 / F2)**  
> These files are **reference / operator copy-paste templates**. They are **not** a product Unix installer, **not** multi-user packaging, and **not** a T1 multi-OS service parity claim.  
> **Windows SCM** (`ai-brains daemon install` → service `AI-Brains-Daemon`) is the **only product-managed service install path**.  
> Automated installer management on Unix is **not** claimed. See [Docs/COMPATIBILITY.md](../../Docs/COMPATIBILITY.md) tiers and [Docs/RELEASE-CLAIMS.md](../../Docs/RELEASE-CLAIMS.md).

| Path | Role |
|------|------|
| [`systemd/ai-brainsd.user.service`](systemd/ai-brainsd.user.service) | **Primary Linux** — systemd **user** unit |
| [`systemd/ai-brainsd.system.service`](systemd/ai-brainsd.system.service) | **Secondary** system unit (honesty only — not recommended primary) |
| [`launchd/dev.ledgerful.ai-brainsd.plist`](launchd/dev.ledgerful.ai-brainsd.plist) | **Primary macOS** — LaunchAgent |
| [`launchd/ai-brainsd.wrapper.sh.example`](launchd/ai-brainsd.wrapper.sh.example) | Secrets-safe wrapper (source 0600 env → `exec`) |
| [`daemon.env.example`](daemon.env.example) | Sample env for systemd `EnvironmentFile` (no secrets) |

Soft static check: [`scripts/check-reference-units.sh`](../../scripts/check-reference-units.sh).

---

## Single-owner fence (ADR-0022)

AI-Brains is a **single-owner desktop** model. Units must not claim multi-user-safe IPC, shared tokens, or IdP product behavior. Prefer **user** units (systemd `--user`, launchd **LaunchAgent**) so the vault and UDS stay under the owner’s session identity.

---

## Binary path residual (cargo-bin / T145-class)

Default templates point at the **cargo install** path (`~/.cargo/bin/ai-brainsd`). That path is **user-writable by design**. Copying the binary into a system-managed directory is packaging/installer scope and is **not** done by these templates. Same residual class as Windows `ProgramData` binary-copy (T145).

Templates ship dual comments:

- cargo: `%h/.cargo/bin/ai-brainsd` / `$HOME/.cargo/bin/ai-brainsd`
- system: `/usr/local/bin/ai-brainsd` (or your package path)

Pick **one** live `ExecStart=` / `ProgramArguments` line for your install.

---

## Absolute vault path (headless / M5)

For headless or unit-managed starts, set **`AI_BRAINS_VAULT_PATH` to an absolute path**. Relative vault paths are not reliable under systemd/launchd (no shell profile, different CWD). `WorkingDirectory` is **not** a substitute for an absolute vault path.

The interactive daemon may fall back to `~/.ai-brains/vault.db` when the env is unset; **do not rely on that** for service units.

---

## XDG / UDS (T195 SOOT)

Daemon and CLI share `resolve_daemon_socket_path`:

1. Absolute `AI_BRAINS_DAEMON_SOCKET` (if set; relative → fail closed)  
2. Valid `$XDG_RUNTIME_DIR/ledgerful-bridge.sock` (absolute, mode `0700`, owned by euid; **AI-Brains does not create XDG**)  
3. `/tmp/ledgerful-bridge.sock` + runtime warning  

**Prefer leaving `AI_BRAINS_DAEMON_SOCKET` unset** when a valid XDG runtime dir exists (typical under systemd user sessions). Document the `/tmp` fallback residual (common on macOS without XDG). If you pin a fixed socket, set the **same absolute path** on daemon **and** CLI.

Post-bind mode is **0o600**. Pre-bind/shutdown unlink only **owned sockets**.

---

## HTTP (default off)

Templates **do not** enable HTTP. Leave `AI_BRAINS_HTTP` unset/off unless you intentionally opt in (loopback + bearer). **Never** set `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK` in these templates. Non-loopback bind is dangerous and not a product default.

Windows-only vars (`AI_BRAINS_PIPE_ACL`, `AI_BRAINS_HTTP_SERVICE`) do **not** belong in Unix units.

---

## Foreground / no-daemonize (M6)

`ai-brainsd` runs as a **foreground** async process under the supervisor (systemd / launchd). Do **not** wrap it in `daemon()`, double-fork, or `setsid` as a default under these units. Supervisors expect the main process to stay in the foreground (`Type=simple` / LaunchAgent).

---

## Signals: SIGINT vs SIGTERM (F36)

- **SIGINT** (`Ctrl-C`) always triggers the graceful shutdown path.  
- **SIGTERM** is what **systemd** and **launchd** send on stop. T196 soft-fixed the interactive/Unix path to race **SIGTERM alongside Ctrl-C** in `ai-brainsd` (no fork).  
- Windows SCM uses the service control path, not this Unix signal helper.

If you run a build older than that soft fix, stop via SIGTERM may not hit the graceful path — upgrade the binary or document the residual.

---

## systemd user unit (primary Linux)

### Install

```bash
mkdir -p ~/.config/systemd/user ~/.config/ai-brains
# Edit vault path (absolute) — never commit real keys
cp packaging/reference/daemon.env.example ~/.config/ai-brains/daemon.env
chmod 600 ~/.config/ai-brains/daemon.env
# Edit ExecStart if not using cargo bin
cp packaging/reference/systemd/ai-brainsd.user.service \
  ~/.config/systemd/user/ai-brainsd.service
systemctl --user daemon-reload
systemctl --user enable --now ai-brainsd.service
systemctl --user status ai-brainsd.service
```

### Linger tradeoff (M1)

| Mode | Behavior |
|------|----------|
| **Without** `loginctl enable-linger $USER` | User systemd instance starts at **login** and the daemon **stops on logout** (last session ends). |
| **With** linger | User manager (and enabled units) can **start at boot** and **survive logout**. |

Linger is **optional**. It is **not** auto-login and does not unlock encrypted home by itself. Only enable linger if you understand the security tradeoff for your host.

### Environment inheritance (F37)

User units **do not** inherit interactive shell profiles (`.bashrc`, etc.). Put paths and non-interactive config in `EnvironmentFile=` / `Environment=`, or deliberately `systemctl --user import-environment` selected vars.

`EnvironmentFile=-%h/.config/ai-brains/daemon.env` — the leading `-` means a **missing file is OK** for unit start, but **`AI_BRAINS_VAULT_PATH` is still required for reliable headless** operation (create the env file).

### Hardening defaults (user unit)

Live defaults: `NoNewPrivileges=true`, `PrivateTmp=true` only.

- **`ProtectHome=yes` and `ProtectHome=read-only` are intentionally not set** — the vault is typically under `$HOME` and needs write access.  
- **`ProtectSystem=strict` is commented off** with a `ReadWritePaths=` placeholder (vault parent + `%t`). Enabling strict without correct `ReadWritePaths` will break vault open.

Do not add relative `Documentation=file:packaging/...` — it resolves under `/`.

### Secondary system unit

[`systemd/ai-brainsd.system.service`](systemd/ai-brainsd.system.service) is **not** the recommended primary path and is **not** a product installer. It requires a non-root `User=` / `Group=` and explicit `ReadWritePaths=` for vault + runtime. Prefer the user unit.

---

## launchd LaunchAgent (primary macOS)

### Secrets pattern (L3/L4)

- Keep the LaunchAgent plist **user-owned**, mode **0600** or **0400** (not group/world writable).  
- Put **non-secret** env only in the plist (e.g. absolute vault path placeholder).  
- Put **secrets** (`AI_BRAINS_KEY` / daemon key vars) in a **0600 env file** sourced by [`launchd/ai-brainsd.wrapper.sh.example`](launchd/ai-brainsd.wrapper.sh.example), then `exec` the binary.  
- **Never** put secrets in a system-wide plist under `/Library/LaunchAgents` or LaunchDaemons.

### KeepAlive (M4)

Recommended plist uses a **dict**:

```xml
<key>KeepAlive</key>
<dict>
  <key>SuccessfulExit</key>
  <false/>
</dict>
```

That relaunches only on **failure** (non-zero exit).  

**Bare `KeepAlive` = true** is aggressive: rapid crash loops can cause launchd to **suspend** the job. If you choose bare true, ensure a **≥10s minimum lifetime** under normal start (or fix the crash) — otherwise expect suspension risk.

### Install steps

```bash
# 1) Logs dir
mkdir -p ~/Library/Logs/ai-brains

# 2) Wrapper + 0600 env (edit paths; never commit secrets)
mkdir -p ~/.config/ai-brains
cp packaging/reference/launchd/ai-brainsd.wrapper.sh.example \
  ~/.local/bin/ai-brainsd.wrapper.sh   # or another user-writable path
chmod 700 ~/.local/bin/ai-brainsd.wrapper.sh
# Create ~/.config/ai-brains/daemon.env (0600) with absolute AI_BRAINS_VAULT_PATH + key vars

# 3) Plist — replace REPLACE_ME home segments; keep mode 0600
cp packaging/reference/launchd/dev.ledgerful.ai-brainsd.plist \
  ~/Library/LaunchAgents/dev.ledgerful.ai-brainsd.plist
chmod 600 ~/Library/LaunchAgents/dev.ledgerful.ai-brainsd.plist

# 4) Load (modern launchctl)
launchctl bootstrap "gui/$(id -u)" ~/Library/LaunchAgents/dev.ledgerful.ai-brainsd.plist
launchctl enable "gui/$(id -u)/dev.ledgerful.ai-brainsd"
launchctl kickstart -k "gui/$(id -u)/dev.ledgerful.ai-brainsd"

# Unload
# launchctl bootout "gui/$(id -u)/dev.ledgerful.ai-brainsd"
```

Label: `dev.ledgerful.ai-brainsd`. Soft note: a future macOS app bundle may use a different reverse-DNS (e.g. `com.ledgerful.*`) — align Label if/when that ships.

Optional LaunchDaemon (system) templates are **out of primary guidance**; if you invent one, treat it like the Linux system unit — non-root identity, absolute vault, no secrets in system-wide plists, **not** product install.

---

## What these templates deliberately omit

| Omitted | Why |
|---------|-----|
| Product Unix `daemon install` CLI | Windows SCM only product path |
| MSI / App Store / notarization | Packaging residual — out of T196 |
| `Type=notify` / socket activation | No `sd_notify` |
| `AI_BRAINS_HTTP_ALLOW_NON_LOOPBACK` | Remote bind danger |
| Active `ProtectHome` on user unit | Breaks vault under `$HOME` |
| Multi-user safe IPC claims | ADR-0022 |
| systemd `LoadCredential` code path | Future (needs daemon support) |
| `WantedBy=graphical-session.target` | Future; v1 stays `default.target` |

---

## Related docs

- [Docs/OPERATIONS.md](../../Docs/OPERATIONS.md) — daemon lifecycle  
- [Docs/INSTALL.md](../../Docs/INSTALL.md) — install how-to  
- [Docs/COMPATIBILITY.md](../../Docs/COMPATIBILITY.md) — OS tiers  
- [Docs/SECURITY-LIMITS.md](../../Docs/SECURITY-LIMITS.md) — honest non-claims  
- [Docs/DECISIONS/ADR-0022-single-owner-daemon-ipc-fence.md](../../Docs/DECISIONS/ADR-0022-single-owner-daemon-ipc-fence.md)  
- Root [CONTRIBUTING.md](../../CONTRIBUTING.md)  
