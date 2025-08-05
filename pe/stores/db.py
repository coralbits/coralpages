"""
Database store implementation.
"""

import json
import logging
from typing import Any, List
import sqlite3
import os
from pathlib import Path

from pe.stores.types import StoreBase
from pe.types import Page, PageInfo, PageListResult, StoreConfig

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
        os.makedirs(Path(path).parent, exist_ok=True)
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
            logger.info("Saved page_id=%s", path)

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

    async def get_page_list(
        self, *, offset: int = 0, limit: int = 10, filter: dict | None = None
    ) -> PageListResult:
        """
        Get a list of all pages.
        """
        if "pages" not in self.config.tags:
            return PageListResult(count=0, results=[])

        sql_filter = []
        if filter and filter.get("type") == "template":
            sql_filter.append("path LIKE '\\_%'")

        if sql_filter:
            sql_filter = "WHERE " + " AND ".join(sql_filter)
        else:
            sql_filter = ""

        with self.conn:
            cursor = self.conn.cursor()
            cursor.execute(f"SELECT COUNT(*) FROM pages {sql_filter}")
            count = cursor.fetchone()[0]
            results = []
            if limit > 0:
                cursor.execute(
                    f"SELECT path, data FROM pages {sql_filter} LIMIT ? OFFSET ?",
                    (limit, offset),
                )
                for row in cursor.fetchall():
                    jsondata = json.loads(row["data"])
                    results.append(
                        PageInfo(id=row["path"], title=jsondata["title"], url="")
                    )

            return PageListResult(count=count, results=results)

    async def delete_page_definition(
        self,
        path: str,
    ) -> bool:
        """
        Delete a page definition from the database store.
        """
        with self.conn:
            cursor = self.conn.cursor()
            cursor.execute("DELETE FROM pages WHERE path = ?", (path,))
            if cursor.rowcount == 0:
                logger.error(
                    f"Failed to delete page page={path}, maybe does not exist in db?"
                )
                return False
            self.conn.commit()
            logger.info("Deleted page_id=%s", path)
            return True
