use axum::http::StatusCode;

use crate::blog;
use blog::BlogPage;

pub async fn rss() -> Result<axum::http::Response<String>, StatusCode> {
    let pages = blog::get_pages().await.unwrap();
    let items = get_pages_rss(&pages[..]).await;

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

async fn get_pages_rss(pages: &[BlogPage]) -> Vec<rss::Item> {
    let mut items = vec![];

    for page in pages {
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
