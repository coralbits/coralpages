// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::{sqlite::SqlitePool, Executor, Row};
use tracing::{debug, error, info};

use crate::{page::types::Page, store::traits::Store, PageInfo, ResultPageList};

pub struct DbStore {
    name: String,
    db: SqlitePool,
}

impl DbStore {
    pub async fn new(name: &str, url: &str) -> Result<Self> {
        info!("Connecting to database at url={}", url);
        // Create database file if it doesn't exist
        if url.starts_with("sqlite://") {
            let path = url.trim_start_matches("sqlite://");
            if !std::path::Path::new(path).exists() {
                debug!("Creating new SQLite database file at {}", path);
                std::fs::File::create(path)?;
            }
        }
        let db = SqlitePool::connect(url).await?;
        let ret = Self {
            name: name.to_string(),
            db,
        };

        ret.init().await?;

        Ok(ret)
    }

    async fn init(&self) -> Result<()> {
        let mut tx = self.db.begin().await?;
        tx.execute("CREATE TABLE IF NOT EXISTS pages (path TEXT PRIMARY KEY, data JSON)")
            .await?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS elements (path TEXT PRIMARY KEY, html TEXT, css TEXT, data JSON)",
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

#[async_trait]
impl Store for DbStore {
    fn name(&self) -> &str {
        &self.name
    }

    async fn load_page_definition(&self, path: &str) -> anyhow::Result<Option<Page>> {
        let rec = sqlx::query(r#"SELECT data FROM pages WHERE path = ?"#)
            .bind(path)
            .fetch_one(&self.db)
            .await;

        match rec {
            Ok(rec) => {
                let page: Page = serde_json::from_str(&rec.get::<String, _>("data"))?;
                Ok(Some(page))
            }
            Err(e) => {
                error!("Error loading page definition from path={}: {}", path, e);
                Ok(None)
            }
        }
    }

    async fn save_page_definition(&self, path: &str, page: &Page) -> anyhow::Result<()> {
        let data = serde_json::to_string(page)?;
        sqlx::query(r#"INSERT OR REPLACE INTO pages (path, data) VALUES (?, ?)"#)
            .bind(path)
            .bind(data)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn get_page_list(
        &self,
        _offset: usize,
        _limit: usize,
        _filter: &HashMap<String, String>,
    ) -> anyhow::Result<ResultPageList> {
        debug!("get_page_list offset={}, limit={}", _offset, _limit);
        let recs = sqlx::query(r#"SELECT path, data FROM pages LIMIT ? OFFSET ?"#)
            .bind(_limit as i64)
            .bind(_offset as i64)
            .fetch_all(&self.db)
            .await?;

        let pages = recs
            .iter()
            .map(|row| {
                let page_def = row.get::<String, _>("data");
                let page: Page = match serde_json::from_str(&page_def) {
                    Ok(page) => page,
                    Err(e) => {
                        error!(
                            "Error loading page definition from path={}: {}",
                            row.get::<String, _>("path"),
                            e
                        );
                        return None;
                    }
                };
                Some(PageInfo {
                    id: row.get::<String, _>("path"),
                    title: page.title.clone(),
                    url: "".to_string(),
                    store: "".to_string(),
                })
            })
            .collect::<Vec<Option<PageInfo>>>();

        let count = sqlx::query("SELECT COUNT(*) FROM pages")
            .fetch_one(&self.db)
            .await?
            .get::<i64, _>(0);

        // remove None from pages
        let pages = pages
            .into_iter()
            .filter_map(|p| p)
            .collect::<Vec<PageInfo>>();

        Ok(ResultPageList {
            count: count as usize,
            results: pages,
        })
    }
}
