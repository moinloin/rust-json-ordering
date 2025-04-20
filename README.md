# PostgreSQL JSON Order Preservation PoC

A demonstration of JSON field order preservation issues in PostgreSQL and a solution using `LinkedHashMap` in Rust.

## Origin of this Project

This project was created to address a JSON field order preservation issue encountered in the [Penumbra Explorer Backend](https://github.com/pk-labs/penumbra-explorer-backend). The problem occurs because PostgreSQL's JSONB format optimizes for querying efficiency rather than preserving exact textual representation, which can cause problems when field order is important.

## 🔍 Problem Overview

When storing JSON in PostgreSQL using the JSONB type, the key order in JSON objects is not preserved. This is because JSONB is optimized for querying and analysis rather than preserving the exact textual representation of the JSON.

### Example:

**Original JSON:**
```json
{
  "title": "Inception",
  "genre": "Sci-Fi",
  "locations": ["Cinema City Berlin", "Movieplex Hamburg"]
}
```

**Retrieved from PostgreSQL:**
```json
{
  "genre": "Sci-Fi",
  "locations": ["Cinema City Berlin", "Movieplex Hamburg"],
  "title": "Inception"
}
```

## ✅ Solution

This project demonstrates two approaches:
1. **Standard approach** (using regular JSON serialization) - order not preserved
2. **Order-preserving approach** (using `LinkedHashMap`) - order preserved exactly as specified

## 🛠️ Tech Stack

- **Rust** with `serde`, `sqlx`, and `linked-hash-map`
- **PostgreSQL** for database storage
- **Docker** for containerization and easy setup
- **Docker Compose** for orchestration

## 📁 Project Structure

```
├── Cargo.toml             # Rust dependencies
├── Dockerfile             # Rust app container setup
├── docker-compose.yml     # Service configuration
├── init.sql               # Database initialization
├── json.txt               # Custom JSON input file
├── run.sh                 # Colorful execution script
└── src/
    └── main.rs            # Rust implementation
```

## 🚀 Getting Started

### Prerequisites

- Docker and Docker Compose
- Basic knowledge of Rust and PostgreSQL

### Quick Start

1. Clone this repository
   ```bash
   git clone https://github.com/yourusername/postgresql-json-order-demo
   cd postgresql-json-order-demo
   ```

2. Create a `json.txt` file with your desired JSON data (optional - a sample will be created if none exists)

3. Make the run script executable and run it
   ```bash
   chmod +x run.sh
   ./run.sh
   ```

## ⚙️ How It Works

The application:

1. Creates a PostgreSQL table with both JSONB and raw text columns
2. Reads JSON from the `json.txt` file
3. Stores the JSON in both formats in the database
4. Retrieves and displays both versions, showing the order difference

### Demo Output

```
==========================================
   JSON Order Preservation Test
==========================================

➤ Original JSON:
{
  "movies": [
    {
      "title": "Inception",
      "director": "Christopher Nolan",
      "year": 2010,
      "genre": "Sci-Fi",
      "locations": ["Cinema City Berlin", "Movieplex Hamburg"]
    }
  ]
}

➤ Retrieved JSONB (order not preserved):
{
  "movies": [
    {
      "genre": "Sci-Fi",
      "locations": ["Cinema City Berlin", "Movieplex Hamburg"],
      "year": 2010,
      "title": "Inception",
      "director": "Christopher Nolan"
    }
  ]
}

➤ Retrieved Raw Text (exactly as inserted):
{
  "movies": [
    {
      "title": "Inception",
      "director": "Christopher Nolan",
      "year": 2010,
      "genre": "Sci-Fi",
      "locations": ["Cinema City Berlin", "Movieplex Hamburg"]
    }
  ]
}
```

## 🔧 Custom JSON Input

This project supports reading JSON from a file rather than using hardcoded values:

1. Create a file named `json.txt` in the project root directory
2. Add your custom JSON data to the file
3. Run the application with `./run.sh`

## 💡 Key Implementation Details

The solution uses:
- `serde` and `serde_json` for JSON serialization/deserialization
- `linked-hash-map::LinkedHashMap` to preserve field order
- Custom structs that use `LinkedHashMap` for field storage
- Explicit field addition in the desired order

## 🧪 Adapting to Your Use Case

To use this approach in your own projects:

1. Add necessary dependencies:
   ```toml
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   linked-hash-map = "0.5"
   ```

2. Create structs similar to `MovieOrdered` using `LinkedHashMap`:
   ```rust
   struct MovieOrdered {
       data: LinkedHashMap<String, Value>,
   }

   impl MovieOrdered {
       fn new() -> Self {
           Self {
               data: LinkedHashMap::new(),
           }
       }

       fn add(&mut self, key: &str, value: Value) -> &mut Self {
           self.data.insert(key.to_string(), value);
           self
       }
   }
   ```

3. When creating objects, add fields in the exact order you want to preserve:
   ```rust
   let mut movie = MovieOrdered::new();
   movie.add("title", json!("Inception"))
        .add("director", json!("Christopher Nolan"))
        .add("year", json!(2010))
        .add("genre", json!("Sci-Fi"));
   ```

4. Serialize and store in the database
