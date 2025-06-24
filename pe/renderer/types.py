from typing import Any
from pe.types import ElementDefinition
from pe.loader.factory import LoaderRoot


class BlockRendererBase:
    """
    Renderer for a block.

    Specific renderers have other implementations.
    """

    def __init__(self, *, element: ElementDefinition, loader: LoaderRoot):
        self.element = element
        self.loader = loader

    async def render_html(
        self, *, data: dict[str, Any], context: dict[str, Any]
    ) -> str:
        """
        Render a block.
        """
        raise NotImplementedError("Not implemented")

    async def render_css(self, *, data: dict[str, Any], context: dict[str, Any]) -> str:
        """
        Render the CSS for a block.
        """
        raise NotImplementedError("Not implemented")
