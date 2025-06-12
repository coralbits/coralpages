import logging
from unittest import TestCase
from pathlib import Path
from pe.renderer import Renderer
from pe.loader import ElementLoader
from pe.page import YamlLoader

logger = logging.getLogger(__name__)

class RendererTestCase(TestCase):
    def test_render(self):
        tmpl = YamlLoader().open(Path(__file__).parent / "example.yaml", "rt", encoding="utf-8")
        element_loader = ElementLoader()

        renderer = Renderer(tmpl, element_loader)
        res = renderer.render()
        logger.debug("%s", res)
        self.assertTrue(res.startswith("<h1>Welcome to My Website</h1>"))
