import datetime
import logging
from dataclasses import dataclass, field
from functools import lru_cache
import hashlib
import json
from typing import Any

from pe.loader import LoaderFactory
from pe.renderer.types import BlockRendererBase
from pe.types import BlockDefinition, PageDefinition

logger = logging.getLogger(__name__)


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
    headers: dict[str, str] = field(default_factory=dict)
    response_code: int = 200

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
        if self.response_code == 304:
            return ""

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

    def new_page(self) -> RenderedPage:
        """
        Create a new page.
        """
        return RenderedPage(title="")

    async def render_page(
        self, page_name: str, *, headers: dict[str, str] = {}
    ) -> RenderedPage:
        """
        Render a page asynchronously.

        Might use headers to check caches and so on
        """
        page_definition = self.loader.load(page_name)
        new_etag = None
        if "etag" in page_definition.cache:
            old_etag = headers.get("If-None-Match")
            new_etag = self.calculate_etag(page_definition)
            logger.debug(f"Old etag: %s, new etag: %s", old_etag, new_etag)
            if old_etag == new_etag:
                page = self.new_page()
                page.headers["ETag"] = old_etag
                page.response_code = 304
                return page
        if "last-modified" in page_definition.cache:
            old_last_modified = headers.get("If-Modified-Since")
            new_last_modified = self.calculate_last_modified(page_definition)
            if old_last_modified == new_last_modified:
                page = self.new_page()
                page.headers["Last-Modified"] = new_last_modified
                page.response_code = 304
                return page

        page = await self.render(page_definition)
        if new_etag:
            page.headers["ETag"] = new_etag
        if new_last_modified:
            page.headers["Last-Modified"] = new_last_modified
        return page

    def calculate_last_modified(self, page_definition: PageDefinition) -> str:
        """
        Calculate the last modified for a page definition.

        Looks at the last modified data of the page definition and the blocks.
        """
        return page_definition.last_modified.isoformat()

    def calculate_etag(self, page_definition: PageDefinition) -> str:
        """
        Calculate the etag for a page definition.
        """
        salt = datetime.datetime.now().strftime(self.config.server.etag_salt)

        return hashlib.sha256(
            json.dumps(page_definition.to_dict(), sort_keys=True).encode()
            + salt.encode()
        ).hexdigest()

    async def render(self, page_def: PageDefinition) -> RenderedPage:
        """
        Render a page asynchronously.
        """
        page = self.new_page()
        page.title = page_def.title

        await self.render_page_data(page=page, page_def=page_def)
        if page_def.template:
            await self.render_in_template(page=page, template_name=page_def.template)

        return page

    async def render_page_data(
        self, *, page: RenderedPage, page_def: PageDefinition
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

    async def render_in_template(
        self, *, page: RenderedPage, template_name: str
    ) -> str:
        """
        Render the template asynchronously.

        It assumes that in the page.data there is already all the data rendered.

        It is like rendering a page, but the previour content is set at the "children" elements.

        Template names use a simplified syntax: type://resource.

        Where types are the normal types around the app:
         - http
         - page

        From it it composes a new definition and sets the viewer as the full string.
        """
        template_def = self.loader.load(template_name)
        page.context = {
            **page.context,
            "children": page.content,
        }
        page.content = ""
        await self.render_page_data(page=page, page_def=template_def)
        if template_def.template:
            await self.render_in_template(
                page=page, template_name=template_def.template
            )

    async def render_block(
        self, page: RenderedPage, block: BlockDefinition
    ) -> tuple[str, dict[str, str]]:
        """
        Render a block asynchronously.
        """
        element_renderer = self.get_block_renderer(block.type)

        html = await element_renderer.render_html(data=block.data, context=page.context)
        css = await element_renderer.render_css(data=block.data, context=page.context)

        return html, css

    @lru_cache(maxsize=100)
    def get_block_renderer(self, type_name: str) -> BlockRendererBase:
        """
        Get a block renderer.
        """
        block = self.config.elements[type_name]

        if block.type == "builtin":
            from pe.renderer.builtin import ElementRendererBuiltin

            return ElementRendererBuiltin(block=block)
        elif block.type == "http":
            from pe.renderer.http import ElementRendererHttp

            return ElementRendererHttp(block=block)

        raise ValueError(f"Unknown block renderer type: {type_name}, {block.viewer}")


def css_dict_to_cs_text(css: dict[str, str]) -> str:
    """
    Convert a dict of CSS to a string.
    """
    return "\n".join([f"  {k}: {v};" for k, v in css.items()])
