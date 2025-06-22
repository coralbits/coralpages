from typing import Dict, List, Tuple
from pe.types import PageElement


class CSSGenerator:
    """Generates CSS classes and styles for page elements."""

    def __init__(self):
        self.css_rules: Dict[str, Dict[str, str]] = {}
        self.element_paths: Dict[int, str] = {}
        self.element_counter = 0

    def generate_class_name(self, element_path: str) -> str:
        """Generate a CSS class name from the element path."""
        # Convert path like "root.header1.section2" to "root_header1_section2"
        # Also handle special characters like hyphens
        return element_path.replace(".", "_").replace("-", "_")

    def add_style(self, element_path: str, style: Dict[str, str]) -> str:
        """Add a style and return the CSS class name."""
        if not style:
            return ""

        class_name = self.generate_class_name(element_path)
        self.css_rules[class_name] = style
        return class_name

    def get_css(self) -> str:
        """Generate the complete CSS string."""
        css_lines = []
        for class_name, styles in self.css_rules.items():
            css_lines.append(f".{class_name} {{")
            for property_name, value in styles.items():
                # CSS properties should be in kebab-case, but we'll keep them as-is
                # since they're already in the correct format from the YAML
                css_lines.append(f"  {property_name}: {value};")
            css_lines.append("}")
        return "\n".join(css_lines)

    def process_element_tree(
        self, elements: List[PageElement], parent_path: str = "root"
    ) -> List[PageElement]:
        """Process the element tree and assign CSS classes."""
        processed_elements = []

        for i, element in enumerate(elements):
            # Generate element path
            element_path = f"{parent_path}.{element.type}{i+1}"

            # Add style if present and store CSS class on element
            if element.style:
                css_class = self.add_style(element_path, element.style)
                element.css_class = css_class
            else:
                element.css_class = ""

            # Process children recursively
            if element.children:
                element.children = self.process_element_tree(
                    element.children, element_path
                )

            processed_elements.append(element)

        return processed_elements
