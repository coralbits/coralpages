use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};

use serde::{Deserialize, Serialize};
use tracing::error;

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
    path: String,
    widgets: HashMap<String, FileWidget>,
}

impl FileStore {
    pub fn new(path: String) -> anyhow::Result<Self> {
        let config_path = Path::new(&path).join("config.yaml");
        let file = File::open(config_path)?;
        let mut config: FileStoreConfig = serde_yaml::from_reader(file)?;

        // Load all widgets HTML and CSS
        for widget in config.widgets.iter_mut() {
            if !widget.html.is_empty() {
                let html_path = Path::new(&path).join(&widget.html);
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
                let css_path = Path::new(&path).join(&widget.css);
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

        Ok(Self {
            path,
            widgets: config
                .widgets
                .into_iter()
                .map(|w| (w.name.clone(), w))
                .collect(),
        })
    }
}

impl Store for FileStore {
    fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
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
