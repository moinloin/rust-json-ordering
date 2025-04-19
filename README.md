# JSON Order Preservation Test Environment

This project demonstrates the issue with JSON key order not being preserved when storing JSON in PostgreSQL, and provides a solution using `LinkedHashMap` to maintain the original field order.

## Problem Overview

When storing JSON in PostgreSQL using the JSONB type, the key order in JSON objects is not preserved. This is because JSONB is optimized for querying and analysis rather than preserving the exact textual representation of the JSON.

For example, if you have:

```json
{
  "title": "Inception",
  "genre": "Sci-Fi",
  "locations": ["Cinema City Berlin", "Movieplex Hamburg"]
}
```

It might be stored and retrieved as:

```json
{
  "genre": "Sci-Fi",
  "locations": ["Cinema City Berlin", "Movieplex Hamburg"],
  "title": "Inception"
}
```

## Solution

This project demonstrates two approaches:
1. Standard approach (using regular JSON serialization) - order not preserved
2. Order-preserving approach (using `LinkedHashMap`) - order preserved exactly as specified

## Project Structure

- `docker-compose.yml` - Sets up PostgreSQL and Rust containers
- `Dockerfile` - Builds the Rust application
- `init-db.sql` - Creates the database table
- `src/main.rs` - The main Rust code demonstrating both approaches
- `run-test.sh` - Script to run the test

## How to Run

Make the run script executable and run it:

```bash
chmod +x run-test.sh
./run-test.sh
```

The script will:
1. Start the PostgreSQL and Rust containers
2. Run the Rust application, which inserts JSON data using both approaches
3. Display the original JSON, the standard stored JSON, and the order-preserved JSON
4. Query the database directly to show how the data is stored
5. Stop the containers

## Key Implementation Details

The solution uses:
- `serde` and `serde_json` for JSON serialization/deserialization
- `linked-hash-map::LinkedHashMap` to preserve field order
- Custom structs that use `LinkedHashMap` for field storage
- Explicit field addition in the desired order

The key part of the solution is in the `insert_ordered_json` function, which:
1. Parses the JSON normally
2. Creates an ordered representation where fields are added in the specific order we want to preserve
3. Stores both the regular JSON and the order-preserved JSON in separate columns

## Adapting to Your Use Case

To adapt this solution to your own code:

1. Create structs similar to `MovieOrdered` using `LinkedHashMap`
2. When creating objects, add fields in the exact order you want to preserve
3. Serialize to JSON and store in the database

This approach ensures that when you retrieve the JSON later, the field order will be exactly as you specified.