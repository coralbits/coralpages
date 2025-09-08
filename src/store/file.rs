// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

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
    CssClass, CssClassResult, CssClassResults, StoreConfig, WidgetResults,
};

#[derive(Debug, Deserialize)]
struct CssClasses {
    css_classes: Vec<CssClass>,
}

#[derive(Debug, Deserialize)]
struct FileStoreConfig {
    #[serde(default)]
    widgets: Vec<Widget>,
}

pub struct FileStore {
    name: String,
    path: PathBuf,
    widgets: HashMap<String, Widget>,
    css_classes: HashMap<String, CssClass>,
    has_widgets: bool,
    has_css_classes: bool,
    has_pages: bool,
}

impl FileStore {
    pub fn new(config: &StoreConfig) -> anyhow::Result<Self> {
        let mut ret = Self {
            name: config.name.clone(),
            path: Path::new(&config.path).to_path_buf(),
            widgets: HashMap::new(),
            css_classes: HashMap::new(),
            has_widgets: config.tags.contains(&"widgets".to_string()),
            has_css_classes: config.tags.contains(&"css_classes".to_string()),
            has_pages: config.tags.contains(&"pages".to_string()),
        };

        if ret.has_widgets {
            ret.load_widgets(&ret.path.join("config.yaml"))?;
        }

        if ret.has_css_classes {
            ret.load_css_classes_config(&ret.path.clone())?;
        }

        Ok(ret)
    }

    fn load_css_classes_config(&mut self, config_path: &Path) -> anyhow::Result<()> {
        if !self.has_css_classes {
            return Ok(());
        }
        self.load_css_classes_path(&config_path)?;
        Ok(())
    }

    fn load_css_classes_path(&mut self, path: &Path) -> anyhow::Result<()> {
        // read all *.yaml files at cconfig_path, as CssClass
        let mut css_classes: HashMap<String, CssClass> = HashMap::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "yaml" {
                let css_class = match self.load_css_class_config(&path) {
                    Ok(css_class) => css_class,
                    Err(e) => {
                        error!(
                            "Error loading CSS class from path={}: {}",
                            path.display(),
                            e
                        );
                        continue;
                    }
                };
                for css_class in css_class.css_classes {
                    css_classes.insert(css_class.name.clone(), css_class);
                }
            }
        }
        info!(
            "Loading CSS classes from path={} count={}",
            path.display(),
            css_classes.len()
        );

        self.css_classes.extend(css_classes);

        Ok(())
    }

    fn load_css_class_config(&mut self, path: &Path) -> anyhow::Result<CssClasses> {
        let file = File::open(path)?;
        let css_class: CssClasses = serde_yaml::from_reader(file)?;
        Ok(css_class)
    }

    fn load_widgets(&mut self, config_path: &Path) -> anyhow::Result<()> {
        if !config_path.exists() {
            info!(
                "Widgets config not found, path={}, no widgets loaded",
                config_path.display()
            );
            return Ok(());
        }

        let config = self.load_widget_config(config_path)?;

        let widgets: HashMap<String, Widget> = config
            .widgets
            .into_iter()
            .map(|w| (w.name.clone(), w))
            .collect();

        info!("Loaded widget_count={}", widgets.len());
        self.widgets.extend(widgets);

        Ok(())
    }

    fn load_widget_config(&mut self, path: &Path) -> anyhow::Result<FileStoreConfig> {
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
        if !self.has_widgets {
            return Ok(None);
        }
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
        if !self.has_pages {
            return Ok(None);
        }
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
        if !self.has_pages {
            return Ok(());
        }
        let path = Path::new(&self.path).join(format!("{}.yaml", path));
        let file = File::create(path)?;
        serde_yaml::to_writer(file, page)?;
        Ok(())
    }

    async fn delete_page_definition(&self, path: &str) -> anyhow::Result<bool> {
        if !self.has_pages {
            return Ok(false);
        }
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
        if !self.has_pages {
            return Ok(ResultPageList {
                count: 0,
                results: vec![],
            });
        }

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
        if !self.has_widgets {
            return Ok(WidgetResults {
                count: 0,
                results: vec![],
            });
        }

        let result = WidgetResults {
            count: self.widgets.len(),
            results: self.widgets.values().cloned().collect(),
        };
        Ok(result)
    }

    async fn load_css_classes(&self) -> anyhow::Result<CssClassResults> {
        if !self.has_css_classes {
            return Ok(CssClassResults {
                count: 0,
                results: vec![],
            });
        }

        let ret = CssClassResults {
            count: self.css_classes.len(),
            results: self
                .css_classes
                .values()
                .map(|c| CssClassResult {
                    name: format!("{}/{}", self.name, c.name.clone()),
                    description: c.description.clone(),
                    tags: c.tags.clone(),
                })
                .collect(),
        };

        Ok(ret)
    }

    async fn load_css_class_definition(&self, name: &str) -> anyhow::Result<Option<CssClass>> {
        if !self.has_css_classes {
            return Ok(None);
        }
        let css_class = self.css_classes.get(name).map(|c| c.clone());
        Ok(css_class)
    }
}
