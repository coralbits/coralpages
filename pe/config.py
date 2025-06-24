from dataclasses import dataclass, field
from typing import Self
from pathlib import Path

import yaml
from pe.types import ElementDefinition, StoreConfig


@dataclass
class ServerConfig:
    """
    The server configuration.
    """

    port: int = 8000
    host: str = "0.0.0.0"
    reload: bool = False
    directory: list[Path] = field(default_factory=list)
    etag_salt: str = "%Y-%m-%d"

    @staticmethod
    def from_dict(data: dict) -> Self:
        """
        Load the server configuration from a dictionary.
        """
        return ServerConfig(**data)


@dataclass
class Config:
    """
    The configuration for the page editor.
    """

    page_path: Path = field(default_factory=Path)
    elements: dict[str, ElementDefinition] = field(default_factory=dict)
    stores: dict[str, StoreConfig] = field(default_factory=dict)
    debug: bool = False
    server: ServerConfig = field(default_factory=ServerConfig)

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
            stores={
                store["name"]: StoreConfig.from_dict(store)
                for store in data.get("stores", [])
            },
            server=ServerConfig.from_dict(data.get("server", {})),
        )
