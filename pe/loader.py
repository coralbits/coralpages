from pathlib import Path

import jinja2

from pe.config import element_definitions
from pe.types import PageElement

jinja2_loader = jinja2.Environment(loader=jinja2.FileSystemLoader(Path(__file__).parent.parent / 'templates'))

class ElementLoader:
    def __init__(self):
        pass

    def render_element(self, element: PageElement, context: dict):
        element_definition = element_definitions.get(element.type)
        if not element_definition:
            raise ValueError(f"Element type {element.type} not found")

        viewer = element_definition.viewer

        if viewer.startswith("builtin://"):
            return self.render_builtin_element(viewer[10:], element, context)
        raise NotImplementedError(f"Element type {element.type} not supported")


    def render_builtin_element(self, builtin_template, element, context):
        template = jinja2_loader.get_template(builtin_template)
        return template.render(element=element, data=element.data, context=context)
