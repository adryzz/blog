use askama::Template;
use axum::{extract::Path, http::StatusCode};
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};

use std::path::PathBuf;

use anyhow::anyhow;
use axum::{routing::get, Router};
use chrono::prelude::*;

pub async fn blog() -> Result<BlogTemplate, StatusCode> {
    let pages = match get_pages().await {
        Ok(p) => p,
        Err(e) => {
            vec![]
        }
    };

    Ok(BlogTemplate { pages })
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogTemplate {
    pages: Vec<BlogPage>,
}



pub struct BlogPage {
    pub title: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub url: String,
    pub tags: Vec<String>,
    pub timestamp: NaiveDateTime,
    pub edit_timestamp: Option<NaiveDateTime>,
    pub time_to_read: u32,
}

pub async fn get_pages() -> anyhow::Result<Vec<BlogPage>> {
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


pub async fn page(Path(page): Path<String>) -> Result<BlogPageTemplate, StatusCode> {
    let s = match tokio::fs::read_to_string(format!("blog/{}.md", page)).await {
        Ok(a) => a,
        Err(_) => return Err(StatusCode::NOT_FOUND), // FIXME: add other status codes
    };

    let adapter = SyntectAdapter::new("base16-ocean.dark");
    let mut opt = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    opt.render.unsafe_ = true;

    Ok(BlogPageTemplate {
        page,
        content: markdown_to_html_with_plugins(&s, &opt, &plugins),
    })
}

#[derive(Template)]
#[template(path = "page.html")]
pub struct BlogPageTemplate {
    page: String,
    content: String,
}
