use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use tracing::{debug, error, info};

use crate::{
    page::types::{Page, Widget},
    store::traits::Store,
    WidgetEditor,
};

#[derive(Debug, Deserialize)]
struct FileWidget {
    name: String,
    html: String,
    #[serde(default)]
    css: String,
    #[serde(default)]
    editor: Vec<WidgetEditor>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    icon: String,
}

#[derive(Debug, Deserialize)]
struct FileStoreConfig {
    widgets: Vec<FileWidget>,
}

pub struct FileStore {
    path: PathBuf,
    widgets: HashMap<String, FileWidget>,
}

impl FileStore {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let mut ret = Self {
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

        let widgets: HashMap<String, FileWidget> = config
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

impl Store for FileStore {
    fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
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

    fn load_page_definition(&self, path: &str) -> anyhow::Result<Option<Page>> {
        let path = Path::new(&self.path).join(format!("{}.yaml", path));
        // info!("Loading page definition from {}", path.display());
        let file = File::open(path)?;
        let page: Page = serde_yaml::from_reader(file)?;
        Ok(Some(page))
    }

    fn save_page_definition(&self, path: &str, page: &Page) -> anyhow::Result<()> {
        let path = Path::new(&self.path).join(format!("{}.yaml", path));
        let file = File::create(path)?;
        serde_yaml::to_writer(file, page)?;
        Ok(())
    }

    fn delete_page_definition(&self, path: &str) -> anyhow::Result<bool> {
        let path = Path::new(&self.path).join(format!("{}.yaml", path));
        if path.exists() {
            std::fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_page_list(
        &self,
        offset: usize,
        limit: usize,
        filter: &Option<std::collections::HashMap<String, String>>,
    ) -> anyhow::Result<crate::ResultI<super::types::PageInfo>> {
        todo!()
    }
}
