use anyhow::Result;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};

#[derive(Debug, Serialize, Deserialize)]
struct Movie {
    title: String,
    genre: String,
    locations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MovieOrdered {
    #[serde(flatten)]
    fields: LinkedHashMap<String, Value>,
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

#[derive(Debug, Serialize, Deserialize)]
struct MoviesOrdered {
    movies: Vec<MovieOrdered>,
}

async fn create_pool(database_url: &str) -> Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?)
}

async fn ensure_table_exists(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS json_test (
            id SERIAL PRIMARY KEY,
            data JSONB,
            preserved_data JSONB
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
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

async fn insert_ordered_json(pool: &PgPool, json_str: &str) -> Result<i32> {
    let regular_value: Value = serde_json::from_str(json_str)?;
    let regular_movies: Movies = serde_json::from_value(regular_value.clone())?;

    let mut ordered_movies = MoviesOrdered { movies: vec![] };

    for movie in regular_movies.movies {
        let mut ordered = MovieOrdered::new();
        ordered.add_field("title", json!(movie.title));
        ordered.add_field("genre", json!(movie.genre));
        ordered.add_field("locations", json!(movie.locations));
        ordered_movies.movies.push(ordered);
    }

    let preserved_json = serde_json::to_value(ordered_movies)?;

    let row = sqlx::query(
        r#"
        INSERT INTO json_test (data, preserved_data)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(&regular_value)
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
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://testuser:testpassword@localhost:5432/testdb".to_string());

    let pool = create_pool(&database_url).await?;
    ensure_table_exists(&pool).await?;

    let json_data = r#"
    {
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
    }
    "#;

    let regular_id = insert_regular_json(&pool, json_data).await?;
    println!("Inserted regular JSON with ID: {}", regular_id);

    let ordered_id = insert_ordered_json(&pool, json_data).await?;
    println!("Inserted ordered JSON with ID: {}", ordered_id);

    let (regular_json, _) = get_json_by_id(&pool, regular_id).await?;
    let (_, preserved_json) = get_json_by_id(&pool, ordered_id).await?;

    println!("\n--- Original JSON ---\n{}", json_data);
    println!("\n--- Inserted Regular JSON (no order) ---\n{}", serde_json::to_string_pretty(&regular_json)?);
    println!("\n--- Inserted Ordered JSON (preserved) ---\n{}", serde_json::to_string_pretty(&preserved_json)?);

    Ok(())
}
