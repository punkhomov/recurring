-- Recurring: initial cloud schema.
--
-- The client is offline-first. Rows carry a hybrid logical clock (`hlc`) so
-- two devices can merge changes with last-writer-wins per row, while deletes
-- are tombstones (`deleted_at`) rather than physical deletes.

create extension if not exists pgcrypto;

create table public.events (
  id uuid primary key,
  user_id uuid not null default auth.uid() references auth.users (id) on delete cascade,
  title text not null check (char_length(title) between 1 and 200),
  notes text not null default '',
  category text not null default 'other'
    check (category in ('subscription', 'bill', 'meter', 'task', 'other')),
  amount_minor bigint check (amount_minor is null or amount_minor >= 0),
  currency text check (currency is null or currency ~ '^[A-Z]{3}$'),
  schedule jsonb not null check (jsonb_typeof(schedule) = 'object'),
  timezone text not null default 'UTC',
  remind_at time not null default '09:00',
  lead_days smallint not null default 3 check (lead_days between 0 and 365),
  active boolean not null default true,
  hlc text not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  deleted_at timestamptz,
  constraint events_amount_currency_pair
    check ((amount_minor is null) = (currency is null))
);

comment on table public.events is
  'Recurring event definitions. Schedule is a JSON-encoded recurring_core::Schedule.';

create index events_user_hlc_idx on public.events (user_id, hlc);
create index events_user_active_idx
  on public.events (user_id, active)
  where deleted_at is null;

create table public.completions (
  id uuid primary key,
  user_id uuid not null default auth.uid() references auth.users (id) on delete cascade,
  event_id uuid not null references public.events (id) on delete cascade,
  due_on date not null,
  completed_at timestamptz not null default now(),
  note text not null default '',
  hlc text not null,
  unique (event_id, due_on)
);

comment on table public.completions is
  'What actually happened. (event_id, due_on) is idempotent across devices.';

create index completions_user_hlc_idx on public.completions (user_id, hlc);

create or replace function public.recurring_set_updated_at()
returns trigger
language plpgsql
security invoker
set search_path = ''
as $$
begin
  new.updated_at = now();
  return new;
end;
$$;

create trigger events_set_updated_at
before update on public.events
for each row execute function public.recurring_set_updated_at();

alter table public.events enable row level security;
alter table public.completions enable row level security;

create policy "events are owned by the signed-in user"
on public.events
for all
to authenticated
using ((select auth.uid()) = user_id)
with check ((select auth.uid()) = user_id);

create policy "completions are owned by the signed-in user"
on public.completions
for all
to authenticated
using ((select auth.uid()) = user_id)
with check (
  (select auth.uid()) = user_id
  and exists (
    select 1
    from public.events
    where events.id = completions.event_id
      and events.user_id = (select auth.uid())
  )
);

grant usage on schema public to authenticated;
grant select, insert, update, delete on public.events to authenticated;
grant select, insert, update, delete on public.completions to authenticated;
