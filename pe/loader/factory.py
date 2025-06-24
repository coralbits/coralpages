"""
Loaders for the page editor.
"""

from pe.config import Config
from pe.types import PageDefinition
from pe.loader.types import LoaderBase


class LoaderFactory:
    """
    Depending on URLs will use another loader. This is the generic one.
    """

    file_loader: LoaderBase = None

    def __init__(self, config: Config):
        self.config = config

    def load(self, path: str) -> PageDefinition:
        """
        Load a page from the config.
        """
        if path.startswith("page://"):
            return self.get_file_loader().load(path.split("://", 1)[1])

        raise ValueError(f"Unsupported path: {path}")

    def get_file_loader(self) -> LoaderBase:
        """
        Get the file loader.
        """
        if not self.file_loader:
            from pe.loader.file import FileLoader

            self.file_loader = FileLoader(self.config.page_path)
        return self.file_loader
