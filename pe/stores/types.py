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
