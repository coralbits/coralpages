"""
Types for the page editor.
"""

from typing import Any, Self
from dataclasses import dataclass, field


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

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the block definition to a JSON-serializable dictionary.
        """
        return {
            "type": self.type,
            "data": self.data,
            "children": [child.to_dict() for child in self.children],
            "style": self.style,
        }


@dataclass
class PageDefinition:
    """
    The page definition, with a title, and a list of blocks
    """

    title: str
    template: str
    data: list[BlockDefinition]
    cache: list[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a page definition from a dictionary.
        """
        return cls(
            title=data["title"],
            template=data["template"],
            data=[BlockDefinition.from_dict(block) for block in data["data"]],
            cache=data.get("cache", []),
        )

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the page definition to a JSON-serializable dictionary.
        """
        return {
            "title": self.title,
            "template": self.template,
            "cache": self.cache,
            "data": [block.to_dict() for block in self.data],
        }


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
    type: str
    viewer: str | None
    editor: str | list[FieldDefinition] | None
    css: str | None

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load an element definition from a dictionary.
        """
        return cls(
            name=data["name"],
            type=data["type"],
            viewer=data.get("viewer", None),
            editor=None,  # TODO: add editor
            css=data.get("css", None),
        )

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the element definition to a JSON-serializable dictionary.
        """
        editor_data = self.editor
        if isinstance(editor_data, list):
            # Convert FieldDefinition objects to dictionaries
            editor_data = [
                {"name": field.name, "type": field.type, "value": field.value}
                for field in editor_data
            ]

        return {
            "name": self.name,
            "type": self.type,
            "viewer": self.viewer,
            "editor": editor_data,
            "css": self.css,
        }
