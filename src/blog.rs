use askama::Template;
use axum::{extract::Path, http::StatusCode};
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};

use std::path::PathBuf;

use anyhow::anyhow;
use chrono::prelude::*;

use crate::metadata;

pub async fn blog() -> Result<BlogTemplate, StatusCode> {
    let pages = match get_pages().await {
        Ok(p) => p,
        Err(e) => {
            vec![]
        }
    };

    Ok(BlogTemplate {
        pages,
        page_name: "blog",
    })
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogTemplate {
    pages: Vec<BlogPage>,
    page_name: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BlogPage {
    pub title: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub url: String,
    pub image: Option<String>,
    pub image_alt: Option<String>,
    pub tags: Vec<String>,
    pub timestamp: NaiveDateTime,
    pub edit_timestamp: Option<NaiveDateTime>,
    pub time_to_read: u32,
}

impl PartialOrd for BlogPage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.timestamp.partial_cmp(&self.timestamp)
    }
}


impl Ord for BlogPage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.timestamp.cmp(&self.timestamp)
    }
}
pub async fn get_pages() -> anyhow::Result<Vec<BlogPage>> {
    let mut entries = tokio::fs::read_dir("blog").await?;

    let mut pages = vec![];

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    let content = tokio::fs::read_to_string(entry.path()).await?;
                    match parse_page(&entry.path(), &content).await {
                        Ok(p) => pages.push(p),
                        Err(e) => {
                            tracing::error!("Error in page {}: {}", &entry.path().display(), e)
                        }
                    }
                }
            }
        }
    }

    pages.sort();

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

    let metadata = metadata::parse_from_markdown(content)?;

    Ok(BlogPage {
        title: metadata::find_single(&metadata, "title")?,
        description: metadata::find_single(&metadata, "description").ok(),
        authors: metadata::find_multiple(&metadata, "author"),
        url,
        image: metadata::find_single(&metadata, "image").ok(),
        image_alt: metadata::find_single(&metadata, "image_alt").ok(),
        tags: metadata::find_multiple(&metadata, "tag"),
        timestamp: metadata::find_timestamp(&metadata, "timestamp")?,
        edit_timestamp: metadata::find_timestamp(&metadata, "edit_timestamp").ok(),
        time_to_read: 2,
    })
}

pub async fn page(Path(page): Path<String>) -> Result<BlogPageTemplate, StatusCode> {
    let path = PathBuf::from(format!("blog/{}.md", page));
    let s = match tokio::fs::read_to_string(&path).await {
        Ok(a) => a,
        Err(_) => return Err(StatusCode::NOT_FOUND), // FIXME: add other status codes
    };

    let metadata = parse_page(&path, &s).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;


    let adapter = SyntectAdapter::new("base16-ocean.dark");
    let mut opt = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    opt.render.unsafe_ = true;

    Ok(BlogPageTemplate {
        page,
        metadata,
        content: markdown_to_html_with_plugins(&s, &opt, &plugins),
        page_name: "blog",
    })
}

#[derive(Template)]
#[template(path = "page.html")]
pub struct BlogPageTemplate {
    page: String,
    content: String,
    metadata: BlogPage,
    page_name: &'static str,
}
