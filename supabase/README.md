# Supabase setup

One personal Supabase project on the free plan. The desktop app is
offline-first: Supabase is the meeting point between the PC and the laptop, not
the source of truth while offline.

## What is safe to expose?

Supabase has two very different kinds of keys:

| Value | Secret? | Where it may live |
| --- | --- | --- |
| Project URL (`https://<ref>.supabase.co`) | No | README, app config, public repo |
| Publishable key (`sb_publishable_...`) | No, designed for clients | App, public repo, CI variables |
| Legacy `anon` JWT | No, legacy publishable equivalent | App, public repo |
| Secret key (`sb_secret_...`) | **Yes** | Server/CI only, never in the app |
| Legacy `service_role` JWT | **Yes**, bypasses RLS | Server/CI only, never in the app |
| Database password | **Yes** | Password manager / CLI only |
| JWT signing secret | **Yes** | Supabase project settings only |
| Supabase access token | **Yes** | Your CLI machine only |
| User access/refresh tokens | **Yes** | OS keychain on the device |

The publishable key is not an authorization mechanism by itself. It maps to the
`anon` Postgres role before login and to `authenticated` after login. RLS
policies are what protect the rows. That is why the publishable key is safe in a
public desktop app, and why a table with RLS disabled is a data leak.

Do **not** put the secret key, `service_role`, DB password, or an access token
into `.env`, the repository, CI logs, or this chat. If one leaks, rotate it in
the Supabase dashboard immediately.

## What I need from you

For the app to connect, only these two values are needed:

```dotenv
RECURRING_SUPABASE_URL=https://<project-ref>.supabase.co
RECURRING_SUPABASE_PUBLISHABLE_KEY=sb_publishable_...
```

You can put them in `.env` in the repository root. It is already ignored by
`.gitignore`. If you paste them in chat, that is acceptable for the
publishable key, but `.env` is cleaner.

For the database migration, you do **not** need to send me credentials. Either:

1. Open **SQL Editor** in the Supabase dashboard and run the contents of
   `supabase/migrations/20261115120000_init.sql`, or
2. On your machine run:

   ```bash
   supabase login
   supabase link --project-ref <project-ref>
   supabase db push
   ```

If you want me to run `supabase db push` from this sandbox, I would need a
Supabase access token or database password. I recommend against sharing those;
the migration is intentionally a single SQL file that can be pasted into the
dashboard.

## Create the project

1. Create a project at <https://supabase.com/dashboard>, free plan.
2. Pick the closest region (for a Russian user, usually Frankfurt).
3. Wait until the database is healthy.
4. Open **Project Settings → API** and copy the project URL and publishable key.
   The legacy `anon` key works too, but prefer the publishable key.
5. Keep the secret key visible only in the dashboard; nobody needs it for this
   app.

## Apply the schema

The migration creates:

- `events` — recurring definitions; `schedule` is JSON produced by
  `recurring_core::Schedule`;
- `completions` — append-only history, unique by `(event_id, due_on)`;
- RLS policies scoped to `auth.uid()`;
- indexes for the HLC sync cursor;
- an `updated_at` trigger.

After applying it, verify that the public API is closed to anonymous requests:

```bash
curl -s "$RECURRING_SUPABASE_URL/rest/v1/events?select=id" \
  -H "apikey: $RECURRING_SUPABASE_PUBLISHABLE_KEY"
# expected: []
```

If this returns rows, stop and check RLS before going further.

## Email OTP

Email OTP is the chosen login method: a six-digit code, no password and no
magic-link callback server.

1. **Authentication → Providers → Email**: enable it.
2. **Authentication → Users**: create the single account manually with your
   email and confirm it.
3. Disable **Allow new users to sign up**. Otherwise anyone with the
   publishable key can create accounts and consume your auth/email quota.
4. Set the OTP length to 6 and a short expiry (10–15 minutes).
5. **Authentication → Email Templates → Magic Link**: make sure the template
   contains `{{ .Token }}`. This is the code the app asks the user to type.
6. The built-in email sender is rate-limited and meant for low volume. For one
   user it is usually enough; if login emails get delayed, configure custom
   SMTP.

The app flow will be:

```text
POST {url}/auth/v1/otp
  apikey: <publishable key>
  { "email": "you@example.com", "create_user": false }

POST {url}/auth/v1/verify
  apikey: <publishable key>
  { "type": "email", "email": "you@example.com", "token": "123456" }
  -> access_token + refresh_token + user

POST {url}/auth/v1/token?grant_type=refresh_token
  { "refresh_token": "..." }
```

The refresh token goes to the OS keychain (Secret Service on Linux, Credential
Manager on Windows). The access token stays in memory. API requests then send
both `apikey: <publishable key>` and `Authorization: Bearer <access_token>`.

There is no signup flow in the app. If the user was deleted or the email is
wrong, the OTP request fails; that is intentional for a one-user system.

## Free-plan caveats

- Supabase pauses free projects after roughly seven days without database
  activity. The app must treat a paused project as a recoverable offline state,
  not as data loss.
- Free plans have no point-in-time recovery. The app will get JSON export; an
  encrypted periodic dump is a later task.
- The publishable key can be used to trigger OTP emails or consume rate limits.
  Keep signups disabled and use Supabase's rate limits. It still cannot read
  your rows without a valid user JWT and matching RLS policy.
- Only one user exists. Do not add a shared account or disable RLS for
  convenience.

## Sync model

The client will push/pull rows with a hybrid logical clock (`hlc`) and
tombstones (`deleted_at`), using `Cargo`-side SQLite as the local source of
truth. See [`../docs/architecture.md`](../docs/architecture.md). The SQL here is
ready for that model; no app code is connected yet.
