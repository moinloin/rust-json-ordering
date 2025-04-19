use anyhow::Result;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};

// Regular Movie struct (for demonstration of the default behavior)
#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    title: String,
    genre: String,
    locations: Vec<String>,
}

// Order-preserving Movie struct using a manual implementation
#[derive(Debug)]
struct MovieOrdered {
    fields: LinkedHashMap<String, Value>,
}

// Manual Serialize implementation for MovieOrdered
impl Serialize for MovieOrdered {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (k, v) in &self.fields {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl MovieOrdered {
    fn new() -> Self {
        Self {
            fields: LinkedHashMap::new(),
        }
    }

    fn add_field(&mut self, key: &str, value: Value) {
        self.fields.insert(key.to_string(), value);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Movies {
    movies: Vec<Movie>,
}

#[derive(Debug, Serialize)]
struct MoviesOrdered {
    movies: Vec<MovieOrdered>,
}

async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Clear existing data for clean testing
    sqlx::query("TRUNCATE TABLE json_test RESTART IDENTITY")
        .execute(&pool)
        .await?;

    Ok(pool)
}

async fn ensure_table_exists(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS json_test (
            id SERIAL PRIMARY KEY,
            data JSONB NOT NULL,
            preserved_data JSONB NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_regular_json(pool: &PgPool, json_data: &str) -> Result<i32> {
    let value: Value = serde_json::from_str(json_data)?;

    let row = sqlx::query(
        r#"
        INSERT INTO json_test (data, preserved_data)
        VALUES ($1, $1)
        RETURNING id
        "#,
    )
    .bind(&value)
    .bind(&value)  // Fixed to bind both parameters
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

async fn insert_ordered_json(pool: &PgPool, json_str: &str) -> Result<i32> {
    // Parse the original JSON
    let original_value: Value = serde_json::from_str(json_str)?;

    // First, insert the regular JSON to demonstrate order loss
    let regular_json = original_value.clone();

    // Now create our ordered version
    let json_obj = original_value.as_object().unwrap();
    let movies_array = json_obj.get("movies").unwrap().as_array().unwrap();

    let mut ordered_movies = MoviesOrdered { movies: vec![] };

    for movie_value in movies_array {
        let movie_obj = movie_value.as_object().unwrap();

        let mut ordered_movie = MovieOrdered::new();

        // Add fields in the specific order we want to preserve
        if let Some(title) = movie_obj.get("title") {
            ordered_movie.add_field("title", title.clone());
        }

        if let Some(genre) = movie_obj.get("genre") {
            ordered_movie.add_field("genre", genre.clone());
        }

        if let Some(locations) = movie_obj.get("locations") {
            ordered_movie.add_field("locations", locations.clone());
        }

        ordered_movies.movies.push(ordered_movie);
    }

    // Convert our ordered structure back to a JSON value
    let preserved_json = serde_json::to_value(&ordered_movies)?;

    let row = sqlx::query(
        r#"
        INSERT INTO json_test (data, preserved_data)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(&regular_json)
    .bind(&preserved_json)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

async fn get_json_by_id(pool: &PgPool, id: i32) -> Result<(Value, Value)> {
    let row = sqlx::query(
        r#"
        SELECT data, preserved_data FROM json_test WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    let data: Value = row.try_get("data")?;
    let preserved: Value = row.try_get("preserved_data")?;

    Ok((data, preserved))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Get database connection string from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://testuser:testpassword@postgres:5432/testdb".to_string());

    println!("Connecting to database at: {}", database_url);

    // Create connection pool and ensure table exists
    let pool = create_pool(&database_url).await?;
    ensure_table_exists(&pool).await?;

    // Test JSON with specific field order
    let json_data = r#"{
        "movies": [
            {
                "title": "Inception",
                "genre": "Sci-Fi",
                "locations": ["Cinema City Berlin", "Movieplex Hamburg"]
            },
            {
                "title": "The Grand Budapest Hotel",
                "genre": "Comedy",
                "locations": ["Filmtheater München", "Kino Köln"]
            }
        ]
    }"#;

    // Demonstrate regular JSON insertion (order not preserved)
    let regular_id = insert_regular_json(&pool, json_data).await?;
    println!("Inserted regular JSON with ID: {}", regular_id);

    // Demonstrate ordered JSON insertion (order preserved)
    let ordered_id = insert_ordered_json(&pool, json_data).await?;
    println!("Inserted ordered JSON with ID: {}", ordered_id);

    // Retrieve both versions
    let (regular_json, _) = get_json_by_id(&pool, regular_id).await?;
    let (_, preserved_json) = get_json_by_id(&pool, ordered_id).await?;

    // Display results
    println!("\n--- Original JSON ---\n{}", json_data);
    println!("\n--- Retrieved Regular JSON (order not preserved) ---\n{}", serde_json::to_string_pretty(&regular_json)?);
    println!("\n--- Retrieved Ordered JSON (order preserved) ---\n{}", serde_json::to_string_pretty(&preserved_json)?);

    Ok(())
}