use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{error, info};

use crate::{
    file::FileStore,
    page::types::{Page, ResultPageList, Widget},
    store::traits::Store,
    StoreConfig, WidgetResults,
};

pub struct StoreFactory {
    stores: HashMap<String, Box<dyn Store>>,
}

impl StoreFactory {
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    pub fn get_store(&self, name: &str) -> Option<&dyn Store> {
        let store = self.stores.get(name).map(|s| s.as_ref());
        if store.is_none() {
            error!("Store not found name={}", name);
        }
        store
    }

    pub fn add_store(&mut self, name: &str, store: Box<dyn Store>) {
        self.stores.insert(name.to_string(), store);
        info!("Added store: {}", name);
    }

    fn split_path(&self, path: &str) -> Result<(String, String), anyhow::Error> {
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid path={}", path));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    pub fn new_store(store_config: &StoreConfig) -> Result<Box<dyn Store>> {
        match store_config.store_type.as_str() {
            "file" => Ok(Box::new(FileStore::new(&store_config.path)?)),
            _ => Err(anyhow::anyhow!(
                "Unsupported store type: {}",
                store_config.store_type
            )),
        }
    }
}

#[async_trait]
impl Store for StoreFactory {
    async fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
        // info!("Loading widget definition, path={}", path);
        let (store, subpath) = self.split_path(path)?;
        let store = self.get_store(&store);
        if let Some(store) = store {
            store.load_widget_definition(&subpath).await
        } else {
            Err(anyhow::anyhow!("Store for widget not found, path={}", path))
        }
    }

    async fn load_page_definition(&self, path: &str) -> anyhow::Result<Option<Page>> {
        let (store, subpath) = self.split_path(path)?;
        // store can be with a | to mark several options, that should be checked in order
        let stores = store.split('|');
        for store in stores {
            let store = self.get_store(store);
            if let Some(store) = store {
                let page = store.load_page_definition(&subpath).await?;
                if page.is_some() {
                    return Ok(page);
                }
            }
        }
        Err(anyhow::anyhow!(
            "Page not found in any store, path={}",
            path
        ))
    }

    async fn save_page_definition(&self, path: &str, page: &Page) -> anyhow::Result<()> {
        let (store, subpath) = self.split_path(path)?;
        let store = self.get_store(&store);
        if let Some(store) = store {
            store.save_page_definition(&subpath, page).await
        } else {
            Err(anyhow::anyhow!(
                "Store for page save not found, path={}",
                path
            ))
        }
    }

    async fn delete_page_definition(&self, path: &str) -> anyhow::Result<bool> {
        let (store, subpath) = self.split_path(path)?;
        let store = self.get_store(&store);
        if let Some(store) = store {
            store.delete_page_definition(&subpath).await
        } else {
            Err(anyhow::anyhow!(
                "Store for page delete not found, path={}",
                path
            ))
        }
    }

    async fn get_page_list(
        &self,
        offset: usize,
        limit: usize,
        filter: &HashMap<String, String>,
    ) -> anyhow::Result<ResultPageList> {
        let mut result = ResultPageList {
            count: 0,
            results: Vec::new(),
        };
        let mut offset = offset;
        let mut limit = limit;

        let filter_store = filter.get("store");

        for (store_name, store) in self.stores.iter() {
            if let Some(filter_store) = filter_store {
                if filter_store != store_name {
                    continue;
                }
            }
            let store_result = store.get_page_list(offset, limit, filter).await?;
            // Update offset and limit
            offset = result.count.saturating_sub(offset); // limits to minimum 0
            let ret_count = store_result.results.len();
            limit = limit.saturating_sub(ret_count);
            // add to results
            result.count += store_result.count;
            let results = store_result.results.into_iter().map(|mut r| {
                r.store = store_name.clone();
                r
            });
            result.results.extend(results);
        }
        Ok(result)
    }

    async fn get_widget_list(&self) -> anyhow::Result<WidgetResults> {
        let mut result = WidgetResults {
            count: 0,
            results: Vec::new(),
        };
        for (store_name, store) in self.stores.iter() {
            let mut store_result = store.get_widget_list().await?;
            for w in store_result.results.iter_mut() {
                w.name = format!("{}/{}", store_name, w.name);
                w.html = "".to_string();
                w.css = "".to_string();
            }
            result.count += store_result.count;
            result.results.extend(store_result.results);
        }
        Ok(result)
    }
}
