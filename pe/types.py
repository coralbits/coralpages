"""
Types for the page editor.
"""

import datetime
from typing import Any, Self
from dataclasses import dataclass, field
import uuid


@dataclass
class MetaDefinition:
    """
    A meta definition.
    """

    name: str
    content: str

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a meta definition from a dictionary.
        """
        return cls(name=data["name"], content=data["content"])

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the meta definition to a JSON-serializable dictionary.
        """
        return {"name": self.name, "content": self.content}


@dataclass
class BlockDefinition:
    """
    Each block definition, with content, and maybe more children
    """

    type: str
    data: Any
    id: str | None = None
    children: list[Self] = field(default_factory=list)
    style: dict[str, str] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a block definition from a dictionary.
        """
        return cls(
            id=data.get("id", None) or f"_{uuid.uuid4()}",
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
            "id": self.id,
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

    title: str = ""
    template: str | None = None
    data: list[BlockDefinition] = field(default_factory=list)
    cache: list[str] = field(default_factory=list)
    last_modified: datetime.datetime | None = None
    meta: list[MetaDefinition] = field(default_factory=list)
    css_variables: dict[str, str] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a page definition from a dictionary.
        """
        last_modified = data.get("last_modified", None)
        if last_modified:
            if isinstance(last_modified, str):
                last_modified = datetime.datetime.fromisoformat(last_modified)

        return cls(
            title=data["title"],
            template=data["template"],
            data=[BlockDefinition.from_dict(block) for block in data["data"]],
            cache=data.get("cache", []),
            last_modified=last_modified,
            meta=[MetaDefinition.from_dict(meta) for meta in data.get("meta", [])],
            css_variables=data.get("css_variables", {}),
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
            "last_modified": (
                self.last_modified.isoformat() if self.last_modified else None
            ),
            "meta": [meta.to_dict() for meta in self.meta],
            "css_variables": self.css_variables,
        }


@dataclass
class FieldDefinition:
    """
    A field definition, with a name, a type, and a value
    """

    name: str
    type: str
    value: Any
    label: str
    placeholder: str

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a field definition from a dictionary.
        """
        return cls(
            name=data.get("name", ""),
            type=data.get("type", ""),
            value=data.get("value", ""),
            label=data.get("label", ""),
            placeholder=data.get("placeholder", ""),
        )

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the field definition to a JSON-serializable dictionary.
        """
        return clean_dict(
            {
                "name": self.name,
                "type": self.type,
                "value": self.value,
                "label": self.label,
                "placeholder": self.placeholder,
            }
        )


def clean_dict(data: dict[str, Any]) -> dict[str, Any]:
    """
    Clean a dictionary of None values.
    """
    return {k: v for k, v in data.items() if v is not None}


@dataclass
class ElementDefinition:
    """
    Each block definition, with content, and maybe more children
    """

    name: str
    store: str | None = None
    html: str | None = None
    editor: list[FieldDefinition] | None = None
    css: str | None = None
    method: str | None = None
    tags: list[str] = field(default_factory=list)
    icon: str | None = None

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load an element definition from a dictionary.
        """
        return cls(
            name=data["name"],
            store=data.get("store", None),
            html=data.get("html"),
            css=data.get("css"),
            method=data.get("method"),
            tags=data.get("tags", []),
            icon=data.get("icon"),
            editor=[
                FieldDefinition.from_dict(field) for field in data.get("editor", [])
            ],
        )

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the element definition to a JSON-serializable dictionary.
        """
        return {
            "name": self.name,
            "store": self.store,
            "html": self.html,
            "css": self.css,
            "method": self.method,
            "tags": self.tags,
            "icon": self.icon,
            "editor": [field.to_dict() for field in self.editor],
        }


@dataclass
class StoreConfig:
    """
    Configuration for a store.
    """

    name: str
    type: str
    path: str | None = None
    base_url: str | None = None
    url: str | None = None
    tags: list[str] = field(default_factory=list)
    blocks: list[ElementDefinition] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a store configuration from a dictionary.
        """
        tags = data.get("tags", [])

        blocks = [
            ElementDefinition.from_dict(block) for block in data.get("blocks", [])
        ]
        for block in blocks:
            block.store = data["name"]
            block.tags.extend(tags)

        return cls(
            name=data["name"],
            type=data["type"],
            path=data.get("path"),
            base_url=data.get("base_url"),
            url=data.get("url"),
            tags=tags,
            blocks=blocks,
        )

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the store configuration to a JSON-serializable dictionary.
        """
        return {
            "name": self.name,
            "type": self.type,
            "path": self.path,
            "base_url": self.base_url,
            "url": self.url,
            "tags": self.tags,
            "blocks": [block.to_dict() for block in self.blocks],
        }
