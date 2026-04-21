-- Same as SQLite version — Postgres uses same DDL for these tables
CREATE TABLE IF NOT EXISTS human_reviews (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL REFERENCES agents(id),
    reviewer_id     TEXT NOT NULL,
    task_id         TEXT,
    stars           INTEGER NOT NULL CHECK(stars BETWEEN 1 AND 5),
    praise          TEXT,
    criticism       TEXT,
    domain_context  TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS human_reviews_unique ON human_reviews(agent_id, reviewer_id);

CREATE TABLE IF NOT EXISTS agent_peer_reviews (
    id              TEXT PRIMARY KEY,
    to_agent_id     TEXT NOT NULL REFERENCES agents(id),
    from_agent_id   TEXT NOT NULL,
    task_id         TEXT,
    stars           INTEGER NOT NULL CHECK(stars BETWEEN 1 AND 5),
    praise          TEXT,
    criticism       TEXT,
    domain_context  TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS agent_peer_reviews_unique ON agent_peer_reviews(to_agent_id, from_agent_id);

CREATE TABLE IF NOT EXISTS agent_capabilities (
    id          TEXT PRIMARY KEY,
    agent_id    TEXT NOT NULL REFERENCES agents(id),
    domain      TEXT NOT NULL,
    level       INTEGER NOT NULL CHECK(level BETWEEN 0 AND 5),
    source      TEXT NOT NULL DEFAULT 'manual',
    confidence  REAL NOT NULL DEFAULT 1.0,
    updated_at  TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS agent_capabilities_unique ON agent_capabilities(agent_id, domain);
