from typing import Any
from pe.types import BlockDefinition


class BlockRendererBase:
    """
    Renderer for a block.

    Specific renderers have other implementations.
    """

    def __init__(self, *, block: BlockDefinition):
        self.block = block

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
