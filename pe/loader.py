"""
Loaders for the page editor.
"""

import datetime
from pathlib import Path

import yaml
from pe.config import Config
from pe.types import BlockDefinition, PageDefinition


class LoaderBase:
    def load(self, path: str) -> PageDefinition:
        """
        Load a page
        """
        raise NotImplementedError("Not implemented")


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
            self.file_loader = FileLoader(self.config.page_path)
        return self.file_loader


class FileLoader(LoaderBase):
    """
    Load a page from a file.
    """

    def __init__(self, base_path: Path):
        self.base_path = base_path

    def load(self, path: str) -> PageDefinition:
        path = f"{path}.yaml"
        filepath = self.base_path / path
        with open(filepath, "r", encoding="utf-8") as file:
            data = yaml.safe_load(file)
            page_def = PageDefinition.from_dict(data)
            if not page_def.last_modified:
                page_def.last_modified = datetime.datetime.fromtimestamp(
                    filepath.stat().st_mtime
                )
            return page_def
