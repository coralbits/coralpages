import datetime
import logging
from dataclasses import dataclass, field
import hashlib
import json
import traceback
from typing import Any

import jinja2
import markdown

from pe.config import Config
from pe.stores.factory import StoreFactory
from pe.types import BlockDefinition, MetaDefinition, PageDefinition

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
    meta: list[MetaDefinition] = field(default_factory=list)
    css_variables: dict[str, str] = field(default_factory=dict)

    def append_content(self, content: str):
        """
        Append content to the page. Also increments the max_id.
        """
        self.content += content
        self.max_id += 1

    def append_meta(self, meta: MetaDefinition):
        """
        Append a meta definition to the page.
        """
        self.meta.append(meta)

    def update_from_definition(self, page_def: PageDefinition):
        """
        Update the page from a page definition.
        """
        if not self.title:
            self.title = page_def.title

        self.meta = [*page_def.meta]
        self.css_variables = {
            **self.css_variables,
            **page_def.css_variables,
        }

    def get_current_id(self, block: BlockDefinition) -> int:
        """
        Get the current id.
        """
        if block.id:
            return block.id

        prefix = block.type

        prefix = prefix.replace("://", "-")
        prefix = prefix.replace(":", "-")
        prefix = prefix.replace("/", "-")
        return f"{prefix}-{self.max_id}"

    def __str__(self):
        """
        Render the page as a string.
        """
        if self.response_code == 304:
            return ""

        return self.content


