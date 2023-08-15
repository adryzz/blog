mod blog;
mod metadata;
mod rss;
use askama::Template;
use axum::{http::StatusCode, routing::get, Router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting server...");

    match run().await {
        Ok(_) => tracing::info!("Program exited successfully."),
        Err(e) => tracing::error!("Error: {}", e),
    }
}

async fn run() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(index))
        .route("/blog/rss.xml", get(rss::rss))
        .route("/blog/:page", get(blog::page))
        .route("/blog", get(blog::blog))
        .nest("/blog", axum_static::static_router("blog"))
        .nest("/badges", axum_static::static_router("badges"));

    let listener = std::net::TcpListener::bind("0.0.0.0:3000")?;
    tracing::info!("Listening on {}...", listener.local_addr()?);

    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index() -> Result<IndexTemplate, StatusCode> {
    Ok(IndexTemplate {})
}
