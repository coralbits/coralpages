import os
from pathlib import Path
from typing import Optional
from pe.ports import PageRepository, PageRenderer, CSSLoader
from pe.types import Page
from pe.page import YamlLoader
from pe.renderer import Renderer
from pe.loader import ElementLoader
from pe.template_loader import TemplateLoader
import jinja2


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
        """List all pages in a directory."""
        dir_path = self.base_path / directory
        if not dir_path.exists() or not dir_path.is_dir():
            return []

        pages = []
        for item in dir_path.iterdir():
            if item.is_file() and item.suffix == ".yaml":
                # Remove .yaml extension and base path
                relative_path = str(item.relative_to(self.base_path))[:-5]
                pages.append(relative_path)
            elif item.is_dir() and (item / "index.yaml").exists():
                # Add directory with trailing slash
                relative_path = str(item.relative_to(self.base_path)) + "/"
                pages.append(relative_path)

        return sorted(pages)


class PePageRenderer(PageRenderer):
    """PE renderer implementation of PageRenderer."""

    def __init__(self, css_loader: CSSLoader, template_loader: TemplateLoader):
        self.element_loader = ElementLoader()
        self.css_loader = css_loader
        self.template_loader = template_loader

    def render_page(self, page: Page) -> str:
        """Render a page using the PE renderer."""
        renderer = Renderer(
            page, self.element_loader, self.css_loader, self.template_loader
        )
        return renderer.render()


class BuiltinCSSLoader(CSSLoader):
    """Builtin CSS loader implementation using Jinja2 templates."""

    def __init__(self):
        self.jinja2_env = jinja2.Environment(
            loader=jinja2.FileSystemLoader(Path(__file__).parent.parent / "templates")
        )
        # self.css_cache: dict[str, str] = {}  # Disabled for development

    def load_css(self, css_path: str) -> str:
        """Load CSS content from a builtin path."""
        # if css_path in self.css_cache:  # Disabled for development
        #     return self.css_cache[css_path]  # Disabled for development

        if not css_path.startswith("builtin://"):
            return ""

        # Remove builtin:// prefix
        template_path = css_path[10:]

        try:
            template = self.jinja2_env.get_template(template_path)
            css_content = template.render()
            # self.css_cache[css_path] = css_content  # Disabled for development
            return css_content
        except Exception:
            # self.css_cache[css_path] = ""  # Disabled for development
            return ""
