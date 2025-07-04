import logging
from typing import Any

from pe.types import BlockTemplate, Page, StoreConfig


logger = logging.getLogger(__name__)


class StoreBase:
    """
    Base class for all stores.
    """

    config: StoreConfig
    tags: list[str]

    def __init__(self, config: StoreConfig):
        self.config = config
        self.tags = config.tags

    async def load_html(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a page from the store.
        """
        raise NotImplementedError("load_html not implemented")

    async def load_css(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a CSS file from the store.
        """
        raise NotImplementedError("load_css not implemented")

    async def load_context(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> dict[str, Any] | None:
        """
        Load the context for the store. By default do nothing, keep current context as is.
        """
        return None

    async def load_page_definition(self, *, path: str) -> Page | None:
        """
        Load a page from the store.
        """
        raise NotImplementedError("load_page_definition not implemented")

    async def save_page_definition(self, *, path: str, data: Page) -> None:
        """
        Save a page to the store.
        """
        raise NotImplementedError("save_page_definition not implemented")

    async def get_element_list(self) -> list[BlockTemplate]:
        """
        Get a list of all elements in the store.
        """
        logger.debug("Getting element list for store: %s", self.config.name)
        return []

    async def get_element_definition(self, path: str) -> BlockTemplate | None:
        """
        Get an element definition from the store.
        """
        for block in await self.get_element_list():
            if block.name == path:
                return block

        raise ValueError(
            f"Element definition not found for path: {path}. Available elements: {list(block.name for block in await self.get_element_list())}"
        )

    def clean_path(self, path: str) -> str:
        """
        Clean a path.

        Removes the protocol part of the path, if its the same as the store's name.
        """
        store_prefix = f"{self.config.name}://"
        if path.startswith(store_prefix):
            path = path[len(store_prefix) :]

        return path
