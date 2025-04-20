use anyhow::Result;
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

async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Clear existing data for clean testing
    sqlx::query("DROP TABLE IF EXISTS json_test")
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
            preserved_data JSONB NOT NULL,
            exact_text TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_json_with_exact_text(pool: &PgPool, json_data: &str) -> Result<i32> {
    // Parse for JSONB columns
    let value: Value = serde_json::from_str(json_data)?;

    let row = sqlx::query(
        r#"
        INSERT INTO json_test (data, preserved_data, exact_text)
        VALUES ($1, $1, $2)
        RETURNING id
        "#,
    )
    .bind(&value)  // For JSONB columns
    .bind(json_data)  // Store exact text in TEXT column
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

async fn get_json_by_id(pool: &PgPool, id: i32) -> Result<(Value, Value, String)> {
    let row = sqlx::query(
        r#"
        SELECT data, preserved_data, exact_text FROM json_test WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    let data: Value = row.try_get("data")?;
    let preserved: Value = row.try_get("preserved_data")?;
    let exact_text: String = row.try_get("exact_text")?;

    Ok((data, preserved, exact_text))
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

    // Insert JSON with exact text preservation
    let id = insert_json_with_exact_text(&pool, json_data).await?;
    println!("Inserted JSON with ID: {}", id);

    // Retrieve all versions
    let (jsonb_data, preserved_jsonb, exact_text) = get_json_by_id(&pool, id).await?;

    // Display results
    println!("\n--- Original JSON ---\n{}", json_data);
    println!("\n--- Retrieved JSONB (order not preserved) ---\n{}", serde_json::to_string_pretty(&jsonb_data)?);
    println!("\n--- Retrieved Exact Text (original format preserved) ---\n{}", exact_text);

    // Example of how to use the exact text in your application
    println!("\n--- Parsing the exact text for use ---");
    let exact_parsed: Value = serde_json::from_str(&exact_text)?;

    // Accessing fields in their original order (when using the exact_text)
    if let Some(movies) = exact_parsed.get("movies").and_then(|m| m.as_array()) {
        if let Some(movie) = movies.get(0) {
            if let Some(title) = movie.get("title") {
                println!("First movie title: {}", title);
            }
            if let Some(genre) = movie.get("genre") {
                println!("First movie genre: {}", genre);
            }
        }
    }

    Ok(())
}