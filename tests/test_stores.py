import logging

from pe.renderer.renderer import Renderer
from pe.stores.db import DbStore
from pe.stores.http import HttpStore
from pe.types import Page, StoreConfig
from tests.base import TestCase

logger = logging.getLogger(__name__)


class TestStores(TestCase):
    def get_config(self):
        return super().get_config()

    async def test_http_store_apicontext(self):
        renderer = Renderer(config=self.get_config())
        http_store = renderer.store.get_store("http")
        self.assertIsNotNone(http_store)
        self.assertTrue(isinstance(http_store, HttpStore))
        logger.debug(
            "http_store=%s element_id=%s",
            http_store,
            [e.name for e in await http_store.get_widget_list()],
        )
        element = await http_store.get_widget_definition("apicontext")
        self.assertIsNotNone(element)

        context = await http_store.load_context(
            path="apicontext", data={"url": "test", "name": "test"}, context={}
        )
        self.assertIsNotNone(context)
        logger.debug("context=%s", context)

        html = await http_store.load_html(
            path="apicontext", data={"url": "test", "name": "test"}, context={}
        )
        logger.debug("html=%s", len(html))
        self.assertEqual(html, "@@children@@")

    async def test_http_store_embed(self):
        renderer = Renderer(config=self.get_config())
        http_store = renderer.store.get_store("http")
        self.assertIsNotNone(http_store)
        self.assertTrue(isinstance(http_store, HttpStore))
        logger.debug(
            "http_store=%s element_id=%s",
            http_store,
            [e.name for e in await http_store.get_widget_list()],
        )
        element = await http_store.get_widget_definition("embed")
        self.assertIsNotNone(element)

        html = await http_store.load_html(
            path="embed",
            data={"html_url": "https://example.com"},
            context={},
        )
        self.assertIsNotNone(html)
        logger.debug("html=%s", len(html))

        css = await http_store.load_css(
            path="embed",
            data={"css_url": "https://example.com"},
            context={},
        )
        self.assertIsNotNone(css)
        logger.debug("css=%s", len(css))

    async def test_store_page_list(self):
        renderer = Renderer(config=self.get_full_config())
        await renderer.store.delete_page_definition(
            "test_store_page_list", ok_if_not_found=True
        )

        pages = await renderer.store.get_page_list()
        logger.debug("pages=%s", pages)
        self.assertIsNotNone(pages)
        self.assertGreater(pages.count, 0)
        self.assertGreater(len(pages.results), 0)
        pre_count = pages.count

        # add a page to the db store
        db_store = renderer.store.get_store("default")
        self.assertIsNotNone(db_store)
        logger.debug("db_store=%s type=%s", db_store, type(db_store))
        self.assertTrue(isinstance(db_store, DbStore))
        await db_store.save_page_definition(
            path="test_store_page_list", data=Page(title="Test", url="", children=[])
        )

        pages = await renderer.store.get_page_list()
        logger.debug("pages=%s", pages)
        self.assertIsNotNone(pages)
        self.assertGreater(pages.count, pre_count)

        await renderer.store.delete_page_definition("test_store_page_list")
