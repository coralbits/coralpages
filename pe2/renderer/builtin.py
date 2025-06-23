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

    def render(self, *, data: dict[str, Any], context: dict[str, Any]) -> str:
        """
        Render a built-in element. using jinja2.

        The component is defined in the config as "builtin:templates/block/block.html"
        """
        template = jinja_env.get_template(self.block.viewer[10:])
        return template.render(
            block=self.block,
            data=data,
            context=context,
        )
