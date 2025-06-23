from typing import Any
import httpx
from pe.renderer.types import BlockRendererBase


class ElementRendererHttp(BlockRendererBase):
    """
    Renderer for HTTP elements.
    """

    async def render_html(
        self, *, data: dict[str, Any], context: dict[str, Any]
    ) -> str:
        """
        Render the HTML for an HTTP element (async version).

        Grab the content from the URL and return it.

        The URL is the viewer of the element.
        """
        url = self.block.viewer
        async with httpx.AsyncClient() as client:
            response = await client.get(url, params=data)
            response.raise_for_status()
            return response.text

    async def render_css(self, *, data: dict[str, Any], context: dict[str, Any]) -> str:
        """
        Render the CSS for an HTTP element (async version).
        """
        return ""
