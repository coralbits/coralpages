from pe.types import Page
from pe.loader import ElementLoader

class Renderer:
    def __init__(self, page: Page, element_loader: ElementLoader):
        self.page = page
        self.element_loader = element_loader

    def render(self) -> str:
        html = ""

        context = {}
        for element in self.page.data:
            html += self.element_loader.render_element(element, context)

        return html
