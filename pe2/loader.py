"""
Loaders for the page editor.
"""

from pathlib import Path

import yaml
from pe2.types import BlockDefinition, ElementDefinition, PageDefinition


class LoaderBase:
    def load_page(self, path: str) -> PageDefinition:
        """
        Load a page
        """
        raise NotImplementedError("Not implemented")

    def load_element_definition(self, type_name: str) -> ElementDefinition:
        """
        Load a definition
        """
        raise NotImplementedError("Not implemented")


class LoaderFactory:
    """
    Depending on URLs will use another loader. This is the generic one.
    """

    file_loader: LoaderBase

    def __init__(self, config: dict[str, str]):
        self.config = config

    def load_page(self, path: str) -> PageDefinition:
        """
        Load a page from the config.
        """
        if path.startswith("builtin"):
            return self.get_file_loader().load_page(path[10:])

        raise ValueError(f"Unsupported path: {path}")

    def load_element_definition(self, type_name: str) -> ElementDefinition:
        """
        Load an element definition from the config.
        """
        if type_name in self.config["elements"]:
            return self.config["elements"][type_name]

        raise ValueError(f"Unsupported type: {type_name}")

    def get_file_loader(self) -> LoaderBase:
        """
        Get the file loader.
        """
        if not self.file_loader:
            self.file_loader = FileLoader(self.config["page_path"])
        return self.file_loader


class FileLoader(LoaderBase):
    """
    Load a page from a file.
    """

    def __init__(self, base_path: str):
        self.base_path = Path(base_path)

    def load_page(self, path: str) -> PageDefinition:
        with open(self.base_path / path, "r", encoding="utf-8") as file:
            data = yaml.safe_load(file)
            return PageDefinition.from_dict(data)

    def load_definition(self, path: str) -> PageDefinition:
        with open(self.base_path / path, "r", encoding="utf-8") as file:
            data = yaml.safe_load(file)
            return PageDefinition.from_dict(data)
