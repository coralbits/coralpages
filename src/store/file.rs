use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use serde::Deserialize;
use tracing::{error, info};

use crate::{
    page::types::{Page, PageInfo, ResultPageList, Widget},
    store::traits::Store,
    WidgetResults,
};

#[derive(Debug, Deserialize)]
struct FileStoreConfig {
    widgets: Vec<Widget>,
}

pub struct FileStore {
    name: String,
    path: PathBuf,
    widgets: HashMap<String, Widget>,
}

impl FileStore {
    pub fn new(name: &str, path: &str) -> anyhow::Result<Self> {
        let mut ret = Self {
            name: name.to_string(),
            path: Path::new(&path).to_path_buf(),
            widgets: HashMap::new(),
        };

        ret.load_widgets(&ret.path.join("config.yaml"))?;

        Ok(ret)
    }

    fn load_widgets(&mut self, config_path: &Path) -> anyhow::Result<()> {
        if !config_path.exists() {
            info!(
                "Widgets config not found, path={}, no widgets loaded",
                config_path.display()
            );
            return Ok(());
        }

        let config = self.load_config(config_path)?;

        let widgets: HashMap<String, Widget> = config
            .widgets
            .into_iter()
            .map(|w| (w.name.clone(), w))
            .collect();

        info!("Loaded widget_count={}", widgets.len());
        self.widgets.extend(widgets);

        Ok(())
    }

    fn load_config(&mut self, path: &Path) -> anyhow::Result<FileStoreConfig> {
        let file = File::open(path)?;
        let mut config: FileStoreConfig = serde_yaml::from_reader(file)?;

        // Load all widgets HTML and CSS
        for widget in config.widgets.iter_mut() {
            if !widget.html.is_empty() {
                let html_path = self.path.join(&widget.html);
                let Ok(html) = fs::read_to_string(&html_path) else {
                    error!(
                        "Widget type={} HTML file not found, filename={}",
                        widget.name,
                        html_path.display()
                    );
                    return Err(anyhow::anyhow!(
                        "Widget type={} HTML file not found, filename={}",
                        widget.name,
                        html_path.display()
                    ));
                };
                widget.html = html;
            }

            if !widget.css.is_empty() {
                let css_path = self.path.join(&widget.css);
                let Ok(css) = fs::read_to_string(&css_path) else {
                    error!(
                        "Widget type={} CSS file not found, filename={}",
                        widget.name,
                        css_path.display()
                    );
                    return Err(anyhow::anyhow!(
                        "Widget type={} CSS file not found, filename={}",
                        widget.name,
                        css_path.display()
                    ));
                };
                widget.css = css;
            }
        }
        Ok(config)
    }
}

#[async_trait]
impl Store for FileStore {
    fn name(&self) -> &str {
        &self.name
    }

    async fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
        // debug!(
        //     "Loading widget definition from path={} available_count={}",
        //     path,
        //     self.widgets.len()
        // );
        let widget = self.widgets.get(path).map(|w| Widget {
            name: w.name.clone(),
            html: w.html.clone(),
            css: w.css.clone(),
            editor: w.editor.clone(),
            description: w.description.clone(),
            icon: w.icon.clone(),
        });
        Ok(widget)
    }

    async fn load_page_definition(&self, path: &str) -> anyhow::Result<Option<Page>> {
        let path = Path::new(&self.path).join(format!("{}.yaml", path));
        // info!("Loading page definition from {}", path.display());
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                error!(
                    "Error loading page definition from path={}: {}",
                    path.display(),
                    e
                );
                return Ok(None);
            }
        };
        let page: Page = serde_yaml::from_reader(file)?;
        Ok(Some(page))
    }

    async fn save_page_definition(&self, path: &str, page: &Page) -> anyhow::Result<()> {
        let path = Path::new(&self.path).join(format!("{}.yaml", path));
        let file = File::create(path)?;
        serde_yaml::to_writer(file, page)?;
        Ok(())
    }

    async fn delete_page_definition(&self, path: &str) -> anyhow::Result<bool> {
        let path = Path::new(&self.path).join(format!("{}.yaml", path));
        if path.exists() {
            std::fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_page_list(
        &self,
        offset: usize,
        limit: usize,
        filter: &HashMap<String, String>,
    ) -> anyhow::Result<ResultPageList> {
        let path = Path::new(&self.path);
        let mut pages: Vec<PageInfo> = Vec::new();
        info!("Getting page list from path={}", path.display());
        let entries = fs::read_dir(path);

        let filter_type = filter.get("type");

        if entries.is_err() {
            error!("Error getting page list from path={}", path.display());
            return Ok(ResultPageList {
                count: 0,
                results: vec![],
            });
        }

        let entries = entries.unwrap();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "yaml" {
                // page path wihtout the .yaml extension, and the self.path prefix, and prefix /
                let path_str = path.to_str().unwrap();
                let page_id =
                    path_str[self.path.to_str().unwrap().len()..path_str.len() - 5].to_string();

                if let Some(filter_type) = filter_type {
                    if filter_type == "template" && !path_str.starts_with("_") {
                        continue; // skip non templates
                    }
                    if filter_type == "page" && path_str.starts_with("_") {
                        continue; // skip non pages
                    }
                }

                // info!("Loading page definition from path={}", page_id);
                let page = match self.load_page_definition(&page_id).await {
                    Ok(page) => page,
                    Err(e) => {
                        error!("Error loading page definition from path={}: {}", page_id, e);
                        continue;
                    }
                };
                if let Some(page) = page {
                    let pageinfo: PageInfo = PageInfo {
                        id: page_id,
                        store: "".to_string(),
                        title: page.title.clone(),
                        url: format!("/{}", page.path).to_string(),
                    };
                    pages.push(pageinfo);
                }
            }
        }

        let count = pages.len();
        let pages = pages.into_iter().skip(offset).take(limit).collect();
        Ok(ResultPageList {
            count,
            results: pages,
        })
    }

    async fn get_widget_list(&self) -> anyhow::Result<WidgetResults> {
        let result = WidgetResults {
            count: self.widgets.len(),
            results: self.widgets.values().cloned().collect(),
        };
        Ok(result)
    }
}
