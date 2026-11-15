# Decisions

Status values: **accepted**, **proposed**, **open**.

## Accepted for the skeleton

### D-001 — Rust 1.100.0, edition 2024, 14-day minimum publish age

`rust-toolchain.toml` pins `1.100.0`; `Cargo.toml` declares `rust-version =
"1.100"`; `Cargo.lock` is committed. `unsafe_code` is forbidden in workspace
crates.

Cargo itself enforces a **14-day minimum publish age** through
`.cargo/config.toml` (`registry.global-min-publish-age = "14 days"`,
`resolver.incompatible-publish-age = "deny"`). This is the native Cargo 1.100
feature from RFC 3923, not a bot-only setting. `Cargo.lock` was regenerated
from scratch under this policy, so every registry version in it was published
at least 14 days ago. Dependabot uses `cooldown.default-days: 14` to match.

To take an urgent security fix that is younger than 14 days, use the explicit
one-off escape hatch:

```bash
CARGO_RESOLVER_INCOMPATIBLE_PUBLISH_AGE=allow cargo update -p <crate> --precise <version>
```

The pinned toolchain itself is the explicitly requested `1.100.0`; future
toolchain bumps follow the 14-day age rule.

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

### D-007 — Tray and background notifications in v1

Accepted: the app must remind while its window is closed. Linux uses `ksni`
(StatusNotifierItem over D-Bus), Windows uses a message-only window plus
`Shell_NotifyIcon`. OS notifications are part of the same milestone. GPUI does
not provide a tray API, so this is a small platform-specific module.

### D-008 — Email OTP authentication

Accepted: email OTP with a six-digit code entered in the app. No password, no
magic-link callback server. The Supabase session's refresh token goes into the
OS keychain; access tokens stay in memory.

### D-009 — Android is a later, separate client

Accepted: Android comes after the desktop sync loop is stable. It will share
`recurring-core` through UniFFI and use Compose for UI. GPUI is not a mobile
framework and will not be ported.

## Open

### Q-001 — Notification cadence

The 14-day rule is only about package publish age, not reminders. Still to
decide: default lead time, quiet hours, and whether a repeated daily reminder is
allowed until an event is marked done. Recommendation: respect `lead_days`,
notify at most once per day per event, and let the user snooze explicitly.

### Q-002 — Completion model details

Recommendation: append-only `completions` keyed by `(event_id, due_on)`, with
"Done" also advancing the visible next occurrence. Editing a schedule must not
rewrite history. Needs a concrete rule for early completion and for multiple
completions of the same occurrence.

### Q-003 — UI language

The skeleton is Russian-only. Recommendation: keep Russian as the default and
add a tiny keyed string layer when a second language appears; do not build a
translation framework for one user yet.

### Q-004 — Packaging and signing

Skeleton releases portable `.tar.gz` / `.zip` archives with SHA-256. Next:
Linux AppImage and Windows MSI/NSIS via `cargo-packager` or WiX. Code signing is
likely skipped for a personal app; document the SmartScreen warning.

### Q-005 — Supabase free-plan operations

Free projects pause after about a week without activity. Need a decision on a
keepalive workflow and on backups (no PITR on free). Recommendation: rely on
normal daily use, add a scheduled ping, and add app-level JSON export plus a
periodic encrypted dump.

### Q-006 — Windows screenshots

Linux has a real headless screenshot. Windows CI can build and test, but a
reliable screenshot needs an interactive desktop. Options are a self-hosted
runner, a Win32 offscreen capture harness, or visual review on a dev machine.
