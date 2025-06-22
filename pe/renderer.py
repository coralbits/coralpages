from pe.types import Page
from pe.loader import ElementLoader
from pe.css_generator import CSSGenerator


class Renderer:
    def __init__(self, page: Page, element_loader: ElementLoader):
        self.page = page
        self.element_loader = element_loader
        self.css_generator = CSSGenerator()

    def render(self) -> str:
        # Process elements and generate CSS classes
        processed_elements = self.css_generator.process_element_tree(self.page.data)

        # Generate HTML content
        html = ""
        context = {}
        for element in processed_elements:
            html += self.element_loader.render_element(
                element, context, element.css_class
            )

        # Generate CSS
        css = self.css_generator.get_css()

        # Wrap in HTML structure with CSS
        if css:
            return f"""<!DOCTYPE html>
<html>
<head>
    <title>{self.page.title}</title>
    <style>
{css}
    </style>
</head>
<body>
{html}
</body>
</html>"""
        else:
            return f"""<!DOCTYPE html>
<html>
<head>
    <title>{self.page.title}</title>
</head>
<body>
{html}
</body>
</html>"""
