CREATE TABLE IF NOT EXISTS eligibility_policies (
    task_kind TEXT PRIMARY KEY,
    rules JSONB NOT NULL
);
