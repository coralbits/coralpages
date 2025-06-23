from typing import Any
from pe.types import BlockDefinition


class ElementRendererBase:
    """
    Renderer for an element.

    Specific renderers have other implementations.
    """

    def __init__(self, *, block: BlockDefinition):
        self.block = block

    async def render_html(
        self, *, data: dict[str, Any], context: dict[str, Any]
    ) -> str:
        """
        Render an element asynchronously.
        """
        raise NotImplementedError("Not implemented")

    async def render_css(self, *, data: dict[str, Any], context: dict[str, Any]) -> str:
        """
        Render the CSS for an element asynchronously.
        """
        raise NotImplementedError("Not implemented")
