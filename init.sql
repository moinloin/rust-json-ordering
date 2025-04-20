CREATE TABLE IF NOT EXISTS json_test (
    id SERIAL PRIMARY KEY,
    data_jsonb JSONB NOT NULL,
    raw_text TEXT NOT NULL
);