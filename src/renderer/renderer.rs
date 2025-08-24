use crate::{
    page::types::Page,
    renderer::renderedpage::{RenderedPage, RenderedingPageData},
    store::factory::StoreFactory,
};
use minijinja::Environment;
use pulldown_cmark::{html::push_html, Parser};

use tracing::instrument;

pub struct PageRenderer {
    pub store: StoreFactory,
    pub env: Environment<'static>,
}

impl std::fmt::Debug for PageRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PageRenderer")
    }
}

fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let parser = Parser::new(markdown);
    push_html(&mut html, parser);
    html
}

impl PageRenderer {
    pub fn new() -> Self {
        let store = StoreFactory::new();
        let mut env = Environment::new();
        env.add_filter("markdown", markdown_to_html);

        Self { store, env }
    }

    #[instrument]
    pub fn render_page(&self, page: &Page, ctx: &minijinja::Value) -> anyhow::Result<RenderedPage> {
        let mut rendering_page = RenderedingPageData::new(&page, &self.store, &self.env);

        rendering_page.render(ctx)?;
        let rendered_page = rendering_page.rendered_page;
        Ok(rendered_page)
    }
}
