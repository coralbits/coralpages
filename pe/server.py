"""
Server module for uvicorn reload mode.
This module creates the FastAPI application that can be imported by uvicorn.
"""

import os
import sys
from pathlib import Path

# Add the project root to the path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

from pe.services import PageService
from pe.adapters import FileSystemPageRepository, PePageRenderer
from pe.api import create_app


def create_page_service(base_directory: str) -> PageService:
    """Create a page service with file system storage."""
    page_repository = FileSystemPageRepository(base_directory)
    page_renderer = PePageRenderer()
    return PageService(page_repository, page_renderer)


# Get the base directory from environment variable or use default
base_directory = os.environ.get("PAGE_EDITOR_BASE_DIR", "test_pages")

# Create the application
page_service = create_page_service(base_directory)
app = create_app(page_service)
