import logging

from pe.renderer.renderer import Renderer, RenderedPage
from pe.types import BlockDefinition, PageDefinition
from tests.base import TestCase

logger = logging.getLogger(__name__)


class TestRenderer(TestCase):
    def get_config(self):
        return super().get_config()

    async def test_render_text(self):
        renderer = Renderer(config=self.get_config())
        page = await renderer.render(
            page_def=PageDefinition(
                title="Test test_render_text",
                data=[
                    BlockDefinition(
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
            page_def=PageDefinition(
                title="Test test_render_html",
                data=[
                    BlockDefinition(
                        type="http://apicontext",
                        data={"url": "test", "name": "test"},
                        children=[
                            BlockDefinition(
                                type="default://text",
                                data={"text": "{{context.test.title}}"},
                            ),
                            BlockDefinition(
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
