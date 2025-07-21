"""
File store implementation.
"""

import logging
from pathlib import Path
from typing import Any

import yaml

from pe.types import Element, Widget, Page, PageInfo, PageListResult, StoreConfig
from pe.stores.types import StoreBase

logger = logging.getLogger(__name__)


class FileStore(StoreBase):
    """
    File-based store implementation.
    """

    def __init__(self, config: StoreConfig):
        super().__init__(config)
        self.base_path = Path(config.path) if config.path else Path(".")
        self.widgets = {}
        self.load_widgets()

    def load_widgets(self) -> dict[str, Widget]:
        """
        Load the blocks from the file store.
        """
        if "widgets" not in self.config.tags:
            logger.debug(
                "No widgets tag in config.yaml for store=%s, skipping load widgets",
                self.config.name,
            )
            return {}
        with open(self.base_path / "config.yaml", "r", encoding="utf-8") as fd:
            yamlconfig = yaml.safe_load(fd)

        block_data = yamlconfig.get("widgets", [])
        if not block_data:
            logger.warning("No widgets found in file store from path=%s", self.base_path)
            return {}

        widgets = [Widget.from_dict(x) for x in block_data]
        self.widgets = {x.name: x for x in widgets}
        if len(self.widgets) == 0:
            logger.warning("No widgets found in file store from path=%s", self.base_path)
        logger.debug(
            "Loaded count=%d widgets from file store from path=%s",
            len(self.widgets),
            self.base_path,
        )
        return self.widgets

    async def load_html(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a page from the file store.
        """
        if path == "html":  # html is special internal for top level html
            return data.get("html", "")

        widget = await self.get_widget_definition(path)
        if widget is None:
            raise ValueError(
                f"Element definition not found for path={path}, known elements={list(self.widgets.keys())}"
            )

        if not widget.html:
            return None

        return await self.load_generic(
            path=widget.html, data=data, context=context
        )

    async def load_css(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load an widget from the file store.

        CSS is plain CSS data
        """
        widget_definition = await self.get_widget_definition(path)
        if widget_definition is None:
            return None

        if not widget_definition.css:
            return None

        return await self.load_generic(
            path=widget_definition.css, data=data, context=context
        )

    async def load_page_definition(self, *, path: str) -> Page | None:
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
        return Page.from_dict(yaml.safe_load(yamldata))

    async def load_html_definition(self, *, path: str) -> Page | None:
        """
        Load an HTML page definition from the file store.
        """
        filepath = self.base_path / path
        if not filepath.exists():
            return None

        with open(filepath, "r", encoding="utf-8") as file:
            html = file.read()

        return Page(children=[Element(type="builtin://html", data={"html": html})])

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

    async def get_widget_list(self) -> list[Widget]:
        """
        Get a list of all widgets in the file store.
        """
        logger.info("Get widget list from file store count=%s", len(self.widgets))
        return list(self.widgets.values())

    async def get_widget_definition(self, path: str) -> Widget | None:
        """
        Get an widget definition from the file store.
        """
        return self.widgets.get(path)

    async def get_page_list(
        self, *, offset: int = 0, limit: int = 10
    ) -> PageListResult:
        """
        Get a list of all pages.
        """
        if "pages" not in self.config.tags:
            return PageListResult(count=0, results=[])

        count = 0
        results = []
        logger.debug("Loading pages from file store from path=%s", self.base_path)
        for path in self.base_path.glob("**.yaml"):
            if path.is_file():
                name = self.clean_name(path)

                count += 1
                # just count, no read the file
                if limit == 0 or count < offset:
                    continue

                limit -= 1
                # read the file data
                with open(path, "r", encoding="utf-8") as file:
                    page_data = yaml.safe_load(file)

                    results.append(PageInfo(id=name, title=page_data["title"], url=""))

        return PageListResult(count=count, results=results)

    def clean_name(self, path: Path) -> str:
        """
        Clean a name from the file store.
        """
        base_path_str = str(self.base_path)
        name = str(path)
        if name.startswith(base_path_str):
            name = name[len(base_path_str) :]
        while name.startswith("/"):
            name = name[1:]
        if name.endswith(".yaml"):
            name = name[:-5]
        if name.endswith(".html"):
            name = name[:-5]
        if name.endswith(".md"):
            name = name[:-3]

        return name
