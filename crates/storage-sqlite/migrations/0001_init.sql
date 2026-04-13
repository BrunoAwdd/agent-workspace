CREATE TABLE IF NOT EXISTS agents (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    role        TEXT NOT NULL,
    capabilities TEXT NOT NULL DEFAULT '[]',
    permissions  TEXT NOT NULL DEFAULT '[]',
    status      TEXT NOT NULL DEFAULT 'offline',
    metadata    TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_sessions (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL REFERENCES agents(id),
    status          TEXT NOT NULL DEFAULT 'active',
    started_at      TEXT NOT NULL,
    last_seen_at    TEXT NOT NULL,
    ended_at        TEXT,
    health          TEXT NOT NULL DEFAULT 'healthy',
    current_task_id TEXT,
    metadata        TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS messages (
    id           TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    channel_id   TEXT,
    thread_id    TEXT,
    from_agent_id TEXT NOT NULL,
    to_agent_id   TEXT,
    kind         TEXT NOT NULL,
    payload      TEXT NOT NULL DEFAULT '{}',
    created_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS inbox_items (
    id                TEXT PRIMARY KEY,
    target_agent_id   TEXT NOT NULL,
    source_agent_id   TEXT,
    kind              TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'pending',
    payload           TEXT NOT NULL DEFAULT '{}',
    deliver_on_checkin INTEGER NOT NULL DEFAULT 1,
    created_at        TEXT NOT NULL,
    processed_at      TEXT,
    expires_at        TEXT
);

CREATE TABLE IF NOT EXISTS tasks (
    id               TEXT PRIMARY KEY,
    title            TEXT NOT NULL,
    description      TEXT NOT NULL DEFAULT '',
    kind             TEXT NOT NULL,
    status           TEXT NOT NULL DEFAULT 'open',
    priority         TEXT NOT NULL DEFAULT 'normal',
    assigned_agent_id TEXT,
    source_ref       TEXT,
    metadata         TEXT NOT NULL DEFAULT '{}',
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS locks (
    id               TEXT PRIMARY KEY,
    scope_type       TEXT NOT NULL,
    scope_id         TEXT NOT NULL,
    lock_type        TEXT NOT NULL,
    owner_agent_id   TEXT NOT NULL,
    owner_session_id TEXT NOT NULL,
    acquired_at      TEXT NOT NULL,
    expires_at       TEXT NOT NULL,
    metadata         TEXT NOT NULL DEFAULT '{}'
);

CREATE UNIQUE INDEX IF NOT EXISTS locks_scope ON locks(scope_type, scope_id);

CREATE TABLE IF NOT EXISTS events (
    id           TEXT PRIMARY KEY,
    workspace_id TEXT,
    agent_id     TEXT,
    session_id   TEXT,
    kind         TEXT NOT NULL,
    payload      TEXT NOT NULL DEFAULT '{}',
    created_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS handoffs (
    id               TEXT PRIMARY KEY,
    from_agent_id    TEXT NOT NULL,
    to_agent_id      TEXT,
    source_session_id TEXT NOT NULL,
    task_id          TEXT,
    summary          TEXT NOT NULL DEFAULT '',
    payload          TEXT NOT NULL DEFAULT '{}',
    created_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS dependencies (
    key         TEXT PRIMARY KEY,
    state       TEXT NOT NULL DEFAULT 'unknown',
    details     TEXT,
    checked_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
