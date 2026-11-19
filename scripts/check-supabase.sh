#!/usr/bin/env bash
set -euo pipefail

# Local-only connectivity check. Uses the publishable key from .env and never
# prints it. No secret/service_role key is required.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ -f "$ROOT/.env" ]; then
  set -a
  # shellcheck disable=SC1091
  . "$ROOT/.env"
  set +a
fi

: "${RECURRING_SUPABASE_URL:?set RECURRING_SUPABASE_URL in .env}"
: "${RECURRING_SUPABASE_PUBLISHABLE_KEY:?set RECURRING_SUPABASE_PUBLISHABLE_KEY in .env}"

AUTH_TMP="$(mktemp)"
EVENTS_TMP="$(mktemp)"
trap 'rm -f "$AUTH_TMP" "$EVENTS_TMP"' EXIT

AUTH_CODE="$(
  curl -sS -o "$AUTH_TMP" -w '%{http_code}' \
    "$RECURRING_SUPABASE_URL/auth/v1/settings" \
    -H "apikey: $RECURRING_SUPABASE_PUBLISHABLE_KEY"
)"
echo "auth settings: HTTP $AUTH_CODE"
if [ "$AUTH_CODE" = "200" ]; then
  grep -o '"disable_signup":[a-z]*' "$AUTH_TMP" || true
  grep -o '"email":[a-z]*' "$AUTH_TMP" || true
fi

EVENTS_CODE="$(
  curl -sS -o "$EVENTS_TMP" -w '%{http_code}' \
    "$RECURRING_SUPABASE_URL/rest/v1/events?select=id&limit=1" \
    -H "apikey: $RECURRING_SUPABASE_PUBLISHABLE_KEY"
)"
echo "events table: HTTP $EVENTS_CODE"
head -c 400 "$EVENTS_TMP"
echo

case "$EVENTS_CODE" in
  200) echo "OK: table exists; anon must receive [] (RLS)." ;;
  404) echo "MIGRATION NOT APPLIED: run supabase/migrations/20261119213845_init.sql." ;;
  401|403) echo "KEY/GRANT PROBLEM: check the publishable key and Data API exposure." ;;
  *) echo "Unexpected response; inspect the body above." ;;
esac

if [ "$AUTH_CODE" = "200" ] && grep -q '"disable_signup":true' "$AUTH_TMP"; then
  echo "OK: self-signup is disabled."
else
  echo "WARNING: check Authentication -> Allow new users to sign up (should be off)."
fi
