from dataclasses import dataclass, field
from pe2.loader import LoaderFactory
from pe2.renderer.types import ElementRendererBase
from pe2.types import BlockDefinition, PageDefinition


@dataclass
class RenderedPage:
    title: str
    content: str = ""
    classes: dict[str, str] = field(default_factory=dict)

    def __str__(self):
        css = "\n".join([f"{k} {{ {v} }}" for k, v in self.classes.items()])
        content = self.content
        title = self.title
        return f"""<!DOCTYPE html>
<html>
<head>
<title>{title}</title>
<style>{css}</style>
</head>
<body>
{content}
</body>
</html>
"""


class Renderer:
    """
    Renderer for the page editor.
    """

    def __init__(self, config: dict[str, str]):
        self.config = config
        self.loader = LoaderFactory(config=config)

    def render(self, page: PageDefinition) -> str:
        """
        Render a page.
        """
        ret = RenderedPage(title=page.title)

        for block in page.data:
            ret.content += self.render_block(block)

        return ret

    def render_block(self, block: BlockDefinition) -> str:
        element_renderer = self.get_element_renderer(block.type)

        return element_renderer.render(data=block.data, context={})

    def get_element_renderer(self, type_name: str) -> ElementRendererBase:
        """
        Get an element renderer.
        """
        block = self.loader.load_element_definition(type_name)

        if block.viewer.startswith("builtin:"):
            from pe2.renderer.builtin import ElementRendererBuiltin

            return ElementRendererBuiltin(block=block)
