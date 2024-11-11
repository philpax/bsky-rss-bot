use crate::database::Database;
use anyhow::Result;
use chrono::{Duration, Utc};
use feed_rs::model::Feed;
use log::debug;
use reqwest::Url;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RSSHandler {
    filter_date: chrono::DateTime<Utc>,
    database: Arc<Database>,
    feed: Url,
}

impl RSSHandler {
    pub fn new(feed: Url, database: Arc<Database>, feed_backdate_hours: u16) -> Self {
        let filter_date = Utc::now() - Duration::hours(feed_backdate_hours as i64);
        debug!("Initializing RSS handler for {feed} with starting filter date of {filter_date}");
        Self {
            database,
            feed,
            filter_date,
        }
    }

    pub fn get_feed(&self) -> &Url {
        &self.feed
    }

    pub async fn fetch_unposted(&mut self) -> Result<Feed> {
        let content = reqwest::get(self.feed.clone()).await?.bytes().await?;
        let mut feed = feed_rs::parser::parse(&content[..])?;
        let mut new_entries = vec![];
        for item in feed.entries {
            // Only count posts that are after the filter date.
            if let Some(pub_date) = item.published {
                if pub_date <= self.filter_date {
                    continue;
                }
            } else {
                continue;
            }
            // Check for duplicate link. No link, no post.
            if let Some(link) = item.links.first() {
                if self.database.has_posted_url(&link.href).await? {
                    continue;
                }
            } else {
                continue;
            }
            new_entries.push(item);
        }
        self.filter_date = Utc::now();
        feed.entries = new_entries;
        Ok(feed)
    }
}
