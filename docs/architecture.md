# Architecture

## Shape

```
crates/core   pure Rust domain + recurrence engine (no UI, no network, no DB)
crates/app    GPUI desktop client for Linux and Windows
supabase/     SQL migrations and cloud setup notes
scripts/      Linux dependencies, headless screenshot capture
```

`recurring-core` is deliberately platform-independent. It is the piece that
will later be shared with an Android client (through UniFFI) without dragging
GPUI along. The dependency direction is always `app → core`, never the reverse.

## Domain model

- **Event** — a recurring thing: title, category, optional exact amount,
  schedule, IANA time zone, reminder time, lead time, soft-delete marker.
- **Schedule** — a validated recurrence anchored at a date:
  - once;
  - every N days;
  - every N weeks on selected weekdays;
  - every N months on a day (or the last day);
  - every N years on a month/day.
- **Completion** (cloud/DB, not yet wired) — `(event_id, due_on)` plus when it
  was actually completed. This is append-only history, not a mutable checkbox.

Calendar rules are evaluated from the anchor, not from the previous result.
That is why "31st" becomes 28 February but returns to 31 March instead of
drifting to 28 forever. Leap days clamp the same way.

Dates are timezone-free in the recurrence engine. The event's time zone is
applied only when an occurrence becomes a notification timestamp.

## Local-first and sync

Planned data path:

```
GPUI views
   │
   ├── in-memory view model (current skeleton)
   │
   └── SQLite (rusqlite, background thread)
          │  outbox
          ▼
       Supabase REST (reqwest)  ← polling on start, focus, and every N minutes
```

- **Identity**: Supabase Auth, single user. Email OTP is the accepted login:
  no browser callback server or deep-link plumbing is needed.
- **Secrets**: refresh token in the OS keychain (`keyring`): Secret Service on
  Linux, Credential Manager on Windows. Never in the repo or SQLite.
- **Merge**: per-row last-writer-wins using a hybrid logical clock (`hlc`),
  plus tombstones for deletes. HLC avoids trusting either laptop's wall clock.
  Completions are idempotent by `(event_id, due_on)`.
- **Pull**: `user_id` + `hlc > cursor`, ordered by HLC, with a page limit.
- **Push**: an outbox table in SQLite, retried with backoff.
- **Offline**: all reads and writes go to SQLite first. Sync failures never
  block the UI.
- **Realtime**: intentionally not used. Polling is simpler, battery-friendlier
  for this workload, and avoids a second connection state machine.

The initial SQL lives in `supabase/migrations`. The Rust client will speak the
small set of PostgREST/GoTrue endpoints directly instead of depending on an
unmaintained all-in-one SDK.

## UI

GPUI renders with Blade over Vulkan on Linux and Direct3D on Windows. The app
uses a small design-token module instead of a theme framework, a 236 px
sidebar, a list of event rows, and in-memory demo data until SQLite lands.

The window is a normal desktop window. Tray icon and OS notifications are a v1
requirement, implemented as a separate layer: `ksni` (StatusNotifierItem over
D-Bus) on Linux and a message-only window + `Shell_NotifyIcon` on Windows. GPUI does not ship a
tray API, and GTK-based tray crates would fight GPUI's event loop, so this needs
its own small platform module.

## Testing

Layers, cheapest first:

1. `cargo test -p recurring-core` — recurrence math, validation, JSON round
   trips. Property tests for month ends/DST are a planned addition.
2. `cargo test -p recurring-app` — Russian formatting and, later, GPUI
   `TestAppContext` interaction tests with a frozen clock.
3. **Headless screenshot** — the real binary runs under Xvfb with Mesa's
   lavapipe software Vulkan; Openbox maps the window; ImageMagick captures the
   client area at a fixed 1280×800 with `RECURRING_TODAY=2026-11-15`. CI uploads
   the PNG plus app/Xvfb/Openbox logs.
4. **Windows** — CI builds and runs tests; a real screenshot harness on Windows
   is still an open question. GUI capture needs an interactive desktop, which
   GitHub-hosted runners do not provide reliably.
5. **Cloud** — later: Supabase CLI + Docker in CI for migrations and a two
   client merge test.

Screenshot determinism comes from a pinned date, fixed window size, system
fonts present in CI (`fonts-dejavu-core`), no animations, and waiting for the
first frames before capture. Golden-image comparison is not enabled yet; the
artifact is reviewed manually until the UI stabilizes.

## Android

GPUI is a desktop framework. The Android path is a separate Compose client
using the same `recurring-core` through UniFFI, with Kotlin owning UI,
background work, and notifications. Do not try to port GPUI to Android; that
would couple the mobile schedule to Zed's desktop release cadence.

## Operational constraints

- Cargo resolves only crate versions published at least 14 days ago
  (`.cargo/config.toml`, native Cargo 1.100 min-publish-age). Dependabot uses
  the same 14-day cooldown.
- Supabase free projects pause after about a week of inactivity. Sync must
  surface that as a recoverable "project paused/offline" state.
- Free plans have no point-in-time recovery. The app will expose JSON export;
  a scheduled encrypted dump is a follow-up.
- Public repo: only the `anon` key may ever be embedded/client-side. The
  `service_role` key is CI/dashboard-only.
- Releases are unsigned portable archives for now. Windows SmartScreen and
  Linux AppImage/MSI packaging are later decisions.
