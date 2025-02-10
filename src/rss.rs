use axum::{extract::State, http::StatusCode};

use crate::{blog, Cache};

#[axum::debug_handler]
pub async fn rss(State(cache): State<Cache>) -> Result<axum::http::Response<String>, StatusCode> {
    let pages = cache.0.read().await;

    let items = get_pages_rss(pages.values()).await;

    let channel = rss::ChannelBuilder::default()
        .title("Lena's blog :3".to_string())
        .link(crate::ROOT_URL.to_string())
        .description("Lena's blog feed".to_string())
        .items(items)
        .build();

    let xml = String::from_utf8(channel.write_to(Vec::new()).unwrap()).unwrap();

    axum::http::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/xml")
        .body(xml)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_pages_rss<'a, T>(pages: T) -> Vec<rss::Item>
where
    T: IntoIterator<Item = &'a blog::BlogPage>,
{
    let mut items = vec![];

    for page in pages.into_iter() {
        let item = rss::ItemBuilder::default()
            .title(Some(page.title.clone()))
            .link(Some(format!("{}{}", crate::ROOT_URL, page.url)))
            .description(page.description.clone())
            .pub_date(Some(page.timestamp.to_rfc2822()))
            .build();

        items.push(item);
    }

    items
}
