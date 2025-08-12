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
from pe.types import Element, MetaDefinition, Page

logger = logging.getLogger(__name__)


@dataclass
class PageError:
    """
    A page error.
    """

    error: Exception
    block: Element


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
    errors: list[Exception] = field(default_factory=list)

    def append_content(self, content: str):
        """
        Append content to the page. Also increments the max_id.
        """
        self.content += content

    def increment_id(self):
        """
        Increment the max_id.
        """
        self.max_id += 1

    def append_meta(self, meta: MetaDefinition):
        """
        Append a meta definition to the page.
        """
        self.meta.append(meta)

    def update_from_definition(self, page_def: Page):
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

    def get_current_id(self, block: Element) -> int:
        """
        Get the current id.
        """
        if block.id:
            id = block.id
        else:
            prefix = block.type

            prefix = prefix.replace("://", "-")
            prefix = prefix.replace(":", "-")
            prefix = prefix.replace("/", "-")
            id = f"{prefix}-{self.max_id}"

        if id[0].isdigit():
            id = f"block_{id}"
        return id

    def __str__(self):
        """
        Render the page as a string.
        """
        if self.response_code == 304:
            return ""

        return self.content

    def has_errors(self) -> bool:
        """
        Check if the page has errors.
        """
        return len(self.errors) > 0

    def add_error(self, *, error: Exception, block: Element):
        """
        Add an error to the page.
        """
        self.errors.append(PageError(error=error, block=block))

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the page to a dictionary.
        """
        return {
            "title": self.title,
            "body": self.get_body(),
            "head": {
                "css": self.get_css(),
                "js": self.get_js(),
                "meta": self.get_meta(),
            },
            "http": {
                "headers": self.get_headers(),
                "response_code": self.response_code,
            },
        }

    def get_body(self) -> str:
        """
        Get the body of the page.
        """
        return self.content

    def get_headers(self) -> dict[str, str]:
        return {}

    def get_meta(self) -> list[dict[str, str]]:
        return [meta.to_dict() for meta in self.meta]

    def get_css(self) -> str:
        """
        Get the CSS for the page.
        """
        vars = self.css_variables
        ret = ":root {\n"
        ret += "\n".join(f"{key}: {value};" for key, value in vars.items())
        ret += "\n}\n\n"

        ret += "\n\n".join(self.classes.values())
        return ret

    def get_js(self) -> str:
        """
        Get the JavaScript for the page.
        """
        return ""


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
        self.jinja2_env.filters["markdown"] = lambda x: markdown.markdown(
            x,
            extensions=[
                "markdown.extensions.fenced_code",
                "markdown.extensions.tables",
            ],
        )
        self.jinja2_env.filters["json"] = lambda x: json.dumps(x, indent=2, default=str)

    def new_page(self) -> RenderedPage:
        """
        Create a new page.
        """
        return RenderedPage(title="")

    async def render_page(
        self,
        *,
        store: str,
        path: str,
        headers: dict[str, str] = {},
        template: str | None = None,
    ) -> RenderedPage | None:
        """
        Render a page asynchronously.

        Might use headers to check caches and so on
        """
        page_definition = None
        for storei in store.split("|"):
            store_obj = self.store.get_store(storei)
            page_definition = store_obj and await store_obj.load_page_definition(
                path=path
            )
            if page_definition:
                # logger.debug("Rendering page: %s/%s", storei, path)
                break
        if not page_definition:
            logger.warning("Page definition not found: %s/%s", store, path)
            return None

        if template == "none":
            page_definition.template = None
        elif template is not None:
            page_definition.template = template

        logger.debug("Set template to %s", page_definition.template)

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

        # logger.debug("Rendering page")
        page = await self.render(page_definition)
        logger.debug(
            "Page rendered block_count=%s, page_size=%s",
            len(page_definition.children),
            len(page.content),
        )

        if new_etag:
            page.headers["ETag"] = new_etag
        if new_last_modified:
            page.headers["Last-Modified"] = new_last_modified

        # logger.debug("Returning page")
        return page

    def calculate_last_modified(self, page_definition: Page) -> str:
        """
        Calculate the last modified for a page definition.

        Looks at the last modified data of the page definition and the blocks.
        """
        return (
            page_definition.last_modified.isoformat()
            if page_definition.last_modified
            else datetime.datetime.now().isoformat()
        )

    def calculate_etag(self, page_definition: Page) -> str:
        """
        Calculate the etag for a page definition.
        """
        salt = datetime.datetime.now().strftime(self.config.server.etag_salt)

        return hashlib.sha256(
            json.dumps(page_definition.to_dict(), sort_keys=True).encode()
            + salt.encode()
        ).hexdigest()

    async def render(self, page: Page, *, context: dict | None = None) -> RenderedPage:
        """
        Render a page asynchronously.
        """
        rendered_page = self.new_page()
        rendered_page.update_from_definition(page)
        rendered_page.context = context or {}

        # logger.debug("Rendering page data")
        await self.render_page_data(page=rendered_page, page_def=page)
        if page.template:
            logger.debug("Rendering in template %s", page.template)
            await self.render_in_template(
                page=rendered_page, template_name=page.template
            )

        return rendered_page

    async def render_page_data(self, *, page: RenderedPage, page_def: Page):
        """
        Render the page data asynchronously.
        """
        for meta in page_def.meta:
            page.append_meta(meta)

        logger.debug(
            "Rendering page=%s, block_count=%d blocks",
            page_def.path,
            len(page_def.children),
        )
        for block in page_def.children:
            # logger.debug("Rendering block: %s", block.type)
            try:
                html = await self.render_block(page, block, context=page.context)
            except Exception as e:  # pylint: disable=broad-exception-caught
                page.add_error(error=e, block=block)
                logger.exception("Error rendering block: %s", block)
                if self.config.debug:
                    html = f"<div style='color: red;'>Error rendering block: {block.type}<pre>{html_safe(traceback.format_exc())}</pre></div>"
                else:
                    raise

            page.append_content(html)
            page.increment_id()

    async def render_in_template(self, *, page: RenderedPage, template_name: str):
        """
        Render the template asynchronously.

        It assumes that in the page.data there is already all the data rendered.

        It is like rendering a page, but the previour content is set at the "children" elements.

        Template names use a simplified syntax: type/resource.

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
        # logger.debug("Rendering template data")
        await self.render_page_data(page=page, page_def=template_def)
        if template_def.template:
            logger.debug("Rendering in template %s", template_def.template)
            await self.render_in_template(
                page=page, template_name=template_def.template
            )

    async def render_block(
        self, page: RenderedPage, block: Element, context: dict[str, Any]
    ) -> str:
        """
        Render a block asynchronously.
        """
        store, eltype = block.type.split("/", 1)
        element_loader = self.store.get_store(store)
        if element_loader is None:
            raise ValueError(f"Element loader not found for block: {block.type}")
        block_id = page.get_current_id(block)
        block.data["id"] = block_id

        new_context = await element_loader.load_context(
            path=eltype, data=block.data, context=context
        )
        if new_context:
            context = {
                **context,
                **new_context,
            }

        children = StrList()
        for child in block.children or []:
            try:
                children_data = await self.render_block(page, child, context=context)
                # logger.debug("Rendered child: %s", children_data)
                children.append(children_data)
                page.increment_id()
            except Exception as e:  # pylint: disable=broad-exception-caught
                page.add_error(error=e, block=child)
                logger.exception("Error rendering child: %s", child)
                if self.config.debug:
                    children_data = f"<div style='color: red;'>Error rendering child: {child.type}<pre>{html_safe(traceback.format_exc())}</pre></div>"
                else:
                    raise

        html = (
            await element_loader.load_html(
                path=eltype, data=block.data, context=context
            )
            or ""
        )
        css = (
            await element_loader.load_css(path=eltype, data=block.data, context=context)
            or ""
        )

        if "jinja2" in element_loader.tags:
            data = self.jinja2_render_dict(block.data, page=page, context=context)

            html = self.jinja2_render(
                html,
                data=data,
                context=context,
                page=page,
                children=children,
            )
            css = self.jinja2_render(
                css,
                data=data,
                context=context,
                page=page,
            )
        elif "@@children@@" in html:
            html = html.replace("@@children@@", "\n".join(children))
            # logger.debug("Rendered block: %s", html)

        # logger.debug(
        #     "Rendered block=%s, data=%s, context=%s, children=%s, html=%s, element_loader=%s",
        #     block.type,
        #     block.data,
        #     context,
        #     children,
        #     html,
        #     element_loader,
        # )

        if block.style:
            css_id = f"#{block_id}"
            page.classes.update(
                {css_id: f"{css_id} {{\n{css_dict_to_css_text(block.style)} \n}}"}
            )
        html = html.replace("@@class@@", block_id)
        html = html.replace("@@id@@", block_id)

        # add the css to the page and content. The idea is to avoid repetition of the same css.
        # This is a very simple approach, but it works.
        css_md5 = hashlib.md5(
            css.encode()
        ).hexdigest()  # md5 is not crypto secure, but much faster, and very low risk of collission
        page.classes.update({css_md5: css})

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
