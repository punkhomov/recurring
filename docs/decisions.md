# Decisions

Status values: **accepted**, **proposed**, **open**.

## Accepted for the skeleton

### D-001 — Rust 1.100.0, edition 2024

`rust-toolchain.toml` pins `1.100.0`; `Cargo.toml` declares `rust-version =
"1.100"`; `Cargo.lock` is committed. `unsafe_code` is forbidden in workspace
crates.

The requested 7-day cooldown is implemented for dependency and GitHub Actions
updates in `.github/dependabot.yml` (`cooldown.default-days: 7`). The initial
toolchain is the explicitly requested `1.100.0`; future toolchain bumps follow
the same "wait a week after release" rule.

### D-002 — GPUI 0.2.2 from crates.io

Pinned to the published `0.2.2` release, not a moving Zed `main` revision.
GPUI is pre-1.0, so upgrades are deliberate PRs, not automatic merges. The
`xattr`/`libc` incompatibility is contained by pinning `libc = 0.2.186`; that
pin should be removed when GPUI's dependency chain updates.

### D-003 — `core` / `app` split

The recurrence engine and data types live in `recurring-core`, with no UI,
network, or database dependencies. This keeps Android and CLI/test surfaces
possible without GPUI.

### D-004 — Calendar-first recurrence

Schedules are calendar rules with an anchor. Month-end clamps without drifting;
yearly 29 February falls back to 28 February in common years. Completion-based
rules ("90 days after I actually did it") are a later addition, not mixed into
the current model.

### D-005 — Offline-first, encrypted-token sync

SQLite is the local source of truth; Supabase is a sync target. Planned merge is
per-row LWW with an HLC and tombstones. No Realtime subscription in v1; polling
on start/focus/interval is enough.

### D-006 — Headless screenshots as a CI gate

The screenshot job runs the actual GPUI binary under Xvfb + lavapipe + Openbox
and uploads a PNG. This tests the real renderer, fonts, and layout rather than a
mock. Golden-image comparison is proposed once the visual design settles.

## Open

### Q-001 — What the 7-day cooldown applies to

Assumed: dependency/toolchain updates (implemented above). If it also means
"one notification per event at most every 7 days", that is a different
production rule and belongs in the reminder engine. The current model has
`lead_days` but no notification cooldown yet.

### Q-002 — Auth method

Recommendation: email OTP (6-digit code entered in the app), not magic links and
not a password. It avoids a loopback callback server and is fine for one user.
Decision needed before sync work.

### Q-003 — Background reminders and tray

Recommendation: v1 includes a tray icon and OS notifications, because a tracker
that only reminds while its window is open is not useful. Linux uses `ksni`;
Windows uses a message-only window + `Shell_NotifyIcon`. This is the largest
platform-specific chunk and should be its own milestone.

### Q-004 — Completion model details

Recommendation: append-only `completions` keyed by `(event_id, due_on)`, with
"Done" also advancing the visible next occurrence. Editing a schedule must not
rewrite history. Needs a concrete rule for early completion and for multiple
completions of the same occurrence.

### Q-005 — UI language

The skeleton is Russian-only. Recommendation: keep Russian as the default and
add a tiny keyed string layer when a second language appears; do not build a
translation framework for one user yet.

### Q-006 — Packaging and signing

Skeleton releases portable `.tar.gz` / `.zip` archives with SHA-256. Next:
Linux AppImage and Windows MSI/NSIS via `cargo-packager` or WiX. Code signing is
likely skipped for a personal app; document the SmartScreen warning.

### Q-007 — Supabase free-plan operations

Need a decision on a weekly keepalive (to avoid project pause) and on backups
(free plan has no PITR). Recommendation: keepalive via a scheduled workflow,
plus app-level JSON export and a periodic encrypted dump.

### Q-008 — Android timing

Recommendation: after the desktop sync loop is stable. Share `recurring-core`
via UniFFI and build a Compose client; do not attempt to run GPUI on Android.

### Q-009 — Windows screenshots

Linux has a real headless screenshot. Windows CI can build and test, but a
reliable screenshot needs an interactive desktop. Options are a self-hosted
runner, a Win32 offscreen capture harness, or visual review on a dev machine.
