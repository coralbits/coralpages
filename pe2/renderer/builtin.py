import re
from typing import Any

import jinja2
import markdown

from pe2.renderer.types import ElementRendererBase
from pe2.types import BlockDefinition


jinja_env = jinja2.Environment(loader=jinja2.FileSystemLoader("templates"))
jinja_env.filters["markdown"] = markdown.markdown


class ElementRendererBuiltin(ElementRendererBase):
    """
    Renderer for a built-in element.
    """

    def __init__(self, *, block: BlockDefinition):
        """
        Initialize the renderer.

        :param block: The block to render.
        """
        super().__init__(block=block)

    async def render_html(
        self, *, data: dict[str, Any], context: dict[str, Any]
    ) -> str:
        """
        Render a built-in element asynchronously using jinja2.

        The component is defined in the config as "builtin://templates/block/block.html"
        """
        template = jinja_env.get_template(self.block.viewer[10:])
        return template.render(
            block=self.block,
            data=data,
            context=context,
        )

    async def render_css(self, *, data: dict[str, Any], context: dict[str, Any]) -> str:
        """
        Render the CSS for a built-in element asynchronously.
        """
        template = jinja_env.get_template(self.block.css[10:])
        css = template.render(
            block=self.block,
            data=data,
            context=context,
        )
        return css
