#!/bin/bash

# Check if json.txt exists
if [ ! -f "json.txt" ]; then
    echo "Warning: json.txt not found in current directory. Creating a sample file..."
    cat > json.txt << EOF
{
    "movies": [
        {
            "title": "Inception",
            "director": "Christopher Nolan",
            "year": 2010,
            "genre": "Sci-Fi",
            "locations": ["Cinema City Berlin", "Movieplex Hamburg"]
        },
        {
            "title": "The Grand Budapest Hotel",
            "director": "Wes Anderson",
            "year": 2014,
            "genre": "Comedy",
            "locations": ["Filmtheater München", "Kino Köln"]
        }
    ]
}
EOF
    echo "Created sample json.txt file"
fi

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