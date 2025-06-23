from typing import Any
from pe2.types import BlockDefinition, ElementDefinition


class ElementRendererBase:
    """
    Renderer for an element.

    Specific renderers have other implementations.
    """

    def __init__(self, *, block: BlockDefinition):
        self.block = block

    def render_html(self, *, data: dict[str, Any], context: dict[str, Any]) -> str:
        """
        Render an element.
        """
        raise NotImplementedError("Not implemented")

    def render_css(self, *, data: dict[str, Any], context: dict[str, Any]) -> str:
        """
        Render the CSS for an element.
        """
        raise NotImplementedError("Not implemented")
