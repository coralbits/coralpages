use crate::{
    page::types::Page,
    renderer::renderedpage::{RenderedPage, RenderedingPageData},
    store::factory::StoreFactory,
};

use tracing::instrument;

pub struct PageRenderer {
    pub store: StoreFactory,
}

impl std::fmt::Debug for PageRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PageRenderer")
    }
}

impl PageRenderer {
    pub fn new() -> Self {
        let store = StoreFactory::new();
        Self { store }
    }

    #[instrument]
    pub fn render_page(&self, page: &Page) -> anyhow::Result<RenderedPage> {
        let mut rendering_page = RenderedingPageData::new(&page, &self.store);

        rendering_page.render()?;
        let rendered_page = rendering_page.rendered_page;
        Ok(rendered_page)
    }
}
