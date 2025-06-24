"""
HTTP store implementation.
"""

from typing import Any
from urllib.parse import urlencode
import httpx

from pe.stores.types import StoreBase
from pe.types import PageDefinition, ElementDefinition, StoreConfig


class HttpStore(StoreBase):
    """
    HTTP-based store implementation.
    """

    def __init__(self, config: StoreConfig):
        super().__init__(config)
        self.base_url = config.base_url or ""

    def _build_url(self, path: str) -> str:
        """
        Build the full URL for a path.
        """
        if path.startswith("http"):
            return path
        return f"{self.base_url.rstrip('/')}/{path.lstrip('/')}"

    async def load_html(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a page from the HTTP store.
        """
        url = self._build_url(f"{path}.yaml")
        return await self.load_generic(path=url, data=data, context=context)

    async def load_css(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a CSS file from the HTTP store.
        """
        url = self._build_url(path)
        return await self.load_generic(path=url, data=data, context=context)

    async def load_generic(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a generic file from the HTTP store.
        """
        url = self._build_url(path)
        if "post:json" in self.config.tags:
            data = {"data": data, "context": context}
            response = await httpx.AsyncClient().post(url, json=data)
        elif "get:qs" in self.config.tags:
            qs = urlencode({**data, **context})
            response = await httpx.AsyncClient().get(url, params=qs)
        else:
            response = await httpx.AsyncClient().get(url)
        if response.status_code == 200:
            return response.text
        return None
