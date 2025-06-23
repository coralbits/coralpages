"""
Types for the page editor.
"""

from typing import Any, Self
from dataclasses import dataclass


@dataclass
class BlockDefinition:
    """
    Each block definition, with content, and maybe more children
    """

    type: str
    data: Any
    children: list[Self]
    style: dict[str, str]

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a block definition from a dictionary.
        """
        return cls(
            type=data["type"],
            data=data.get("data", {}),
            children=[
                BlockDefinition.from_dict(child) for child in data.get("children", [])
            ],
            style=data.get("style", {}),
        )


@dataclass
class PageDefinition:
    """
    The page definition, with a title, and a list of blocks
    """

    title: str
    template: str
    data: list[BlockDefinition]

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a page definition from a dictionary.
        """
        return cls(
            title=data["title"],
            template=data["template"],
            data=[BlockDefinition.from_dict(block) for block in data["data"]],
        )


@dataclass
class FieldDefinition:
    """
    A field definition, with a name, a type, and a value
    """

    name: str
    type: str
    value: Any


@dataclass
class ElementDefinition:
    """
    Each block definition, with content, and maybe more children
    """

    name: str
    viewer: str
    editor: str | list[FieldDefinition]
    css: str

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load an element definition from a dictionary.
        """
        return cls(
            name=data["name"],
            viewer=data["viewer"],
            editor=data["editor"],
            css=data["css"],
        )
