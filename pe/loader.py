from pathlib import Path

import jinja2

from pe.config import element_definitions
from pe.types import PageElement

jinja2_loader = jinja2.Environment(loader=jinja2.FileSystemLoader(Path(__file__).parent.parent / 'templates'))

class PEList(list):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
    def __str__(self):
        return '\n'.join(str(child) for child in self)

class ElementLoader:
    def __init__(self):
        pass

    def render_element(self, element: PageElement, context: dict):
        element_definition = element_definitions.get(element.type)
        if not element_definition:
            raise ValueError(f"Element type {element.type} not found")

        viewer = element_definition.viewer

        if viewer.startswith("builtin://"):
            children = PEList([
                self.render_element(child, context)
                for child in element.children
            ])
            return self.render_builtin_element(type=viewer[10:], element=element, context=context, children=children)
        raise NotImplementedError(f"Element type {element.type} not supported")


    def render_builtin_element(self, *, type, element, context, children):
        template = jinja2_loader.get_template(type)
        return template.render(element=element, data=element.data, context=context, children=children)
