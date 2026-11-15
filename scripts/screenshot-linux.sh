#!/usr/bin/env bash
set -euo pipefail

# Renders the real GPUI window on a virtual X11 display and captures it.
# Mesa's lavapipe (software Vulkan) keeps this working on CI runners without a GPU.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${RECURRING_BIN:-$ROOT/target/debug/recurring}"
OUT="${1:-$ROOT/artifacts/screenshots/today.png}"
TODAY="${RECURRING_TODAY:-2026-11-15}"

for tool in Xvfb openbox xdotool import identify; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "error: missing $tool; run scripts/install-linux-deps.sh --screenshot" >&2
    exit 1
  fi
done

if [ ! -x "$BIN" ]; then
  echo "error: $BIN not found; run: cargo build -p recurring-app" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUT")"
LOG_DIR="$ROOT/artifacts/screenshots"
mkdir -p "$LOG_DIR"

DISPLAY_NUM=99
while [ -e "/tmp/.X${DISPLAY_NUM}-lock" ]; do
  DISPLAY_NUM=$((DISPLAY_NUM + 1))
done
export DISPLAY=":${DISPLAY_NUM}"

Xvfb "$DISPLAY" -screen 0 1440x900x24 -nolisten tcp >"$LOG_DIR/xvfb.log" 2>&1 &
XVFB_PID=$!
APP_PID=""
OPENBOX_PID=""

cleanup() {
  if [ -n "$APP_PID" ] && kill -0 "$APP_PID" 2>/dev/null; then
    kill "$APP_PID" 2>/dev/null || true
    wait "$APP_PID" 2>/dev/null || true
  fi
  if [ -n "$OPENBOX_PID" ] && kill -0 "$OPENBOX_PID" 2>/dev/null; then
    kill "$OPENBOX_PID" 2>/dev/null || true
  fi
  kill "$XVFB_PID" 2>/dev/null || true
  wait "$XVFB_PID" 2>/dev/null || true
}
trap cleanup EXIT

sleep 1
openbox >"$LOG_DIR/openbox.log" 2>&1 &
OPENBOX_PID=$!
sleep 1

# An inherited Wayland socket would take precedence over DISPLAY inside GPUI.
unset WAYLAND_DISPLAY

if [ -f /usr/share/vulkan/icd.d/lvp_icd.json ]; then
  export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json
fi
export RECURRING_TODAY="$TODAY"
export RUST_LOG="${RUST_LOG:-info}"

"$BIN" --screenshot >"$LOG_DIR/app.log" 2>&1 &
APP_PID=$!

WINDOW_ID=""
for _ in $(seq 1 40); do
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "error: app exited before opening a window" >&2
    tail -80 "$LOG_DIR/app.log" >&2 || true
    exit 1
  fi
  WINDOW_ID="$(xdotool search --name '^Recurring$' 2>/dev/null | head -1 || true)"
  if [ -n "$WINDOW_ID" ]; then
    break
  fi
  sleep 0.5
done

if [ -z "$WINDOW_ID" ]; then
  echo "error: Recurring window did not appear" >&2
  tail -80 "$LOG_DIR/app.log" >&2 || true
  exit 1
fi

# Let the first frames and font atlas settle.
sleep 2

import -display "$DISPLAY" -window "$WINDOW_ID" "$OUT"
identify "$OUT"
echo "screenshot: $OUT"
