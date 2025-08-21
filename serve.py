#!/usr/bin/env -S uv run --script

import argparse
import json
import traceback
import logging
import random

from pydantic import Field
import uvicorn
import fastapi
import fastapi.responses
import fastapi.middleware.cors

from pe.config import Config
from pe.renderer.renderer import Renderer
from pe.setup import setup_logging, trace_id_var
from pe.stores.factory import StoreFactory
from pe.types import Element, Page
from pe.renderer.renderer import RenderedPage
from typing import Annotated, Literal
import os
from typing import Union
from pe.stores.types import StoreBase

logger = logging.getLogger(__name__)


def create_app(args: argparse.Namespace):
    """
    Create the FastAPI app.
    """
    app = fastapi.FastAPI()  # type: ignore

    def prepare_config() -> Config:
        """
        Prepare the config for the renderer.
        """
        config = Config.read(args.config)
        return config

    config = prepare_config()
    store = StoreFactory(config=config)
    renderer = Renderer(config, store)

    # Add CORS middleware
    app.add_middleware(
        fastapi.middleware.cors.CORSMiddleware,
        allow_origins=config.server.allow_origins,
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    @app.middleware("http")
    async def set_trace_id(request: fastapi.Request, call_next):
        def trace_id():
            return f"{random.getrandbits(64):016x}"

        trace_id = request.headers.get("x-trace-id") or trace_id()
        request.state.trace_id = trace_id
        trace_id_var.set(trace_id)
        response = await call_next(request)
        trace_id_var.set(None)
        return response

    @app.get("/api/v1/page/")
    async def list_pages(
        offset: int = 0,
        limit: int = 10,
        type_: Annotated[
            Literal["template", "page"] | None, fastapi.Query(alias="type")
        ] = None,
        store_name: Annotated[
            str | None,
            fastapi.Query(alias="store"),
            Field(
                description="The store to use for rendering the page. Can use '|' to mark several in order."
            ),
        ] = None,
    ):
        """
        List of all pages. Can have filters:
        """
        filter = {}
        if type_:
            filter["type"] = type_
        if store_name:
            filter["store"] = store_name
        pages = await store.get_page_list(offset=offset, limit=limit, filter=filter)
        return fastapi.responses.Response(
            content=json.dumps(
                {
                    "count": pages.count,
                    "results": [page.to_dict() for page in pages.results],
                }
            ),
            media_type="application/json",
        )

    @app.get("/api/v1/page/{store_name:str}/{path:path}")
    async def read_page_json(request: fastapi.Request, store_name: str, path: str):
        page_store = store.get_store(store_name)
        if not page_store:
            return fastapi.responses.JSONResponse(
                {"content": "Store not found"}, status_code=404
            )
        page = await page_store.load_page_definition(path=path)
        if not page:
            return fastapi.responses.JSONResponse(
                {"content": "Page not found"}, status_code=404
            )
        page.url = f"{request.url.scheme}://{request.url.netloc}/api/v1/render/{store_name}/{path}"
        return fastapi.responses.Response(
            content=json.dumps(page.to_dict()), media_type="application/json"
        )

    @app.post("/api/v1/page/{store_name:str}/{page_name:path}")
    async def save_page_json(request: fastapi.Request, store_name: str, page_name: str):
        data = await request.json()
        path = f"{store_name}/{page_name}"
        page = Page.from_dict(data)
        await store.save_page_definition(path=path, data=page)
        return fastapi.responses.Response(content="OK", status_code=200)

    @app.delete("/api/v1/page/{store_name:str}/{page_name:path}")
    async def delete_page(store_name: str, page_name: str):
        path = f"{store_name}/{page_name}"
        try:
            if await store.delete_page_definition(path=path):
                return fastapi.responses.JSONResponse(
                    {"details": "ok"}, status_code=200
                )
            else:
                return fastapi.responses.JSONResponse(
                    {"details": "Failed - check logs"}, status_code=400
                )
        except NotImplementedError:
            return fastapi.responses.JSONResponse(
                {"details": "Not implemented"}, status_code=400
            )
        except Exception as e:
            logger.error(f"Failed to delete page {page_name}: {e}")
            return fastapi.responses.JSONResponse({"details": str(e)}, status_code=500)

    @app.get("/api/v1/widget/")
    async def list_known_widgets():
        widgets_dict = []
        for store_item in store.get_all_stores().values():
            for widget in await store_item.get_widget_list():
                eldef = widget.to_dict()
                eldef["store"] = store_item.config.name
                eldef["name"] = f"{store_item.config.name}/{widget.name}"
                widgets_dict.append(eldef)
        return fastapi.responses.Response(
            content=json.dumps(widgets_dict), media_type="application/json"
        )

    @app.get("/api/v1/widget/{widget_name}/html")
    async def read_widget_html(request: fastapi.Request, widget_name: str):
        data = dict(request.query_params)

        widget_name = f"builtin/{widget_name}"

        block = Element(
            type=widget_name,
            data=data,
            children=[],
            style={},
        )
        page = renderer.new_page()
        html, _ = await renderer.render_block(page, block)
        return fastapi.responses.Response(content=html, media_type="text/html")

    @app.get("/api/v1/widget/{widget_name}/css")
    async def read_widget_css(request: fastapi.Request, widget_name: str):
        data = dict(request.query_params)

        widget_name = f"builtin/{widget_name}"

        block = Element(
            type=widget_name,
            data=data,
            children=[],
            style={},
        )
        page = renderer.new_page()
        _, css = await renderer.render_block(page, block)
        return fastapi.responses.Response(content=css, media_type="text/css")

    @app.post("/api/v1/render/")
    async def render_page(
        request: fastapi.Request,
        format: Literal[None, "html", "json"] = fastapi.Query(None),
    ):
        if not config.server.render:
            return fastapi.responses.Response(content="Not enabled", status_code=500)

        data = await request.json()
        page = Page.from_dict(data)
        context = dict(request.query_params)
        rendered_page = await renderer.render(page, context=context)

        return render_page_by_format(rendered_page, format or "json")

    @app.get("/api/v1/render/{store:str}/{path:path}")
    async def read_page(
        request: fastapi.Request,
        store: str = fastapi.Path(
            description="The store to use for rendering the page. Can use '|' to mark several in order."
        ),
        path: str = fastapi.Path(description="The path to the page to render."),
        template: str | None = fastapi.Query(
            None,
            description="The template to use for rendering the page. "
            "Set to 'none' to avoid using templates. "
            "By default uses the one set at the page json.",
        ),
        format: Literal[None, "html", "json", "css", "js"] | None = fastapi.Query(None),
    ):
        """

        Renders a page from a given store, returning the rendered HTML content.

        If ?format=json, returns the page as JSON {body: str, head: {css: str[], js: str[], meta: str[]}}

        Formats:
        - html: Returns the full page as HTML.
        - json: Returns the page as JSON.
        - css: Returns all the page's CSS.
        - js: Returns all the page's JavaScript.

        Can also set the format indicating an extension.
        """

        if path.startswith("_") or ".." in path:
            return fastapi.responses.Response(content="Forbidden", status_code=403)

        if path == "":
            path = "index"

        logger.debug("Rendering page=%s/%s template=%s", store, path, template)

        format, path = guess_format(format, path, request)
        try:
            page: RenderedPage | None = await renderer.render_page(
                store=store, path=path, headers=request.headers, template=template
            )
        except Exception:  # pylint: disable=broad-exception-caught
            traceback.print_exc()
            if config.debug:
                exception_str = traceback.format_exc()
                return fastapi.responses.Response(
                    content=f"Internal Server Error: {exception_str}",
                    status_code=500,
                )
            else:
                return fastapi.responses.Response(
                    content="Internal Server Error", status_code=500
                )
        if not page:
            return fastapi.responses.Response(content="Page not found", status_code=404)

        return render_page_by_format(page, format)

    def render_page_by_format(
        page: RenderedPage, format: Literal["html", "json", "css", "js"]
    ) -> fastapi.responses.Response:
        if format == "html":
            return render_page_html(page)
        elif format == "json":
            return render_page_json(page)
        elif format == "css":
            return render_page_css(page)
        elif format == "js":
            return render_page_js(page)
        else:
            return fastapi.responses.Response(
                content="Unsupported format",
                status_code=400,
            )

    def guess_format(
        format: str | None, path: str, request: fastapi.Request
    ) -> tuple[Literal["html", "json", "css", "js"], str]:
        if format in ["html", "json", "css", "js"]:
            return format, path
        if "." in path:
            ext = os.path.splitext(path)[1]
            path = path[: -len(ext)]
            if ext == ".html":
                format = "html"
            elif ext == ".json":
                format = "json"
            elif ext == ".css":
                format = "css"
            elif ext == ".js":
                format = "js"

        if not format:
            content_type = request.headers.get("Accept", "text/html").split(",")[0]
            if content_type == "application/json":
                format = "json"
            elif content_type == "text/html":
                format = "html"
            elif content_type == "text/css":
                format = "css"
            elif content_type == "text/javascript":
                format = "js"
            else:
                logger.warning(
                    f"Unsupported content type: {content_type}, returning HTML"
                )
                format = "html"
        return format, path

    def render_page_html(page: RenderedPage) -> fastapi.responses.Response:
        return fastapi.responses.Response(
            content=str(page),
            media_type="text/html",
            status_code=page.response_code,
            headers=page.headers,
        )

    def render_page_json(page: RenderedPage) -> fastapi.responses.Response:
        return fastapi.responses.Response(
            content=json.dumps(page.to_dict()),
            media_type="application/json",
            status_code=page.response_code,
            headers=page.headers,
        )

    def render_page_css(page: RenderedPage) -> fastapi.responses.Response:
        return fastapi.responses.Response(
            content=str(page.get_css()),
            media_type="text/css",
            status_code=page.response_code,
            headers=page.headers,
        )

    def render_page_js(page: RenderedPage) -> fastapi.responses.Response:
        return fastapi.responses.Response(
            content=str(page.get_js()),
            media_type="text/javascript",
            status_code=page.response_code,
            headers=page.headers,
        )

    @app.get("/api/v1/store")
    def get_store_list(tags: str | None = None):
        tagsl = tags.split(",") if tags else []
        stores: list[StoreBase] = list(store.get_all_stores().values())

        for tag in tagsl:
            tag = tag.strip()
            stores = [store for store in stores if tag in store.config.tags]

        return fastapi.responses.JSONResponse(
            {
                "count": len(stores),
                "results": [
                    {"id": store.config.name, "name": store.config.name}
                    for store in stores
                ],
            }
        )

    @app.get("/")
    def redirect_to_index():
        return fastapi.responses.RedirectResponse(url="/api/v1/page/index")

    return app


def parse_args():
    """
    Parse the arguments.
    """
    parser = argparse.ArgumentParser(
        description="Serve pages using FastAPI with hexagonal architecture."
    )
    parser.add_argument(
        "other",
        help="Other arguments are ignored",
        nargs="*",
    )
    parser.add_argument(
        "--config",
        help="Path to the config file",
        default="config.yaml",
    )
    parser.add_argument(
        "--host", default="0.0.0.0", help="Host to bind to (default: 0.0.0.0)"
    )
    parser.add_argument(
        "--port", type=int, default=8000, help="Port to bind to (default: 8000)"
    )
    parser.add_argument(
        "--reload", action="store_true", help="Enable auto-reload on file changes"
    )
    parser.add_argument("--log-level", default="info", help="Log level")
    return parser.parse_args()


setup_logging()
app = create_app(parse_args())


def main():
    opts = parse_args()

    uvicorn.run(
        "serve:app",
        host=opts.host,
        port=opts.port,
        reload=opts.reload,
        log_level=opts.log_level,
    )


if __name__ == "__main__":
    main()
