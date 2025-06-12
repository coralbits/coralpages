from dataclasses import dataclass
from typing import Any
from typing import Self

@dataclass
class PageElement:
    type: str
    data: Any
    children: list[Self]

    @classmethod
    def __from_dict__(cls, data: dict):
        return cls(
            type=data['type'],
            data=data['data'],
            children=[
                cls.__from_dict__(x) for x in
                data.get("children", [])
            ]
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
