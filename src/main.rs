mod blog;

use std::path::PathBuf;

use anyhow::anyhow;
use askama::Template;
use axum::{http::StatusCode, routing::get, Router};
use chrono::prelude::*;

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
        .route("/blog/:page", get(blog::blog))
        .nest("/blog", axum_static::static_router("blog"));

    let listener = std::net::TcpListener::bind("0.0.0.0:3000")?;
    tracing::info!("Listening on {}...", listener.local_addr()?);

    axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    pages: Vec<BlogPage>,
}

async fn index() -> Result<IndexTemplate, StatusCode> {
    let pages = match get_pages().await {
        Ok(p) => p,
        Err(e) => {
            vec![]
        }
    };

    Ok(IndexTemplate { pages })
}

struct BlogPage {
    title: String,
    description: Option<String>,
    authors: Vec<String>,
    url: String,
    tags: Vec<String>,
    timestamp: NaiveDateTime,
    edit_timestamp: Option<NaiveDateTime>,
    time_to_read: u32,
}

async fn get_pages() -> anyhow::Result<Vec<BlogPage>> {
    let mut entries = tokio::fs::read_dir("blog").await?;

    let mut pages = vec![];

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    let content = tokio::fs::read_to_string(entry.path()).await?;
                    let page = parse_page(&entry.path(), &content).await?;
                    pages.push(page)
                }
            }
        }
    }

    Ok(pages)
}

async fn parse_page(path: &PathBuf, content: &str) -> anyhow::Result<BlogPage> {
    let url = format!(
        "/blog/{}",
        path.file_stem()
            .ok_or_else(|| anyhow!("Error while generating page URL"))?
            .to_str()
            .ok_or_else(|| anyhow!("Error while generating page URL"))?
    );

    Ok(BlogPage {
        title: "Very cool blog page".to_string(),
        description: Some("very cool indeed".to_string()),
        authors: vec!["Lena".to_string()],
        url,
        tags: vec!["meta".to_string()],
        timestamp: NaiveDateTime::from_timestamp_opt(1692003237, 0)
            .ok_or_else(|| anyhow!("Error while generating timestamp"))?,
        edit_timestamp: Some(NaiveDateTime::from_timestamp_opt(1692063237, 0)
        .ok_or_else(|| anyhow!("Error while generating timestamp"))?),
        time_to_read: 2,
    })
}
