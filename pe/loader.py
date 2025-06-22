from pathlib import Path

import jinja2
import markdown

from pe.config import element_definitions
from pe.types import PageElement

# Create Jinja2 environment with markdown filter
jinja2_loader = jinja2.Environment(
    loader=jinja2.FileSystemLoader(Path(__file__).parent.parent / "templates")
)


# Add markdown filter to Jinja2 environment
def markdown_filter(text):
    """Convert markdown text to HTML"""
    if not text:
        return ""
    return markdown.markdown(text, extensions=["extra", "codehilite", "tables"])


jinja2_loader.filters["markdown"] = markdown_filter


class PEList(list):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    def __str__(self):
        return "\n".join(str(child) for child in self)


class ElementLoader:
    def __init__(self):
        pass

    def render_element(self, element: PageElement, context: dict, css_class: str = ""):
        element_definition = element_definitions.get(element.type)
        if not element_definition:
            raise ValueError(f"Element type {element.type} not found")

        viewer = element_definition.viewer

        if viewer.startswith("builtin://"):
            # Process children with their own CSS classes
            children = PEList(
                [
                    self.render_element(child, context, getattr(child, "css_class", ""))
                    for child in element.children
                ]
            )
            return self.render_builtin_element(
                type=viewer[10:],
                element=element,
                context=context,
                children=children,
                css_class=css_class,
            )
        raise NotImplementedError(f"Element type {element.type} not supported")

    def render_builtin_element(
        self, *, type, element, context, children, css_class: str = ""
    ):
        template = jinja2_loader.get_template(type)
        rendered = template.render(
            element=element, data=element.data, context=context, children=children
        )

        # Replace @@class@@ marker with actual CSS class
        if css_class:
            rendered = rendered.replace("@@class@@", css_class)
        else:
            rendered = rendered.replace("@@class@@", "")

        return rendered
