use atom_feed::{AtomEntry, Generator, Person};
use axum::{extract::State, http::StatusCode};
use chrono::Utc;

use crate::{blog, Cache};
use blog::BlogPage;

pub async fn atom(State(cache): State<Cache>) -> Result<axum::http::Response<String>, StatusCode> {
    let pages = cache.0.read().await;
    // fixup
    let pages: Vec<&BlogPage> = pages.values().collect();

    let mut entries = vec![];

    for page in pages {
        let url = format!("{}{}", crate::ROOT_URL, page.url);
        let mut authors = Vec::with_capacity(page.authors.len());
        for author in page.authors.clone() {
            authors.push(Person::new(author));
        }
        let mut entry = AtomEntry::new(&page.title)
            .id(&page.id)
            .uri(url)
            .published(page.timestamp)
            .authors(authors);
        if let Some(desc) = &page.description {
            entry = entry.summary(desc);
        }
        entry = if let Some(edit) = page.edit_timestamp {
            entry.updated(edit)
        } else {
            entry.updated(page.timestamp)
        };
        entries.push(entry);
    }

    let feed = atom_feed::AtomFeedBuilder::new("Lena's blog :3")
        .subtitle("Lena's blog feed")
        .uri(crate::ROOT_URL)
        .self_uri(crate::ATOM_URL)
        .generator(
            Generator::new("Lena's Atom feed generator")
                .uri(crate::ROOT_URL)
                .version("1.0.0"),
        )
        .id(crate::ROOT_URL)
        .updated(Utc::now())
        .subtitle("Atom Feed Edition")
        .entries(entries)
        .build();

    let xml = String::from_utf8(feed.write_to(Vec::new()).unwrap()).unwrap();

    axum::http::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/atom+xml")
        .body(xml)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
