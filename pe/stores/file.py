"""
File store implementation.
"""

import logging
from pathlib import Path
from typing import Any

import yaml

from pe.types import BlockDefinition, ElementDefinition, PageDefinition, StoreConfig
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
        if path == "html":
            return data.get("html", "")

        element_definition = await self.get_element_definition(path)
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
        element_definition = await self.get_element_definition(path)
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

        if path.endswith(".html"):
            return await self.load_html_definition(path=path)

        path = f"{path}.yaml"

        yamldata = await self.load_generic(path=path, data={}, context={})
        if not yamldata:
            return None
        return PageDefinition.from_dict(yaml.safe_load(yamldata))

    async def load_html_definition(self, *, path: str) -> PageDefinition | None:
        """
        Load an HTML page definition from the file store.
        """
        filepath = self.base_path / path
        if not filepath.exists():
            return None

        with open(filepath, "r", encoding="utf-8") as file:
            html = file.read()

        return PageDefinition(
            data=[BlockDefinition(type="builtin://html", data={"html": html})]
        )

    async def load_generic(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]  # type: ignore
    ) -> str | None:
        """
        Load raw content from the file store.

        data is ignored, to be rendered by the renderer.
        """
        filepath = self.base_path / path
        if not filepath.exists():
            return None

        with open(filepath, "r", encoding="utf-8") as file:
            return file.read()

    async def get_element_list(self) -> list[ElementDefinition]:
        """
        Get a list of all elements in the file store.
        """
        return self.config.blocks
