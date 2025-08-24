use std::{fs::File, path::Path};

use crate::{
    page::types::{Page, Widget},
    store::traits::Store,
    WidgetEditor,
};

pub struct FileStore {
    path: String,
}

impl FileStore {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Store for FileStore {
    fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
        return Ok(Some(Widget {
            name: "test".to_string(),
            html: "{{data.text}}".to_string(),
            css: "test".to_string(),
            editor: vec![WidgetEditor {
                name: "test".to_string(),
                editor_type: "test".to_string(),
                label: "test".to_string(),
                placeholder: "test".to_string(),
            }],
            description: "test".to_string(),
            icon: "test".to_string(),
        }));

        let path = Path::new(&self.path).join(format!("{}.yaml", path));
        let file = File::open(path)?;
        let widget: Widget = serde_yaml::from_reader(file)?;
        Ok(Some(widget))
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
