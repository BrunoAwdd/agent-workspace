-- Human reviews: one per (reviewer_id, agent_id) — upsertable
CREATE TABLE IF NOT EXISTS agent_reviews (
    id           TEXT PRIMARY KEY,
    agent_id     TEXT NOT NULL REFERENCES agents(id),
    reviewer_id  TEXT NOT NULL,
    score        INTEGER NOT NULL CHECK(score BETWEEN 1 AND 5),
    review_text  TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS agent_reviews_unique ON agent_reviews(agent_id, reviewer_id);

-- Agent endorsements: unlimited, positive OR negative sentiment
CREATE TABLE IF NOT EXISTS agent_endorsements (
    id             TEXT PRIMARY KEY,
    to_agent_id    TEXT NOT NULL REFERENCES agents(id),
    from_agent_id  TEXT NOT NULL,
    sentiment      TEXT NOT NULL DEFAULT 'positive',
    reason         TEXT,
    created_at     TEXT NOT NULL
);
