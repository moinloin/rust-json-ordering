use anyhow::{Result, Context};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::fs;
use std::path::Path;

async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

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
    let parsed_value: Value = serde_json::from_str(json_data)?;

    let row = sqlx::query(
        r#"
        INSERT INTO json_test (data_jsonb, raw_text)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(&parsed_value)
    .bind(json_data)
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

fn read_json_file(file_path: &str) -> Result<String> {
    let json_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read JSON file: {}", file_path))?;

    serde_json::from_str::<Value>(&json_content)
        .with_context(|| format!("Invalid JSON in file: {}", file_path))?;

    Ok(json_content)
}

#[tokio::main]
async fn main() -> Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://testuser:testpassword@postgres:5432/testdb".to_string());

    println!("Connecting to database at: {}", database_url);

    let pool = create_pool(&database_url).await?;
    ensure_table_exists(&pool).await?;

    let possible_paths = vec![
        "json.txt",
        "/app/json.txt",
        "../json.txt",
    ];

    let mut json_file_path = None;
    for path in possible_paths {
        if Path::new(path).exists() {
            json_file_path = Some(path);
            break;
        }
    }

    let json_data = match json_file_path {
        Some(path) => {
            println!("Reading JSON from file: {}", path);
            read_json_file(path)?
        },
        None => {
            println!("Warning: json.txt file not found in expected locations.");
            println!("Using fallback hardcoded JSON sample.");
            String::from(r#"{
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
            }"#)
        }
    };

    let id = insert_json(&pool, &json_data).await?;
    println!("Inserted JSON with ID: {}", id);

    let (jsonb_data, raw_text) = get_json_by_id(&pool, id).await?;

    println!("\n--- Original JSON ---\n{}", json_data);
    println!("\n--- Retrieved JSONB (order not preserved) ---\n{}", serde_json::to_string_pretty(&jsonb_data)?);
    println!("\n--- Retrieved Raw Text (exactly as inserted) ---\n{}", raw_text);

    Ok(())
}