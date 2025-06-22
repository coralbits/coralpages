import os
from pathlib import Path
from typing import Optional
from pe.ports import PageRepository, PageRenderer
from pe.types import Page
from pe.page import YamlLoader
from pe.renderer import Renderer
from pe.loader import ElementLoader


class FileSystemPageRepository(PageRepository):
    """File system implementation of PageRepository."""

    def __init__(self, base_directory: str):
        self.base_path = Path(base_directory)
        self.yaml_loader = YamlLoader()

    def get_page(self, path: str) -> Optional[Page]:
        """Retrieve a page from the file system."""
        try:
            # Handle directory requests by looking for index.yaml
            if path.endswith("/") or not path:
                page_path = self.base_path / path / "index.yaml"
            else:
                # First try the exact path with .yaml extension
                page_path = self.base_path / f"{path}.yaml"
                if not page_path.exists():
                    # If not found, try as a directory with index.yaml
                    page_path = self.base_path / path / "index.yaml"

            if not page_path.exists():
                return None

            return self.yaml_loader.open(str(page_path))
        except Exception:
            return None

    def page_exists(self, path: str) -> bool:
        """Check if a page exists in the file system."""
        if path.endswith("/") or not path:
            page_path = self.base_path / path / "index.yaml"
        else:
            # Check both possibilities
            yaml_path = self.base_path / f"{path}.yaml"
            index_path = self.base_path / path / "index.yaml"
            return yaml_path.exists() or index_path.exists()

        return page_path.exists()

    def list_pages(self, directory: str = "") -> list[str]:
        """List all available pages in a directory."""
        dir_path = self.base_path / directory
        if not dir_path.exists() or not dir_path.is_dir():
            return []

        pages = []
        for item in dir_path.iterdir():
            if item.is_file() and item.suffix == ".yaml":
                # Remove .yaml extension and add to list
                relative_path = item.relative_to(self.base_path)
                pages.append(str(relative_path.with_suffix("")))
            elif item.is_dir() and (item / "index.yaml").exists():
                # Add directory with trailing slash to indicate it's a directory
                relative_path = item.relative_to(self.base_path)
                pages.append(f"{relative_path}/")

        return sorted(pages)


class PePageRenderer(PageRenderer):
    """PE renderer implementation of PageRenderer."""

    def __init__(self):
        self.element_loader = ElementLoader()

    def render_page(self, page: Page) -> str:
        """Render a page using the PE renderer."""
        renderer = Renderer(page, self.element_loader)
        return renderer.render()
