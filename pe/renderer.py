from pathlib import Path
from typing import Dict, Any, Optional, List
import yaml
from pe.types import Page, PageElement
from pe.loader import ElementLoader
from pe.css_generator import CSSGenerator
from pe.ports import CSSLoader


class ElementRendererFactory:
    """Factory for creating element renderers based on element type."""

    def __init__(self, element_loader: ElementLoader):
        self.element_loader = element_loader

    def create_renderer(self, element_type: str):
        """Create an appropriate renderer for the given element type."""
        if element_type == "children":
            return ChildrenRenderer(self.element_loader)
        else:
            # For now, all other types use BuiltInRenderer
            return BuiltInRenderer(self.element_loader)

    def render_element(
        self, element: PageElement, context: dict, css_class: str = ""
    ) -> str:
        """Render an element using the appropriate renderer."""
        renderer = self.create_renderer(element.type)
        return renderer.render(element, context, css_class)


class ElementRenderer:
    """Base class for element renderers."""

    def render(self, element: PageElement, context: dict, css_class: str = "") -> str:
        """Render an element. Must be implemented by subclasses."""
        raise NotImplementedError


class BuiltInRenderer(ElementRenderer):
    """Renderer for built-in element types."""

    def __init__(self, element_loader: ElementLoader):
        self.element_loader = element_loader

    def render(self, element: PageElement, context: dict, css_class: str = "") -> str:
        """Render a built-in element using the existing element loader."""
        return self.element_loader.render_element(element, context, css_class)


class ChildrenRenderer(ElementRenderer):
    """Special renderer for 'children' type elements."""

    def __init__(self, element_loader: ElementLoader):
        self.element_loader = element_loader

    def render(self, element: PageElement, context: dict, css_class: str = "") -> str:
        """Render children from context."""
        children = context.get("children", [])
        if not children:
            return ""

        # Render each child recursively using the element loader
        rendered_children = []
        for child in children:
            try:
                rendered_child = self.element_loader.render_element(
                    child, context, getattr(child, "css_class", "")
                )
                rendered_children.append(rendered_child)
            except Exception as e:
                # Handle child rendering errors
                error_html = f'<div class="error-placeholder" style="border: 2px solid red; padding: 10px; margin: 10px; background: #ffe6e6;">Error rendering child: {str(e)}</div>'
                rendered_children.append(error_html)

        return "\n".join(rendered_children)


class ErrorRenderer(ElementRenderer):
    """Renderer for elements that failed to load."""

    def render(self, element: PageElement, context: dict, css_class: str = "") -> str:
        """Render an error placeholder."""
        return f'<div class="error-placeholder" style="border: 2px solid red; padding: 10px; margin: 10px; background: #ffe6e6;">Error: Element type "{element.type}" not found or failed to load</div>'


class TemplateLoader:
    """Simplified template loader for YAML templates."""

    def __init__(self, base_directory: str):
        self.base_path = Path(base_directory)

    def load_template(self, template_name: str) -> Optional[Dict[str, Any]]:
        """Load a template by name."""
        # Handle namespaced templates (e.g., "builtin:plain")
        if ":" in template_name:
            namespace, name = template_name.split(":", 1)
            if namespace == "builtin":
                template_path = self.base_path / f"_{name}.yaml"
            else:
                return None
        else:
            # Default to builtin namespace
            template_path = self.base_path / f"_{template_name}.yaml"

        if not template_path.exists():
            return None

        try:
            with open(template_path, "r", encoding="utf-8") as f:
                return yaml.safe_load(f)
        except Exception:
            return None


