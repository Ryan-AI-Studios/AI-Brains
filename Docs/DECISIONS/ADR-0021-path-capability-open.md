# ADR-0021: Path Capability Open (cap-std + component nofollow)

## Status

**Accepted** — 2026-08-02.

Normative for **T190** (Path Hardening / TOCTOU). Satisfies the [ADR-0019](ADR-0019-connector-sandbox-execution-model.md) **L2** requirement that production `cap-std` needs a **new** track + ADR. Complements ADR-0019 path residual honesty (L10): TrustedBuiltin vault-relative open+list is hardened; product-wide ambient path TOCTOU is **not** claimed closed.

Freezes cited: **F1–F32**, acceptance **AC1–AC13** — see
[`conductor/tracks/trackT190-path-toctou-hardening/spec.md`](../../conductor/tracks/trackT190-path-toctou-hardening/spec.md).

## Context

P6 / T154 shipped vault containment + reparse refuse, but `vault_fs::read_file_under_root`
and `obsidian::walk_vault` still used check-then-ambient-open (`std::fs::read` /
`std::fs::read_dir`). That class of TOCTOU is residual **#12** / RELEASE-CLAIMS **R-12**.

Research (2026-08-02) on **cap-std 4.0.2**:

| Fact | Implication |
|------|-------------|
| `Dir::open` / `Dir::open_dir` are **containment**, not nofollow | Bare relative open **follows** symlinks — insufficient for product zero-reparse policy (F9) |
| Public `OpenOptions` has no `follow_symlinks(false)` | Must use platform flags / component walk (F27 / F32) **plus** `FollowSymlinks::No` |
| cap-primitives default `follow = Yes` | OS `O_NOFOLLOW` alone is **insufficient** on the manual resolver (macOS; Linux without openat2): software `readlink`+reopen defeats custom flags |
| Fix | `cap-fs-ext::OpenOptionsFollowExt::follow(FollowSymlinks::No)` on every nofollow open |
| Unix: `OpenOptionsExt::custom_flags` | `O_NOFOLLOW` (+ `O_DIRECTORY` for dirs) via rustix `OFlags` (defense-in-depth with FollowSymlinks::No) |
| Windows: open-time reparse control | `FILE_FLAG_OPEN_REPARSE_POINT` (+ `FILE_FLAG_BACKUP_SEMANTICS` for directories); refuse if handle has reparse attribute |
| License | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT — deny-compatible (cap-std + cap-fs-ext) |

## Decision

### 1. Allow production `cap-std` 4.0.x (F1, F13, F14)

Workspace pins: `cap-std = "4.0"` (researched **4.0.2**) and `cap-fs-ext = "4.0"` (FollowSymlinks). Production consumers:

- `ai-brains-path` — shared open helpers (`cap-std` + `cap-fs-ext`)
- `ai-brains-sources` — vault_fs, walk_vault, Hermes/Honcho export loaders

Documented transitives include `cap-primitives`, `io-lifetimes`, `io-extras`, and on Unix `rustix` (fs). Windows may pull `windows-sys` alongside workspace `windows` 0.62 (multiple-versions warn is acceptable when deliberate).

### 2. SOOT open mechanism (F8, F27, F32)

**Single source of truth = component walk + platform open-time refuse:**

1. **Lexical** resolve still in `vault_fs` (reserved stems, `..` refuse) — F31.
2. `Dir::open_ambient_dir(root, ambient_authority())` once for the trusted root — F21.
3. **Per component** (not multi-segment open):
   - **All platforms:** `cap_fs_ext::OpenOptionsFollowExt::follow(FollowSymlinks::No)` so cap-primitives does **not** software-follow after an OS nofollow probe (F27 correction).
   - **Unix:** also `custom_flags(O_NOFOLLOW)`; directories also `O_DIRECTORY`. Map ELOOP / “Too many levels of symbolic links” → `ReparseRefused`. Prefer `rustix::fs::OFlags::NOFOLLOW`.
   - **Windows:** also open each component with `FILE_FLAG_OPEN_REPARSE_POINT` (dirs also `FILE_FLAG_BACKUP_SEMANTICS`) via cap-std `OpenOptionsExt::custom_flags`. If the opened handle has reparse attribute (or is symlink), close and refuse. Successful regular dir handle → `cap_std::fs::Dir::from_std_file`.
