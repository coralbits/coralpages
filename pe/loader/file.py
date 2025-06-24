"""
Loaders for the page editor.
"""

import datetime
from pathlib import Path

import yaml
from pe.loader.types import LoaderBase
from pe.types import PageDefinition


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
