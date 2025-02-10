use askama::Template;
use axum::extract::{Query, State};
use axum::{extract::Path, http::StatusCode};
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};
use serde::Deserialize;

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::ops::DerefMut;
use std::path::PathBuf;

use anyhow::anyhow;
use chrono::prelude::*;

use crate::{metadata, Cache};

#[axum::debug_handler]
pub async fn blog(
    State(cache): State<Cache>,
    Query(params): Query<BlogQuery>,
) -> Result<BlogTemplate, StatusCode> {
    let pages = cache.0.read().await;

    if let Some(tag) = params.tagged {
        return Ok(BlogTemplate {
            pages: pages
                .values()
                .filter(|v| v.tags.contains(&tag))
                .cloned()
                .collect(),
            page_name: "blog",
            root_url: crate::ROOT_URL,
        });
    }

    Ok(BlogTemplate {
        pages: pages.values().cloned().collect(),
        page_name: "blog",
        root_url: crate::ROOT_URL,
    })
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogTemplate {
    pages: Vec<BlogPage>,
    page_name: &'static str,
    root_url: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlogQuery {
    tagged: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlogPage {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub url: String,
    pub image: Option<String>,
    pub image_alt: Option<String>,
    pub tags: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub edit_timestamp: Option<DateTime<Utc>>,
    pub time_to_read: u32,
}

impl PartialOrd for BlogPage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.timestamp.cmp(&self.timestamp))
    }
}

impl Ord for BlogPage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.timestamp.cmp(&self.timestamp)
    }
}

pub async fn get_pages<T>(pages: &mut T) -> anyhow::Result<()>
where
    T: DerefMut<Target = BTreeMap<DateTime<Utc>, BlogPage>>,
{
    let mut entries = tokio::fs::read_dir("content").await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }
        if let Some(ext) = entry.path().extension() {
            if ext != "md" {
                continue;
            }
            if pages
                .values()
                .any(|p| p.id == entry.path().file_stem().unwrap().to_str().unwrap())
            {
                continue;
            }
            let content = tokio::fs::read_to_string(entry.path()).await?;
            match parse_page(&entry.path(), &content).await {
                Ok(p) => {
                    pages.insert(p.timestamp, p);
                }
                Err(e) => {
                    tracing::error!("Error in page {}: {}", &entry.path().display(), e);
                }
            }
        }
    }

    Ok(())
}

async fn parse_page(path: &std::path::Path, content: &str) -> anyhow::Result<BlogPage> {
    let stem = path
        .file_stem()
        .ok_or_else(|| anyhow!("Error while generating page URL"))?
        .to_str()
        .ok_or_else(|| anyhow!("Error while generating page URL"))?;

    let url = format!("/blog/{}", stem);

    let min = metadata::calculate_read_time(content);

    let metadata = metadata::parse_from_markdown(content)?;

    Ok(BlogPage {
        id: stem.to_string(),
        title: metadata::find_single(&metadata, "title")?,
        description: metadata::find_single(&metadata, "description").ok(),
        authors: metadata::find_multiple(&metadata, "author"),
        url,
        image: metadata::find_single(&metadata, "image").ok(),
        image_alt: metadata::find_single(&metadata, "image_alt").ok(),
        tags: metadata::find_multiple(&metadata, "tag"),
        timestamp: metadata::find_timestamp(&metadata, "timestamp")?,
        edit_timestamp: metadata::find_timestamp(&metadata, "edit_timestamp").ok(),
        time_to_read: min,
    })
}

pub async fn page(
    State(cache): State<Cache>,
    Path(page): Path<String>,
) -> Result<BlogPageTemplate, StatusCode> {
    let path = PathBuf::from(format!("content/{}.md", page));
    let s = match tokio::fs::read_to_string(&path).await {
        Ok(a) => a,
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                return Err(StatusCode::NOT_FOUND);
            }

            let tmp = PathBuf::from(format!("content/{}.md.new", page));
            match tokio::fs::read_to_string(tmp).await {
                Ok(o) => o,
                Err(_) => return Err(StatusCode::NOT_FOUND),
            }
        } // FIXME: add other status codes
    };

    let metadata = parse_page(&path, &s)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let adapter = SyntectAdapter::new(Some("Solarized (dark)"));
    let mut opt = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);
    opt.extension.header_ids = Some(String::new());
    opt.extension.math_code = true;
    opt.extension.strikethrough = true;
    opt.extension.superscript = true;
    opt.extension.table = true;
    opt.extension.tasklist = true;
    opt.extension.footnotes = true;
    opt.render.unsafe_ = true;

    Ok(BlogPageTemplate {
        page,
        metadata,
        content: markdown_to_html_with_plugins(&s, &opt, &plugins),
        page_name: "blog",
        root_url: crate::ROOT_URL,
    })
}

#[derive(Template)]
#[template(path = "page.html")]
pub struct BlogPageTemplate {
    page: String,
    content: String,
    metadata: BlogPage,
    page_name: &'static str,
    root_url: &'static str,
}
