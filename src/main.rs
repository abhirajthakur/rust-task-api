mod app;
mod config;
mod database;
mod handlers;
mod models;
mod seed;
mod state;

use config::AppConfig;
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let config = AppConfig::from_env()?;
    let pool = database::connect(&config.database.url).await?;

    sqlx::migrate!().run(&pool).await?;

    match std::env::args().nth(1).as_deref() {
        Some("seed") => {
            seed::seed_database(&pool).await?;
            println!("Seed command completed.");
            return Ok(());
        }
        Some(command) => {
            return Err(format!("Unknown command: {command}. Usage: cargo run [-- seed]").into());
        }
        None => {}
    }

    let state = AppState::new(pool);
    let app = app::create_router(state);
    let port = config.server.port;

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    println!("Server listening on http://localhost:{port}");

    axum::serve(listener, app).await?;

    Ok(())
}
