"""
HTTP store implementation.
"""

from functools import lru_cache
import logging
from typing import Any
import httpx

from pe.stores.types import StoreBase
from pe.types import ElementDefinition, FieldDefinition, StoreConfig

logger = logging.getLogger(__name__)


class HttpElementGeneric:
    """
    A generic HTTP element.
    """

    def __init__(
        self, config: StoreConfig, data: dict[str, Any], context: dict[str, Any]
    ):
        self.config = config
        self.data = data
        self.context = context

    async def load_html(self) -> str | None:
        """
        Load the HTML for the HTTP element.
        """
        return "@@children@@"

    async def load_css(self) -> str | None:
        """
        Load the CSS for the HTTP element.
        """
        return None

    async def load_context(self) -> dict | None:
        """
        Load the generic data for the HTTP element.
        """
        return None

    @lru_cache(maxsize=100)
    async def load_generic(self, *, url: str) -> str | None:
        """
        Load a generic file from the HTTP store.
        """
        logger.debug("Loading %s", url)

        for allowed_prefix in self.config.get("allowed_prefixes", []):
            if not url.startswith(allowed_prefix):
                raise ValueError(f"URL {url} is not allowed to be loaded")

        method = self.data.get("method", "GET")
        data = self.data.get("query_params", "").split("\n")
        data = {k: v for k, v in [line.split("=") for line in data if line]}

        if method == "GET":
            logger.debug("Loading url=%s method=%s params=%s", url, method, data)
            response = await httpx.AsyncClient().get(url, params=data)
        elif method == "POST":
            logger.debug("Loading url=%s method=%s data=%s", url, method, data)
            response = await httpx.AsyncClient().post(url, json=data)
        else:
            raise ValueError(f"Invalid method: {method}")

        if response.status_code != 200:
            raise ValueError(f"Failed to load {url}: {response.status_code}")
        return response


class HttpStore(StoreBase):
    """
    HTTP-based store implementation.
    """

    def get_element_impl(
        self, name: str, data: dict[str, Any], context: dict[str, Any]
    ) -> HttpElementGeneric:
        """
        Get the implementation for an element.
        """
        if name == "apicontext":
            return HttpApiContext(self.config, data, context)
        elif name == "embed":
            return HttpEmbed(self.config, data, context)
        raise ValueError(f"Unknown element: {name}")

    async def load_html(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a page from the HTTP store.
        """
        element = self.get_element_impl(path, data, context)
        return await element.load_html()

    async def load_css(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> str | None:
        """
        Load a CSS file from the HTTP store.
        """
        element = self.get_element_impl(path, data, context)
        return await element.load_css()

    async def load_context(
        self, *, path: str, data: dict[str, Any], context: dict[str, Any]
    ) -> dict[str, Any] | None:
        """
        Load the context for the HTTP store.
        """
        logger.debug(
            "Loading context for path=%s data=%s context=%s", path, data, context
        )
        element = self.get_element_impl(path, data, context)
        return await element.load_context()

    async def get_element_list(self) -> list[ElementDefinition]:
        """
        Load the element list from the HTTP store.
        """
        return [
            ElementDefinition(
                name="apicontext",
                description="Point to a JSON API endpoint, and children can use the data to render. It can itself use some context data as well.",
                icon="api",
                children=True,
                editor=[
                    FieldDefinition(
                        type="text",
                        label="Variable Name",
                        placeholder="The name of the context variable to insert/replace.",
                        name="name",
                    ),
                    FieldDefinition(
                        type="text",
                        label="URL",
                        name="url",
                        placeholder="Enter URL | 'test' for test data",
                    ),
                    FieldDefinition(
                        type="select",
                        label="Method",
                        name="method",
                        options=["GET", "POST", "PUT", "DELETE"],
                    ),
                ],
            ),
            ElementDefinition(
                name="embed",
                description="Embed a URL, and children can use the data to render. It can itself use some context data as well.",
                icon="link",
                children=True,
                editor=[
                    FieldDefinition(
                        type="text",
                        label="HTML URL",
                        name="html_url",
                        placeholder="Enter URL",
                    ),
                    FieldDefinition(
                        type="text",
                        label="CSS URL",
                        name="css_url",
                        placeholder="Enter URL",
                    ),
                    FieldDefinition(
                        type="textarea",
                        label="Query Params",
                        name="query_params",
                        placeholder="Enter query params, one per line with key=value",
                    ),
                    FieldDefinition(
                        type="select",
                        label="Method",
                        name="method",
                        options=["GET", "POST", "JSON"],
                    ),
                ],
            ),
        ]


class HttpApiContext(HttpElementGeneric):
    """
    A HTTP API context element.
    """

    async def load_context(self) -> dict | None:
        """
        Load the generic data for the HTTP API context element.
        """
        url = self.data.get("url")
        if not url:
            logger.warning("No URL `url` for API context")
            return None
        variable = self.data.get("name")
        if not variable:
            logger.warning("No variable `name` for API context")
            return None

        if url == "test":
            return {
                variable: {
                    "array": [
                        {"id": 1, "name": "test1"},
                        {"id": 2, "name": "test2"},
                        {"id": 3, "name": "test3"},
                    ],
                    "title": "Test JSON Data",
                }
            }

        response = await self.load_generic(url=url)

        resjson = response.json()
        logger.debug(
            "Loading API context url=%s variable=%s response=%s",
            url,
            variable,
            resjson,
        )
        if response.status_code == 200:
            return {variable: resjson}

        raise ValueError(f"Failed to load API context: {response.status_code}")


class HttpEmbed(HttpElementGeneric):
    """
    A HTTP embed element.
    """

    async def load_html(self) -> str | None:
        """
        Load the HTML for the HTTP embed element.
        """
        if not self.data.get("html_url"):
            return None
        response = await self.load_generic(url=self.data["html_url"])
        return response.text

    async def load_css(self) -> str | None:
        """
        Load the CSS for the HTTP embed element. -- Nothing to do here.
        """
        if not self.data.get("css_url"):
            return None
        response = await self.load_generic(url=self.data["css_url"])
        return response.text
