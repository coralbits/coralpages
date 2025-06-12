from dataclasses import dataclass
from typing import Any

@dataclass
class PageElement:
    type: str
    data: Any

    @classmethod
    def __from_dict__(cls, data: dict):
        return cls(
            type=data['type'],
            data=data['data'],
        )

@dataclass
class Page:
    title: str
    data: list[PageElement]
    template: str

    @classmethod
    def __from_dict__(cls, data: dict):
        return cls(
            title=data['title'],
            data=[PageElement.__from_dict__(elem) for elem in data['data']],
            template=data['template']
        )

@dataclass
class ElementDefinition:
    name: str
    viewer: str
    editor: str

    @classmethod
    def __from_dict__(cls, data: dict):
        return cls(
            name=data['name'],
            viewer=data['viewer'],
            editor=data['editor']
        )