class Renderer:
    def __init__(
        self,
        page: Page,
        element_loader: ElementLoader,
        css_loader: CSSLoader,
        template_loader: TemplateLoader,
    ):
        self.page = page
        self.element_loader = element_loader
        self.css_generator = CSSGenerator(css_loader)
        self.template_loader = template_loader
        self.factory = ElementRendererFactory(element_loader)
        self.has_errors = False

    def render(self) -> str:
        """Main render method that returns HTML."""
        # Apply templates recursively and get final page
        final_page = self.render_page_recursive(self.page)

        # Process elements and generate CSS classes
        processed_elements = self.css_generator.process_element_tree(final_page.data)

        # Generate HTML content
        html = ""
        context = {
            "title": final_page.title,
            "children": final_page.data,
            "has_errors": self.has_errors,
        }

        for element in processed_elements:
            try:
                html += self.factory.render_element(element, context, element.css_class)
            except Exception as e:
                # Handle rendering errors
                self.has_errors = True
                print(f"DEBUG: Error rendering element {element.type}: {e}")
                error_element = PageElement(
                    type="__error__",
                    data={"error": str(e), "original_type": element.type},
                    children=[],
                    style=None,
                )
                error_renderer = ErrorRenderer()
                html += error_renderer.render(error_element, context, "")

        # Generate CSS
        css = self.css_generator.get_css()

        # Wrap in HTML structure with CSS
        if css:
            return f"""<!DOCTYPE html>
<html>
<head>
    <title>{final_page.title}</title>
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
    <title>{final_page.title}</title>
</head>
<body>
{html}
</body>
</html>"""

    def render_page_recursive(self, page: Page) -> Page:
        """Recursively render a page, applying templates."""
        if not page.template:
            # No template, render children recursively
            rendered_children = []
            for child in page.data:
                try:
                    rendered_child = self.render_element_recursive(child)
                    # If the result is a list, extend; else, append
                    if isinstance(rendered_child, list):
                        rendered_children.extend(rendered_child)
                    else:
                        rendered_children.append(rendered_child)
                except Exception as e:
                    # Handle element rendering errors
                    self.has_errors = True
                    error_element = PageElement(
                        type="__error__",
                        data={"error": str(e), "original_type": child.type},
                        children=[],
                        style=None,
                    )
                    rendered_children.append(error_element)

            return Page(title=page.title, data=rendered_children, template="")

        # Load and apply template
        template_data = self.template_loader.load_template(page.template)
        if not template_data:
            # Template not found, mark as error and continue without template
            self.has_errors = True
            return self.render_page_recursive(
                Page(title=page.title, data=page.data, template="")
            )

        # Create context with page metadata and children
        context = {
            "title": page.title,
            "children": page.data,
            "template_name": page.template,
            "template_elements": template_data.get(
                "data", []
            ),  # Include template elements for recursion guard
        }

        # Process template elements
        template_elements = template_data.get("data", [])
        rendered_elements = []

        for element_data in template_elements:
            try:
                # Convert template element to PageElement
                element = PageElement.__from_dict__(element_data)

                # Use the same context for all elements
                rendered_element = self.render_element_recursive(element, context)
                if isinstance(rendered_element, list):
                    rendered_elements.extend(rendered_element)
                else:
                    rendered_elements.append(rendered_element)
            except Exception as e:
                # Handle template element errors
                self.has_errors = True
                error_element = PageElement(
                    type="__error__",
                    data={
                        "error": str(e),
                        "original_type": element_data.get("type", "unknown"),
                    },
                    children=[],
                    style=None,
                )
                rendered_elements.append(error_element)

        # Check if template has its own template (nested templates)
        if "template" in template_data:
            # Create a temporary page with the template's template
            temp_page = Page(
                title=page.title,
                data=rendered_elements,
                template=template_data["template"],
            )
            # Recursively apply the parent template
            return self.render_page_recursive(temp_page)

        return Page(title=page.title, data=rendered_elements, template="")

    def render_element_recursive(self, element: PageElement, context: dict = None):
        """Recursively render an element and its children."""
        if context is None:
            context = {}

        # Handle children type specially
        if element.type == "children":
            # Replace with children from context
            children_from_context = context.get("children", [])

            # Filter out elements that would cause infinite recursion
            # Check if any child has the same structure as elements already in the template
            filtered_children = []
            template_elements = context.get("template_elements", [])

            for child in children_from_context:
                # Skip if this child type already exists in the template to prevent recursion
                if any(
                    template_elem.get("type") == child.type
                    for template_elem in template_elements
                ):
                    print(
                        f"DEBUG: Skipping recursive element {child.type} to prevent infinite loop"
                    )
                    continue
                filtered_children.append(child)

            # Return the filtered children directly (as a list of PageElements)
            return filtered_children

        # For non-children elements, just return the element as-is
        # The actual rendering will be done by the element loader
        return element
