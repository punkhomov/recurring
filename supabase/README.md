# Supabase

The cloud side is a personal, single-user Supabase project on the free plan.
The app is offline-first; Supabase is the meeting point between the PC and the
laptop, not the source of truth while offline.

## One-time setup

```bash
supabase login
supabase init          # creates supabase/config.toml on a fresh checkout
supabase link --project-ref <project-ref>
supabase db push
```

Then, in the Supabase dashboard:

1. Create the only account under **Authentication → Users**.
2. Disable public sign-ups after that account exists.
3. Keep email OTP/password auth enabled for that account.
4. Do not put the `service_role` key in the desktop app. The `anon` key is
   public by design and RLS is what protects the data.

## Free-plan caveat

Free projects pause after roughly a week without activity. The desktop sync
loop tolerates a paused project (it reports "offline" and keeps working), and a
small weekly health check can keep the project warm. Do not store anything in
this project that you cannot afford to lose: free plans do not include
point-in-time recovery. The app will get a JSON export, and a scheduled dump is
planned.

## Schema

`migrations/20261115120000_init.sql` creates:

- `events` — recurring definitions, with the schedule as JSON.
- `completions` — append-only history, unique by `(event_id, due_on)`.
- RLS policies scoped to `auth.uid()`.
- An `updated_at` trigger.

`hlc` is a hybrid logical clock string supplied by the client. Sync merges by
comparing HLC values, so device wall clocks do not decide the winner.
