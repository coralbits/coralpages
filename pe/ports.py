from abc import ABC, abstractmethod
from pathlib import Path
from typing import Optional
from pe.types import Page


class PageRepository(ABC):
    """Port for page storage operations."""

    @abstractmethod
    def get_page(self, path: str) -> Optional[Page]:
        """Retrieve a page by its path."""
        pass

    @abstractmethod
    def page_exists(self, path: str) -> bool:
        """Check if a page exists at the given path."""
        pass

    @abstractmethod
    def list_pages(self, directory: str = "") -> list[str]:
        """List all available pages in a directory."""
        pass


class PageRenderer(ABC):
    """Port for page rendering operations."""

    @abstractmethod
    def render_page(self, page: Page) -> str:
        """Render a page to HTML."""
        pass
