CREATE TABLE IF NOT EXISTS agents (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    role         TEXT NOT NULL,
    capabilities JSONB NOT NULL DEFAULT '[]',
    permissions  JSONB NOT NULL DEFAULT '[]',
    status       TEXT NOT NULL DEFAULT 'offline',
    metadata     JSONB NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS agent_sessions (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id         TEXT NOT NULL REFERENCES agents(id),
    status           TEXT NOT NULL DEFAULT 'active',
    started_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at         TIMESTAMPTZ,
    health           TEXT NOT NULL DEFAULT 'healthy',
    current_task_id  UUID,
    metadata         JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS messages (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id  TEXT NOT NULL,
    channel_id    TEXT,
    thread_id     UUID,
    from_agent_id TEXT NOT NULL,
    to_agent_id   TEXT,
    kind          TEXT NOT NULL,
    payload       JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS messages_channel ON messages(channel_id, created_at);

CREATE TABLE IF NOT EXISTS inbox_items (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_agent_id   TEXT NOT NULL,
    source_agent_id   TEXT,
    kind              TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'pending',
    payload           JSONB NOT NULL DEFAULT '{}',
    deliver_on_checkin BOOLEAN NOT NULL DEFAULT true,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at      TIMESTAMPTZ,
    expires_at        TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS inbox_agent_pending ON inbox_items(target_agent_id, status);

CREATE TABLE IF NOT EXISTS tasks (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title            TEXT NOT NULL,
    description      TEXT NOT NULL DEFAULT '',
    kind             TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'open',
    priority         TEXT NOT NULL DEFAULT 'normal',
    assigned_agent_id TEXT,
    source_ref       TEXT,
    metadata         JSONB NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS locks (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope_type       TEXT NOT NULL,
    scope_id         TEXT NOT NULL,
    lock_type        TEXT NOT NULL,
    owner_agent_id   TEXT NOT NULL,
    owner_session_id UUID NOT NULL,
    acquired_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at       TIMESTAMPTZ NOT NULL,
    metadata         JSONB NOT NULL DEFAULT '{}',
    UNIQUE (scope_type, scope_id)
);

CREATE TABLE IF NOT EXISTS events (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id TEXT,
    agent_id     TEXT,
    session_id   UUID,
    kind         TEXT NOT NULL,
    payload      JSONB NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS events_agent ON events(agent_id, created_at DESC);

CREATE TABLE IF NOT EXISTS handoffs (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_agent_id    TEXT NOT NULL,
    to_agent_id      TEXT,
    source_session_id UUID NOT NULL,
    task_id          UUID,
    summary          TEXT NOT NULL DEFAULT '',
    payload          JSONB NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dependencies (
    key         TEXT PRIMARY KEY,
    state       TEXT NOT NULL DEFAULT 'unknown',
    details     TEXT,
    checked_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
