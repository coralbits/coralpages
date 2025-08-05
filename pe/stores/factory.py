"""
Store factory for creating store instances.
"""

import logging
from typing import Dict

from pe.config import Config
from pe.stores.types import StoreBase
from pe.stores.file import FileStore
from pe.stores.http import HttpStore
from pe.stores.db import DbStore
from pe.types import Page, PageListResult, StoreConfig
import json

logger = logging.getLogger(__name__)


class StoreFactory:
    """
    Factory for creating store instances.
    """

    def __init__(self, config: Config):
        self.config = config
        self._stores: dict[str, StoreBase] = {
            store_name: self.create_store(store_config)
            for store_name, store_config in self.config.stores.items()
        }

    def create_store(self, store_config: StoreConfig) -> StoreBase:
        """
        Create a store instance by name.
        """
        if store_config.type == "file":
            return FileStore(store_config)
        elif store_config.type == "http":
            return HttpStore(store_config)
        elif store_config.type == "db":
            return DbStore(store_config)
        else:
            raise ValueError(f"Unknown store type: {store_config.type}")

    def get_store(self, store_name: str) -> StoreBase:
        """
        Get a store instance by name.
        """
        if "/" in store_name:
            store_name = store_name.split("/", 1)[0]

        store = self._stores.get(store_name)
        if not store:
            logger.error(
                "Store %s not found. Available stores: %s",
                store_name,
                self._stores.keys(),
            )
            raise ValueError(
                f"Store {store_name} not found. Available stores: {list(self._stores.keys())}"
            )
        return store

    def get_all_stores(self) -> Dict[str, StoreBase]:
        """
        Get all store instances.
        """
        return self._stores

    async def load_page_definition_all_stores(self, path: str) -> Page:
        """
        Load a page definition from all stores.

        This is for pages which do not have a proper schema, so we try them all in order
        """
        for store in self.get_all_stores().values():
            try:
                page_definition = await store.load_page_definition(path=path)
                if page_definition:
                    return page_definition
            except NotImplementedError:
                pass
        return None

    async def save_page_definition(self, path: str, data: Page) -> None:
        """
        Save a page definition to all stores.
        """
        if "/" in path:
            store_name, path = path.split("/", 1)
        else:
            store_name = "default"
        store = self.get_store(store_name)
        if not store:
            raise ValueError(f"Store {store_name} not found")
        await store.save_page_definition(path=path, data=data)

    async def delete_page_definition(
        self,
        path: str,
    ) -> bool:
        """
        Delete a page definition from all stores.
        """
        if "/" in path:
            store_name, path = path.split("/", 1)
        else:
            store_name = "default"
        store = self.get_store(store_name)
        if not store:
            raise ValueError(f"Store {store_name} not found")
        return await store.delete_page_definition(path=path)

    async def get_page_list(
        self, *, offset: int = 0, limit: int = 10, filter: dict | None = None
    ) -> PageListResult:
        """
        Get a list of all pages.

        It asks each store for the pages. We need to ask all to get the proper count.
        If the pending is 0 we just get the count of pages in that store.

        """
        res = PageListResult(count=0, results=[])
        pending = limit
        for store in self.get_all_stores().values():
            store_res = await store.get_page_list(
                offset=offset, limit=pending, filter=filter
            )
            logger.debug(
                "get_page_list store=%s offset=%s limit=%s add_count=%s response_count=%s filter=%s",
                store,
                offset,
                pending,
                store_res.count,
                len(store_res.results),
                json.dumps(filter),
            )

            for item in store_res.results:
                item.store = store.config.name
            res.results.extend(store_res.results)
            res.count += store_res.count
            offset = max(0, offset - store_res.count)
            if len(store_res.results) < pending:
                pending -= len(store_res.results)
            else:
                pending = 0
        return res
