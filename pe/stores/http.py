"""
HTTP store implementation.
"""

import logging
from typing import Any
from urllib.parse import urlencode
import httpx

from pe.stores.types import StoreBase
from pe.types import StoreConfig

logger = logging.getLogger(__name__)


class HttpStore(StoreBase):
    """
    HTTP-based store implementation.
    """

    def __init__(self, config: StoreConfig):
        super().__init__(config)
        self.base_url = config.base_url or ""

    def _build_url(self, path: str, type: str = "") -> str:
        """
        Build the full URL for a path.
        """
        path = self.clean_path(path)

        if self.base_url:
            if path.startswith("http://") or path.startswith("https://"):
                raise ValueError(
                    f"Can not use bare URLs ({path}), they have to be relative to {self.base_url}"
                )

            path = self.base_url.rstrip("/") + "/" + path.lstrip("/")

        if type:
            path = f"{path}/{type}"

        return path

    async def load_html(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a page from the HTTP store.
        """
        url = self._build_url(path, "html")
        return await self.load_generic(url=url, data=data, context=context)

    async def load_css(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a CSS file from the HTTP store.
        """
        url = self._build_url(path, "css")
        return await self.load_generic(url=url, data=data, context=context)

    async def load_generic(
        self, *, url: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a generic file from the HTTP store.
        """
        logger.debug("Loading %s", url)

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
