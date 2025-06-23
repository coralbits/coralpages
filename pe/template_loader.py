from pathlib import Path
from typing import Optional, Dict, Any
import yaml
from pe.types import Page, PageElement
from pe.page import YamlLoader


class TemplateLoader:
    """Loads and manages page templates."""

    def __init__(self, base_directory: str):
        self.base_path = Path(base_directory)
        self.yaml_loader = YamlLoader()
        # self.template_cache: Dict[str, Dict[str, Any]] = {}  # Disabled for development

    def load_template(self, template_name: str) -> Optional[Dict[str, Any]]:
        """Load a template by name, supporting namespaced templates."""
        # if template_name in self.template_cache:  # Disabled for development
        #     return self.template_cache[template_name]  # Disabled for development

        # Handle namespaced templates (e.g., "builtin:plain")
        if ":" in template_name:
            namespace, name = template_name.split(":", 1)
            if namespace == "builtin":
                template_path = self.base_path / f"_{name}.yaml"
            else:
                # Could support other namespaces in the future
                return None
        else:
            # Default to builtin namespace
            template_path = self.base_path / f"_{template_name}.yaml"

        if not template_path.exists():
            return None

        try:
            with open(template_path, "r", encoding="utf-8") as f:
                template_data = yaml.safe_load(f)
                # self.template_cache[template_name] = template_data  # Disabled for development
                return template_data
        except Exception:
            return None

    def apply_template(self, page: Page) -> Page:
        """Apply a template to a page, supporting nested templates."""
        if not page.template:
            return page

        template_data = self.load_template(page.template)
        if not template_data:
            return page

        # Create a new page with template data merged
        merged_data = self._merge_template_data(template_data, page)

        # Check if the template itself has a template (nested templates)
        if "template" in template_data:
            # Create a temporary page with the template's template
            temp_page = Page(
                title=page.title, data=merged_data, template=template_data["template"]
            )
            # Recursively apply the parent template
            return self.apply_template(temp_page)

        return Page(
            title=page.title,
            data=merged_data,
            template="",  # Clear template after applying
        )

    def _merge_template_data(
        self, template_data: Dict[str, Any], page: Page
    ) -> list[PageElement]:
        """Merge template data with page data."""
        template_elements = template_data.get("data", [])

        # Process template elements and replace children placeholders
        merged_elements = []
        for element in template_elements:
            # Convert template element to PageElement and process its children
            processed_element = self._process_template_element(element, page.data)
            # Check if this is a children placeholder
            if processed_element.type == "__children_placeholder__":
                # Replace with the page data
                merged_elements.extend(processed_element.children)
            else:
                merged_elements.append(processed_element)

        return merged_elements

    def _process_template_element(
        self, element_data: Dict[str, Any], page_data: list[PageElement]
    ) -> PageElement:
        """Process a template element, replacing children placeholders."""
        # Handle children type as a special case
        if element_data.get("type") == "children":
            # Create a dummy element that will be replaced with page data
            # We'll use a special marker to identify this
            return PageElement(
                type="__children_placeholder__",
                data={},
                children=page_data,  # Store page data in children
                style=None,
            )

        # Create a copy of element_data without children for initial conversion
        element_data_copy = element_data.copy()
        original_children = element_data_copy.pop("children", [])

        # Convert to PageElement first (without children)
        element = PageElement.__from_dict__(element_data_copy)

        # Process children separately
        processed_children = []
        for child_data in original_children:
            if child_data.get("type") == "children":
                # Replace children placeholder with page data
                processed_children.extend(page_data)
            else:
                # Recursively process this child
                processed_child = self._process_template_element(child_data, page_data)
                processed_children.append(processed_child)

        # Create new element with processed children
        return PageElement(
            type=element.type,
            data=element.data,
            children=processed_children,
            style=element.style,
        )
