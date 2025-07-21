import logging
from pathlib import Path

import yaml

from pe.renderer.renderer import Renderer, RenderedPage
from pe.types import Element, Page
from tests.base import TestCase

logger = logging.getLogger(__name__)


class TestRenderer(TestCase):
    def get_config(self):
        return super().get_config()

    async def test_render_text(self):
        renderer = Renderer(config=self.get_config())
        page = await renderer.render(
            page_def=Page(
                title="Test test_render_text",
                children=[
                    Element(
                        type="default://text",
                        data={"text": "Hello, world!"},
                    )
                ],
            )
        )
        self.assertIsNotNone(page)
        self.assertEqual(page.title, "Test test_render_text")
        self.assertIn("Hello, world!", page.content)

    async def test_render_html(self):
        """
        Test rendering a page with a HTML block.
        """
        renderer = Renderer(config=self.get_config())
        page: RenderedPage = await renderer.render(
            page_def=Page(
                title="Test test_render_html",
                children=[
                    Element(
                        type="http://apicontext",
                        data={"url": "test", "name": "test"},
                        children=[
                            Element(
                                type="default://text",
                                data={"text": "{{context.test.title}}"},
                            ),
                            Element(
                                type="default://text",
                                data={
                                    "text": "{% for item in context.test.array %}* {{item.name}}\n{% endfor %}"
                                },
                            ),
                        ],
                    )
                ],
            )
        )
        self.assertIsNotNone(page)
        self.assertEqual(page.title, "Test test_render_html")
        self.assertIn("Test JSON Data", page.content)
        self.assertIn("* test1", page.content)

        logger.debug("page=%s", page.content)

    async def test_page_render(self):
        """
        Test rendering a page with a HTML block.
        """
        renderer = Renderer(config=self.get_full_config())
        page = await renderer.render(
            Page.from_dict(
                yaml.safe_load(
                    open(Path(__file__).parent.parent / "docs" / "index.yaml")
                )
            )
        )
        logger.debug("page_size=%s", len(page.content))
        for error in page.errors:
            logger.error("error=%s", error)
        self.assertFalse(page.has_errors())
        self.assertIsNotNone(page)
