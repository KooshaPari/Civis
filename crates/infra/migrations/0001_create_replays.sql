create table if not exists replays (
    id bigserial primary key,
    name text not null,
    created_at timestamptz not null default now(),
    blob bytea not null
);
