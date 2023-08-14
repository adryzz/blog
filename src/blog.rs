use askama::Template;
use axum::{extract::Path, http::StatusCode};
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};

pub async fn blog(Path(page): Path<String>) -> Result<BlogTemplate, StatusCode> {
    let s = match tokio::fs::read_to_string(format!("blog/{}.md", page)).await {
        Ok(a) => a,
        Err(_) => return Err(StatusCode::NOT_FOUND), // FIXME: add other status codes
    };

    let adapter = SyntectAdapter::new("base16-ocean.dark");
    let mut opt = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    opt.render.unsafe_ = true;

    Ok(BlogTemplate {
        page,
        content: markdown_to_html_with_plugins(&s, &opt, &plugins),
    })
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogTemplate {
    page: String,
    content: String,
}
