#!/bin/bash

# Build and start the containers
echo "Starting containers..."
docker compose down -v --remove-orphans
docker compose up -d

# Wait for the database to start
echo "Waiting for PostgreSQL to start..."
sleep 15

# Run the Rust app
echo "Running Rust application..."
docker compose run --rm rust-app

# Query the database directly to see how data is stored
echo "Querying the database directly:"
docker compose exec postgres psql -U testuser -d testdb -c "SELECT id, data_jsonb, raw_text FROM json_test;"

# Stop the containers
echo "Stopping containers..."
docker compose down --remove-orphans