"""
Database store implementation.
"""

import json
import logging
from typing import Any
import sqlite3

from pe.stores.types import StoreBase
from pe.types import Page, StoreConfig

logger = logging.getLogger(__name__)


class DbStore(StoreBase):
    """
    Database-based store implementation.
    """

    def __init__(self, config: StoreConfig):
        super().__init__(config)
        logger.info("Connecting to database: %s", config.url)
        if not config.url.startswith("sqlite://"):
            raise ValueError("Database URL must start with sqlite://")
        path = config.url.replace("sqlite://", "")
        self.conn = sqlite3.connect(path)
        self.conn.row_factory = sqlite3.Row

        self.make_migrations()

    async def load_html(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a page from the database store.
        """
        with self.conn:
            cursor = self.conn.cursor()
            cursor.execute("SELECT html FROM elements WHERE path = ?", (path,))
            result = cursor.fetchone()
        if result:
            return result["html"]
        return None

    async def load_css(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load an element from the database store.
        """
        with self.conn:
            cursor = self.conn.cursor()
            cursor.execute("SELECT css FROM elements WHERE path = ?", (path,))
            result = cursor.fetchone()
        if result:
            return result["css"]
        return None

    async def load_page_definition(self, *, path: str) -> Page | None:
        """
        Load a page definition from the database store.
        """
        with self.conn:
            cursor = self.conn.cursor()
            cursor.execute("SELECT data FROM pages WHERE path = ?", (path,))
            result = cursor.fetchone()
        if result:
            page_def = json.loads(result["data"])
            return Page.from_dict(page_def)
        return None

    async def save_page_definition(self, *, path: str, data: Page) -> None:
        """
        Save a page definition to the database store.
        """
        with self.conn:
            cursor = self.conn.cursor()
            page_def = json.dumps(data.to_dict())
            # first update, and if not found, insert
            cursor.execute("UPDATE pages SET data = ? WHERE path = ?", (page_def, path))
            if cursor.rowcount == 0:
                cursor.execute(
                    "INSERT INTO pages (path, data) VALUES (?, ?)", (path, page_def)
                )
            self.conn.commit()

    def make_migrations(self) -> None:
        """
        Make migrations to the database.
        """
        with self.conn:
            cursor = self.conn.cursor()
            cursor.execute(
                "CREATE TABLE IF NOT EXISTS pages (path TEXT PRIMARY KEY, data JSON)"
            )
            cursor.execute(
                "CREATE TABLE IF NOT EXISTS elements (path TEXT PRIMARY KEY, html TEXT, css TEXT, data JSON)"
            )
