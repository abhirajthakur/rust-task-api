use sqlx::PgPool;

pub async fn seed_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Remove existing tasks and reset generated IDs
    sqlx::query("TRUNCATE TABLE tasks RESTART IDENTITY")
        .execute(&mut *tx)
        .await?;

    let tasks = [
        (
            "Learn Rust",
            Some("Learn ownership, borrowing, and lifetimes"),
            false,
        ),
        (
            "Build a REST API",
            Some("Build a task API using Axum and PostgreSQL"),
            false,
        ),
        (
            "Test API endpoints",
            Some("Practice GET, POST, PATCH, and DELETE"),
            true,
        ),
    ];

    for (title, description, completed) in tasks {
        sqlx::query(
            r#"
            INSERT INTO tasks (title, description, completed)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(title)
        .bind(description)
        .bind(completed)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    println!("Database seeded successfully.");

    Ok(())
}
