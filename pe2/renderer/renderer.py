from dataclasses import dataclass, field
from functools import lru_cache
from typing import Any

from pe2.loader import LoaderFactory
from pe2.renderer.types import ElementRendererBase
from pe2.types import BlockDefinition, PageDefinition


@dataclass
class RenderedPage:
    """
    A rendered page.

    This class is used to render a page.
    It contains the title, content, classes, and max_id.

    The classes are a dict of CSS classes, the key is to avoid repetition, all the css to be inserted is the value.
    """

    title: str
    content: str = ""
    classes: dict[str, str] = field(default_factory=dict)
    max_id: int = 1
    context: dict[str, Any] = field(default_factory=dict)

    def append_content(self, content: str):
        """
        Append content to the page. Also increments the max_id.
        """
        self.content += content
        self.max_id += 1

    def get_current_id(self, prefix: str = "id-") -> int:
        """
        Get the current id.
        """
        return f"{prefix}-{self.max_id}"

    def __str__(self):
        """
        Render the page as a string.
        """
        css = "\n".join(self.classes.values())
        content = self.content
        title = self.title
        return f"""<!DOCTYPE html>
<html>
<head>
<title>{title}</title>
<style type="text/css">{css}</style>
</head>
<body>
{content}
</body>
</html>
"""


class Renderer:
    """
    Renderer for the page editor.
    """

    def __init__(self, config: dict[str, str]):
        """
        Initialize the renderer.
        """
        self.config = config
        self.loader = LoaderFactory(config=config)
        self.page = None

    def new_page(self) -> RenderedPage:
        """
        Create a new page.
        """
        return RenderedPage(title="")

    async def render_page(self, page_name: str) -> str:
        """
        Render a page asynchronously.
        """
        page = self.loader.load_page(page_name)
        return await self.render(page)

    async def render(self, page_def: PageDefinition) -> str:
        """
        Render a page asynchronously.
        """
        page = self.new_page()
        page.title = page_def.title

        await self.render_page_data(page, page_def)
        if page_def.template:
            await self.render_in_template(page, page_def.template)

        return page

    async def render_page_data(
        self, page: RenderedPage, page_def: PageDefinition
    ) -> str:
        """
        Render the page data asynchronously.
        """
        for block in page_def.data:
            html, css = await self.render_block(page, block)
            cid = page.get_current_id(block.type)

            if block.style:
                css_id = f"#{cid}"
                page.classes.update(
                    {css_id: f"{css_id} {{\n{css_dict_to_cs_text(block.style)} \n}}"}
                )
            html = html.replace("@@class@@", cid)
            html = html.replace("@@id@@", cid)

            # add the css to the page and content
            page.classes.update({block.type: css})
            page.append_content(html)

    async def render_in_template(self, page: RenderedPage, template_name: str) -> str:
        """
        Render the template asynchronously.

        It assumes that in the page.data there is already all the data rendered.

        It is like rendering a page, but the previour content is set at the "children" elements.
        """
        template = self.loader.load_page(template_name)
        page.context = {
            **page.context,
            "children": page.content,
        }
        page.content = ""
        await self.render_page_data(page, template)
        if template.template:
            await self.render_in_template(page, template.template)

    async def render_block(
        self, page: RenderedPage, block: BlockDefinition
    ) -> tuple[str, dict[str, str]]:
        """
        Render a block asynchronously.
        """
        element_renderer = self.get_element_renderer(block.type)

        html = await element_renderer.render_html(data=block.data, context=page.context)
        css = await element_renderer.render_css(data=block.data, context=page.context)

        return html, css

    @lru_cache(maxsize=100)
    def get_element_renderer(self, type_name: str) -> ElementRendererBase:
        """
        Get an element renderer.
        """
        block = self.loader.load_element_definition(type_name)

        if block.viewer.startswith("builtin://"):
            from pe2.renderer.builtin import ElementRendererBuiltin

            return ElementRendererBuiltin(block=block)

        if block.viewer.startswith("https://") or block.viewer.startswith("http://"):
            from pe2.renderer.http import ElementRendererHttp

            return ElementRendererHttp(block=block)

        raise ValueError(f"Unknown element renderer type: {type_name}, {block.viewer}")


def css_dict_to_cs_text(css: dict[str, str]) -> str:
    """
    Convert a dict of CSS to a string.
    """
    return "\n".join([f"  {k}: {v};" for k, v in css.items()])
