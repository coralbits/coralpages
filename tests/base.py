import logging
from pathlib import Path
from unittest import IsolatedAsyncioTestCase

from pe.config import Config
from pe.types import BlockDefinition, ElementDefinition, StoreConfig


class TestCase(IsolatedAsyncioTestCase):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        # make logging to log DEBUG in blue, warning in yellow, error in red
        logging.addLevelName(logging.DEBUG, "\033[94mDEBUG\033[0m")
        logging.addLevelName(logging.WARNING, "\033[93mWARNING\033[0m")
        logging.addLevelName(logging.ERROR, "\033[91mERROR\033[0m")
        logging.basicConfig(
            format="\033[94m[%(levelname)s\t]\033[0m \033[92m[%(name)24s]\033[0m %(message)s",
            level=logging.DEBUG,
        )
        ALLOWED_NAME_PREFIX = ["pe.", "tests."]
        for handler in logging.root.handlers:
            handler.addFilter(
                lambda record: any(
                    record.name.startswith(prefix) for prefix in ALLOWED_NAME_PREFIX
                )
            )

    def get_config(self) -> Config:
        config = Config()
        config.stores = {
            "default": StoreConfig(
                name="default",
                type="file",
                path="builtin",
                tags=["jinja2", "blocks"],
                blocks=[
                    ElementDefinition(
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
