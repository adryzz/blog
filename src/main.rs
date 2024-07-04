mod blog;
mod metadata;
mod rss;
use askama::Template;
use axum::{http::StatusCode, routing::get, Router};

const ROOT_URL: &str = "http://lena.nihil.gay";

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
        //.route("/blog/:page/", get(blog::page))
        .route("/blog", get(blog::blog))
        .nest("/blog", axum_static::static_router("content"))
        .nest("/badges", axum_static::static_router("badges"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Listening on {}...", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    page_name: &'static str,
}

async fn index() -> Result<IndexTemplate, StatusCode> {
    Ok(IndexTemplate { page_name: "home" })
}
