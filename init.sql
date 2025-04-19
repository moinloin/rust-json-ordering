CREATE TABLE IF NOT EXISTS json_test (
    id SERIAL PRIMARY KEY,
    data JSONB NOT NULL,
    preserved_data JSONB NOT NULL
);
