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
class Element:
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
            children=[Element.from_dict(child) for child in data.get("children", [])],
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
class Page:
    """
    The page definition, with a title, and a list of blocks
    """

    title: str = ""
    url: str | None = None
    template: str | None = None
    children: list[Element] = field(default_factory=list)
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
            url=data.get("url", None),
            template=data["template"],
            children=[Element.from_dict(block) for block in data.get("children", [])],
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
            "url": self.url,
            "template": self.template,
            "cache": self.cache,
            "children": [child.to_dict() for child in self.children],
            "last_modified": (
                self.last_modified.isoformat() if self.last_modified else None
            ),
            "meta": [meta.to_dict() for meta in self.meta],
            "css_variables": self.css_variables,
        }


@dataclass
class PageInfo:
    """
    A page info, with a title, and a url
    """

    id: str
    title: str
    url: str
    store: str = ""

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the page info to a JSON-serializable dictionary.
        """
        return {
            "id": self.id,
            "title": self.title,
            "url": self.url,
            "store": self.store,
        }


@dataclass
class PageListResult:
    """
    A page list result, with a list of pages and a total count
    """

    count: int  # total count on this store. It might return less elements, which means that we are at the end.
    results: list[PageInfo]


@dataclass
class FieldDefinition:
    """
    A field definition, with a name, a type, and a value
    """

    name: str
    type: str
    label: str | None = None
    placeholder: str | None = None
    options: list[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a field definition from a dictionary.
        """
        return cls(
            name=data.get("name", ""),
            type=data.get("type", ""),
            label=data.get("label", None),
            placeholder=data.get("placeholder", None),
            options=data.get("options", []),
        )

    def to_dict(self) -> dict[str, Any]:
        """
        Convert the field definition to a JSON-serializable dictionary.
        """
        return clean_dict(
            {
                "name": self.name,
                "type": self.type,
                "label": self.label,
                "placeholder": self.placeholder,
                "options": self.options,
            }
        )


def clean_dict(data: dict[str, Any]) -> dict[str, Any]:
    """
    Clean a dictionary of None values.
    """
    return {k: v for k, v in data.items() if v is not None}


@dataclass
class Widget:
    """
    Each block definition, with content, and maybe more children
    """

    name: str
    description: str | None = None
    store: str | None = None
    html: str | None = None
    editor: list[FieldDefinition] | None = None
    css: str | None = None
    tags: list[str] = field(default_factory=list)
    icon: str | None = None
    children: bool = False

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load an element definition from a dictionary.
        """
        return cls(
            name=data["name"],
            store=data.get("store", None),
            description=data.get("description", None),
            html=data.get("html"),
            css=data.get("css"),
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
            "description": self.description,
            "html": self.html,
            "css": self.css,
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
    blocks: list[Widget] = field(default_factory=list)
    config: dict[str, Any] = field(default_factory=dict)

    def get(self, key: str, default: Any = None) -> Any:
        """
        Get a configuration value.
        """
        return self.config.get(key, default)

    @classmethod
    def from_dict(cls, data: dict) -> Self:
        """
        Load a store configuration from a dictionary.
        """
        tags = data.get("tags", [])

        blocks = [Widget.from_dict(block) for block in data.get("blocks", [])]
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
            config=data,
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
            **map_drop(
                self.config,
                ["name", "type", "path", "base_url", "url", "tags", "blocks"],
            ),
        }


def map_drop(data: dict[str, Any], keys: list[str]) -> dict[str, Any]:
    """
    Map a dictionary, dropping the keys.
    """
    return {k: v for k, v in data.items() if k not in keys}
