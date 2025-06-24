"""
File store implementation.
"""

import logging
from pathlib import Path
from typing import Any

import yaml

from pe.types import PageDefinition, StoreConfig
from pe.stores.types import StoreBase

logger = logging.getLogger(__name__)


class FileStore(StoreBase):
    """
    File-based store implementation.
    """

    def __init__(self, config: StoreConfig):
        super().__init__(config)
        self.base_path = Path(config.path) if config.path else Path(".")

    async def load_html(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a page from the file store.
        """
        element_definition = self.get_element_definition(path)
        if element_definition is None:
            return None

        if not element_definition.html:
            return None

        return await self.load_generic(
            path=element_definition.html, data=data, context=context
        )

    async def load_css(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load an element from the file store.

        CSS is plain CSS data
        """
        element_definition = self.get_element_definition(path)
        if element_definition is None:
            return None

        if not element_definition.css:
            return None

        return await self.load_generic(
            path=element_definition.css, data=data, context=context
        )

    async def load_page_definition(self, *, path: str) -> PageDefinition | None:
        """
        Load a page definition from the file store.
        """
        if "://" in path:
            path = path.split("://", 1)[1]

        path = f"{path}.yaml"

        yamldata = await self.load_generic(path=path, data={}, context={})
        if not yamldata:
            return None
        return PageDefinition.from_dict(yaml.safe_load(yamldata))

    async def load_generic(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]  # type: ignore
    ) -> str | None:
        """
        Load raw content from the file store.

        data is ignored, to be rendered by the renderer.
        """
        filepath = self.base_path / path
        logger.debug("Loading generic from: %s", filepath)
        if not filepath.exists():
            logger.debug("File not found: %s", filepath)
            return None

        with open(filepath, "r", encoding="utf-8") as file:
            return file.read()
