from typing import Optional
from pe.ports import PageRepository, PageRenderer
from pe.types import Page


class PageService:
    """Application service for page operations."""

    def __init__(self, page_repository: PageRepository, page_renderer: PageRenderer):
        self.page_repository = page_repository
        self.page_renderer = page_renderer

    def get_page_html(self, path: str) -> Optional[str]:
        """Get the HTML content for a page."""
        page = self.page_repository.get_page(path)
        if page is None:
            return None

        return self.page_renderer.render_page(page)

    def get_page_data(self, path: str) -> Optional[Page]:
        """Get the page data without rendering."""
        return self.page_repository.get_page(path)

    def page_exists(self, path: str) -> bool:
        """Check if a page exists."""
        return self.page_repository.page_exists(path)

    def list_pages(self, directory: str = "") -> list[str]:
        """List all pages in a directory."""
        return self.page_repository.list_pages(directory)
