from dataclasses import dataclass, field
from typing import Self
from pathlib import Path

import yaml
from pe.types import ElementDefinition


@dataclass
class Config:
    """
    The configuration for the page editor.
    """

    page_path: Path = field(default_factory=Path)
    elements: dict[str, ElementDefinition] = field(default_factory=dict)
    debug: bool = False

    @staticmethod
    def read(path: str) -> Self:
        """
        Read the configuration from a file.
        """
        with open(path, "r", encoding="utf-8") as file:
            data = yaml.safe_load(file)
            return Config.from_dict(data)

    @staticmethod
    def from_dict(data: dict) -> Self:
        """
        Load the configuration from a dictionary.
        """
        return Config(
            debug=data.get("debug", False),
            page_path=Path(data.get("page_path", "")),
            elements={
                element["name"]: ElementDefinition.from_dict(element)
                for element in data.get("elements", [])
            },
        )
