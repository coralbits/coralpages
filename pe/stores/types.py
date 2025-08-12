import logging
from typing import Any, List

from pe.types import Widget, Page, PageInfo, PageListResult, StoreConfig


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

    def __repr__(self) -> str:
        return f"<{self.__class__.__name__} name={self.config.name} tags={self.tags}>"

    async def load_html(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a page from the store.
        """
        raise NotImplementedError(
            f"load_html not implemented in {self.__class__.__name__}"
        )

    async def load_css(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a CSS file from the store.
        """
        raise NotImplementedError(
            f"load_css not implemented in {self.__class__.__name__}"
        )

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
        raise NotImplementedError(
            f"load_page_definition not implemented in {self.__class__.__name__}"
        )

    async def save_page_definition(self, *, path: str, data: Page) -> None:
        """
        Save a page to the store.
        """
        raise NotImplementedError(
            f"save_page_definition not implemented in {self.__class__.__name__}"
        )

    async def get_widget_list(self) -> list[Widget]:
        """
        Get a list of all widgets in the store.
        """
        logger.warning(
            "Getting widget list for store=%s. This list is not implemented!",
            self.config.name,
        )
        return []

    async def get_widget_definition(self, path: str) -> Widget | None:
        """
        Get an widget definition from the store.
        """
        for block in await self.get_widget_list():
            logger.debug("Comparing path=%s with block=%s", path, block.name)
            if block.name == path:
                logger.debug("Found widget definition for path=%s", path)
                return block

        raise ValueError(
            f"Element definition not found for path: {path}. Available widgets: {list(block.name for block in await self.get_widget_list())}"
        )

    def clean_path(self, path: str) -> str:
        """
        Clean a path.

        Removes the protocol part of the path, if its the same as the store's name.
        """
        store_prefix = f"{self.config.name}/"
        if path.startswith(store_prefix):
            path = path[len(store_prefix) :]

        return path

    async def get_page_list(
        self, *, offset: int = 0, limit: int = 10, filter: dict | None = None
    ) -> PageListResult:
        """
        Get a list of all pages.
        """
        return PageListResult(count=0, results=[])

    async def delete_page_definition(
        self,
        path: str,
    ) -> bool:
        """
        Delete a page definition from the store.
        """
        raise NotImplementedError(
            f"delete_page_definition not implemented in {self.__class__.__name__}"
        )

    def is_writable(self) -> bool:
        """
        Check if the store is writable.
        """
        return False
