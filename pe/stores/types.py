import logging
from typing import Any

from pe.types import ElementDefinition, PageDefinition, StoreConfig


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

    async def load_page_definition(self, *, path: str) -> PageDefinition | None:
        """
        Load a page from the store.
        """
        raise NotImplementedError("load_page_definition not implemented")

    async def save_page_definition(self, *, path: str, data: PageDefinition) -> None:
        """
        Save a page to the store.
        """
        raise NotImplementedError("save_page_definition not implemented")

    def get_element_list(self) -> list[ElementDefinition]:
        """
        Get a list of all elements in the store.
        """
        logger.debug("Getting element list for store: %s", self.config.name)
        return []

    def get_element_definition(self, path: str) -> ElementDefinition | None:
        """
        Get an element definition from the store.
        """
        if "://" in path:
            path = path.split("://", 1)[1]

        for block in self.config.blocks:
            if block.name == path:
                logger.debug("Found element definition: %s", block)
                return block

        logger.debug("No element definition found for: %s", path)
        return None

    def clean_path(self, path: str) -> str:
        """
        Clean a path.

        Removes the protocol part of the path, if its the same as the store's name.
        """
        store_prefix = f"{self.config.name}://"
        if path.startswith(store_prefix):
            path = path[len(store_prefix) :]

        return path
