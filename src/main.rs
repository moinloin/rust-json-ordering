use anyhow::Result;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};

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
            data_jsonb JSONB NOT NULL,
            raw_text TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_json(pool: &PgPool, json_data: &str) -> Result<i32> {
    // Parse the original JSON for the JSONB column
    let parsed_value: Value = serde_json::from_str(json_data)?;

    let row = sqlx::query(
        r#"
        INSERT INTO json_test (data_jsonb, raw_text)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(&parsed_value)  // For JSONB column
    .bind(json_data)      // Store raw text exactly as-is
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

async fn get_json_by_id(pool: &PgPool, id: i32) -> Result<(Value, String)> {
    let row = sqlx::query(
        r#"
        SELECT data_jsonb, raw_text FROM json_test WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    let data: Value = row.try_get("data_jsonb")?;
    let raw_text: String = row.try_get("raw_text")?;

    Ok((data, raw_text))
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

    // Insert JSON
    let id = insert_json(&pool, json_data).await?;
    println!("Inserted JSON with ID: {}", id);

    // Retrieve all versions
    let (jsonb_data, raw_text) = get_json_by_id(&pool, id).await?;

    // Display results
    println!("\n--- Original JSON ---\n{}", json_data);
    println!("\n--- Retrieved JSONB (order not preserved) ---\n{}", serde_json::to_string_pretty(&jsonb_data)?);
    println!("\n--- Retrieved Raw Text (exactly as inserted) ---\n{}", raw_text);

    Ok(())
}