4. On final file handle: handle-bound `metadata()` for size/`is_file` (F29); capped read on the **same** handle — never ambient `std::fs::read(path)` after open.
5. **F26:** Dir/open failure → error; **never** silent ambient `std::fs::read` fallback.

Shared API lives in `ai-brains-path` (`cap_open` module): `open_ambient_vault_dir`,
`read_file_nofollow_components`, `open_dir_component_nofollow`, `list_entry_names`, etc.

### 3. Scope (F4, F4b, F5, F6)

| In scope (primary) | Behavior |
|--------------------|----------|
| `vault_fs::read_file_under_root` | Cap open only |
| `obsidian::walk_vault` | `Dir` entries + nofollow descent; **zero** `std::fs::read_dir` |
| Hermes / Honcho path export loaders | Elevated to shared list+read helpers |

| Evaluate / residual | Disposition |
|---------------------|-------------|
| T188 `artifact_security` write + kit write | **Elevated (T193):** shared write SOOT (`CreateMode::CreateNew` \| `Replace`); parent reparse + ACL order retained |
| `ai-brains-api-server` token load/write | **Elevated (T193):** ambient parent Dir → nofollow leaf read/write + owner ACL |
| Migrate report/manifest, shadow-manifest, dogfood/evaluate reports | **Elevated (T193 P1)** via `write_file_nofollow_under_parent_path` |
| Soft-canonicalize | **Non-claim** for TOCTOU (F10) — permanent residual |
| Parent `create_dir_all` chain; backup dest tree | **Residual** (F26 / L3); backup adds parent reparse refuse only |
| Plugin WASI / ambient CLI long-tail | **Non-claim** (inventory residual; see T193 spec §13) |

### 4. Non-claims

This ADR does **not** claim:

- Plugin or third-party connector isolation
- Perfect TOCTOU closure on every ambient Windows/CLI path
- Soft-canonicalize as a security open gate
- That cap-std default open is nofollow (it is not)
- Product-wide “path TOCTOU closed” marketing language beyond TrustedBuiltin vault open+list

### 5. Errors (F16)

Map capability errors to `VaultFsError`: `ReparseRefused`, `PathEscape`, `Oversized`,
`NotFound`, `Io` (and related NotAFile → Io for callers).

## Consequences

### Positive

- Closes check-then-open TOCTOU on Obsidian vault read + list and Hermes/Honcho export dirs.
- ADR-0019 L2 carve-out for cap-std is satisfied with an explicit decision record.
- Zero-symlink product policy preserved (F9) via open-time refuse, not containment-follow.

### Negative / residual

- Soft-canon, parent `create_dir_all`, ambient CLI long-tail, perfect Windows all-API TOCTOU remain honesty residuals (T193 residual register).
- **Write path elevated (T193 short amend):** P0 artifact/token/kit + P1 operator reports use shared nofollow write SOOT; never claim product-wide write TOCTOU closed.
- Windows junction final-as-file may fail as `Io` (access denied) rather than typed reparse — still fail closed.
- Multiple `windows-sys` versions may warn under `cargo deny` bans (warn level).

## Implementation notes (T190)

| Work | Required? |
|------|-----------|
| This ADR Accepted in-tree | Yes |
| `ai-brains-path` cap_open helpers + proof tests | Yes |
| Wire vault_fs + walk_vault | Yes |
| Hermes/Honcho elevate or residual | Elevate preferred (done) |
| R-12 / SECURITY-LIMITS / deferred #12 honesty | Yes |
| SECURITY cross-model review | Required (orchestrator) |

## Relationship to ADR-0019

- **L2:** production `cap-std` allowed for TrustedBuiltin path hardening under **this** ADR.
- **L10 residual #12:** rewritten as **closed-with-residuals** for connector vault open+list (T190) and high-risk write/token surfaces (T193); soft-canon / ambient long-tail / parent mkdir residuals remain.
- Pointer: ADR-0019 §4 path residual → T190 / ADR-0021; write residual elevation → T193.