class Renderer:
    """
    Renderer for the page editor.
    """

    def __init__(self, config: Config, store: StoreFactory | None = None):
        """
        Initialize the renderer.
        """
        self.config = config
        if store:
            self.store = store
        else:
            self.store = StoreFactory(config=config)
        self.jinja2_env = jinja2.Environment()
        self.jinja2_env.filters["markdown"] = markdown.markdown

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
        logger.debug("Rendering page: %s", page_name)
        page_definition = await self.store.load_page_definition_all_stores(
            path=page_name
        )
        if not page_definition:
            raise ValueError(f"Page definition not found: {page_name}")

        logger.debug("Page definition: %s", page_definition)
        new_etag = None
        new_last_modified = None
        if "etag" in page_definition.cache:
            logger.debug("Checking etag")
            old_etag = headers.get("If-None-Match")
            new_etag = self.calculate_etag(page_definition)
            logger.debug("Old etag: %s, new etag: %s", old_etag, new_etag)
            if old_etag == new_etag:
                page = self.new_page()
                page.headers["ETag"] = old_etag
                page.response_code = 304
                logger.debug("Page not modified, returning 304")
                return page
        if "last-modified" in page_definition.cache:
            logger.debug("Checking last modified")
            old_last_modified = headers.get("If-Modified-Since")
            new_last_modified = self.calculate_last_modified(page_definition)
            if old_last_modified == new_last_modified:
                page = self.new_page()
                page.headers["Last-Modified"] = new_last_modified
                page.response_code = 304
                logger.debug("Page not modified, returning 304")
                return page

        logger.debug("Rendering page")
        page = await self.render(page_definition)
        logger.debug("Page rendered")

        if new_etag:
            page.headers["ETag"] = new_etag
        if new_last_modified:
            page.headers["Last-Modified"] = new_last_modified

        logger.debug("Returning page")
        return page

    def calculate_last_modified(self, page_definition: PageDefinition) -> str:
        """
        Calculate the last modified for a page definition.

        Looks at the last modified data of the page definition and the blocks.
        """
        return (
            page_definition.last_modified.isoformat()
            if page_definition.last_modified
            else datetime.datetime.now().isoformat()
        )

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
        page.update_from_definition(page_def)

        logger.debug("Rendering page data")
        await self.render_page_data(page=page, page_def=page_def)
        if page_def.template:
            logger.debug("Rendering in template %s", page_def.template)
            await self.render_in_template(page=page, template_name=page_def.template)

        return page

    async def render_page_data(
        self, *, page: RenderedPage, page_def: PageDefinition
    ) -> str:
        """
        Render the page data asynchronously.
        """
        for meta in page_def.meta:
            page.append_meta(meta)

        logger.debug("Rendering page, %d blocks", len(page_def.data))
        for block in page_def.data:
            logger.debug("Rendering block: %s", block)
            try:
                html = await self.render_block(page, block)
                logger.debug("Block rendered: %s", block)
            except Exception as e:
                logger.error("Error rendering block: %s", block)
                logger.error("Error: %s", e)
                if self.config.debug:
                    html = f"<div style='color: red;'>Error rendering block: {block.type}<pre>{html_safe(traceback.format_exc())}</pre></div>"
                else:
                    raise

            page.append_content(html)
            logger.debug("Block appended: %s", block)

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
        template_def = await self.store.get_store(template_name).load_page_definition(
            path=template_name
        )
        if not template_def:
            raise ValueError(f"Template not found: {template_name}")
        page.update_from_definition(template_def)

        page.context = {
            **page.context,
            "children": page.content,
        }
        page.content = ""
        logger.debug("Rendering template data")
        await self.render_page_data(page=page, page_def=template_def)
        if template_def.template:
            logger.debug("Rendering in template %s", template_def.template)
            await self.render_in_template(
                page=page, template_name=template_def.template
            )

    async def render_block(self, page: RenderedPage, block: BlockDefinition) -> str:
        """
        Render a block asynchronously.
        """
        element_loader = self.store.get_store(block.type)
        if element_loader is None:
            raise ValueError(f"Element loader not found for block: {block.type}")

        logger.debug("Loader for element: %s", element_loader)
        logger.debug("Loading HTML for block: %s", block)
        html = (
            await element_loader.load_html(
                path=block.type, data=block.data, context=page.context
            )
            or ""
        )
        logger.debug("Loading CSS for block: %s", block)
        css = (
            await element_loader.load_css(
                path=block.type, data=block.data, context=page.context
            )
            or ""
        )

        children = StrList()
        for child in block.children or []:
            try:
                children_data = await self.render_block(page, child)
                children.append(children_data)
            except Exception as e:
                logger.error("Error rendering child: %s", child)
                logger.error("Error: %s", e)
                if self.config.debug:
                    children_data = f"<div style='color: red;'>Error rendering child: {child.type}<pre>{html_safe(traceback.format_exc())}</pre></div>"
                else:
                    raise

        if "jinja2" in element_loader.tags:
            logger.debug("Rendering Jinja2 for block: %s", block)

            data = self.jinja2_render_dict(block.data, page=page, context=page.context)

            html = self.jinja2_render(
                html,
                data=data,
                context=page.context,
                page=page,
                children=children,
            )
            css = self.jinja2_render(
                css,
                data=data,
                context=page.context,
                page=page,
            )
        elif "@@children@@" in html:
            html = html.replace("@@children@@", "\n".join(children))

        block_id = page.get_current_id(block)

        if block.style:
            css_id = f"#{block_id}"
            page.classes.update(
                {css_id: f"{css_id} {{\n{css_dict_to_css_text(block.style)} \n}}"}
            )
        html = html.replace("@@class@@", block_id)
        html = html.replace("@@id@@", block_id)

        # add the css to the page and content
        page.classes.update({block.type: css})

        return html

    def jinja2_render(
        self,
        html: str,
        **context: Any,
    ) -> str:
        """
        Render a Jinja2 template.
        """
        template = self.jinja2_env.from_string(html)
        return template.render(**context)

    def jinja2_render_dict(
        self, data: dict[str, Any], **context: Any
    ) -> dict[str, Any]:
        """
        Render a Jinja2 template.
        """
        ret = {}
        for key, value in data.items():
            if isinstance(value, str):
                ret[key] = self.jinja2_render(value, **context)
            elif isinstance(value, dict):
                ret[key] = self.jinja2_render_dict(value, **context)
            elif isinstance(value, list):
                ret[key] = [self.jinja2_render_dict(item, **context) for item in value]
            else:
                ret[key] = value
        return ret


def css_dict_to_css_text(css: dict[str, str]) -> str:
    """
    Convert a dict of CSS to a string.
    """
    return "\n".join([f"  {k}: {v};" for k, v in css.items()])


class StrList(list[str]):
    """
    A list of strings.
    """

    def __str__(self) -> str:
        return "\n".join([str(x) for x in self])


def html_safe(text: str) -> str:
    """
    Escape HTML.
    """
    return text.replace("<", "&lt;").replace(">", "&gt;")
