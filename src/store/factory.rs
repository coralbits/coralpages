use std::collections::HashMap;

use tracing::{error, info};

use crate::{
    page::types::{Page, PageListResult, Widget},
    store::{traits::Store, types::PageInfo},
    ResultI,
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
            error!("Store not found: {}", name);
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
            return Err(anyhow::anyhow!("Invalid path"));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }
}

impl Store for StoreFactory {
    fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
        let (store, path) = self.split_path(path)?;
        let store = self.get_store(&store);
        if let Some(store) = store {
            store.load_widget_definition(&path)
        } else {
            Err(anyhow::anyhow!("Store not found"))
        }
    }

    fn load_page_definition(&self, path: &str) -> anyhow::Result<Option<Page>> {
        let (store, path) = self.split_path(path)?;
        let store = self.get_store(&store);
        if let Some(store) = store {
            store.load_page_definition(&path)
        } else {
            Err(anyhow::anyhow!("Store not found"))
        }
    }

    fn save_page_definition(&self, path: &str, page: &Page) -> anyhow::Result<()> {
        let (store, path) = self.split_path(path)?;
        let store = self.get_store(&store);
        if let Some(store) = store {
            store.save_page_definition(&path, page)
        } else {
            Err(anyhow::anyhow!("Store not found"))
        }
    }

    fn delete_page_definition(&self, path: &str) -> anyhow::Result<bool> {
        let (store, path) = self.split_path(path)?;
        let store = self.get_store(&store);
        if let Some(store) = store {
            store.delete_page_definition(&path)
        } else {
            Err(anyhow::anyhow!("Store not found"))
        }
    }

    fn get_page_list(
        &self,
        offset: usize,
        limit: usize,
        filter: &Option<HashMap<String, String>>,
    ) -> anyhow::Result<ResultI<PageInfo>> {
        let mut result = ResultI {
            count: 0,
            results: Vec::new(),
        };
        for store in self.stores.values() {
            let store_result = store.get_page_list(offset, limit, filter)?;
            result.count += store_result.count;
            result.results.extend(store_result.results);
        }
        Ok(result)
    }
}
