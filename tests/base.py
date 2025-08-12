from pathlib import Path
from unittest import IsolatedAsyncioTestCase

from pe.config import Config
from pe.setup import setup_logging
from pe.types import Widget, StoreConfig

setup_logging()


class TestCase(IsolatedAsyncioTestCase):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    def get_config(self) -> Config:
        config = Config()
        config.stores = {
            "default": StoreConfig(
                name="default",
                type="file",
                path="builtin",
                tags=["jinja2", "widgets"],
                blocks=[
                    Widget(
                        name="text",
                        html="text/view.html",
                        css="text/style.css",
                    )
                ],
            ),
            "http": StoreConfig(name="http", type="http"),
        }
        return config

    def get_full_config(self) -> Config:
        config = Config.read(Path(__file__).parent.parent / "config.yaml")
        return config
