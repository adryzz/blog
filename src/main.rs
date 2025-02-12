mod atom;
mod blog;
mod metadata;
mod rss;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;

use askama::Template;
use axum::{http::StatusCode, routing::get, Router};
use chrono::{DateTime, Utc};
use tower_http::services::{ServeDir, ServeFile};

const ROOT_URL: &str = "http://lena.nihil.gay";
const ATOM_URL: &str = "http://lena.nihil.gay/blog/atom.xml";

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
    let cache = Cache::new();


    let mut c = cache.clone();
    tokio::spawn(async move {
        c.update_5min().await;
    });

    let app = Router::new()
        .route("/", get(index))
        .route_service("/style.css", ServeFile::new("style.css"))
        .route("/blog/rss.xml", get(rss::rss))
        .route("/blog/atom.xml", get(atom::atom))
        .route("/blog/{page}", get(blog::page))
        .route("/blog", get(blog::blog))
        .nest(
            "/blog",
            Router::new().fallback_service(ServeDir::new("content")),
        )
        .nest_service("/badges", ServeDir::new("badges"))
        .with_state(cache);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
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

#[derive(Debug, Clone)]
pub struct Cache(Arc<RwLock<BTreeMap<std::cmp::Reverse<DateTime<Utc>>, blog::BlogPage>>>);

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(BTreeMap::new())))
    }

    pub async fn update(&mut self) {
        blog::get_pages(&mut self.0.write().await).await;
    }

    pub async fn update_5min(&mut self) {
        loop {
            self.update().await;
            tokio::time::sleep(Duration::from_secs(5 * 60)).await;
        }
    }
}
