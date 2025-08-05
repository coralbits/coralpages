#!/usr/bin/env -S uv run --script

import argparse
import json
import traceback
import logging
import uuid

import uvicorn
import fastapi
import fastapi.responses
import fastapi.middleware.cors

from pe.config import Config
from pe.renderer.renderer import Renderer
from pe.setup import setup_logging, trace_id_var
from pe.stores.factory import StoreFactory
from pe.types import Element, Page

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
        trace_id = request.headers.get("x-trace-id") or uuid.uuid4().hex
        request.state.trace_id = trace_id
        trace_id_var.set(trace_id)
        response = await call_next(request)
        trace_id_var.set(None)
        return response

    @app.get("/api/v1/page/")
    async def list_pages(request: fastapi.Request):
        """
        List of all pages. Can have filters:

            - `offset`: Offset of the first page to return.
            - `limit`: Maximum number of pages to return.
            - `type`: `template`
        """
        offset = int(request.query_params.get("offset", 0))
        limit = int(request.query_params.get("limit", 10))
        pages = await store.get_page_list(offset=offset, limit=limit, filter={"type": "template"})
        return fastapi.responses.Response(
            content=json.dumps(
                {
                    "count": pages.count,
                    "results": [page.to_dict() for page in pages.results],
                }
            ),
            media_type="application/json",
        )

    @app.get("/api/v1/view/{page_name}")
    async def read_page(request: fastapi.Request, page_name: str, template: str | None = fastapi.Query(None), as_json: bool = fastapi.Query(False)):
        if page_name.startswith("_") or ".." in page_name:
            return fastapi.responses.Response(content="Forbidden", status_code=403)

        if page_name == "":
            page_name = "index"

        try:
            page = await renderer.render_page(page_name, headers=request.headers, template=template)
            if as_json:
                return fastapi.responses.Response(
                    content=json.dumps(page.to_dict()),
                    media_type="application/json",
                )
            else:
                return fastapi.responses.Response(
                    content=str(page),
                media_type="text/html",
                status_code=page.response_code,
                headers=page.headers,
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
                return fastapi.responses.Response(content="Internal Server Error", status_code=500)

    @app.get("/api/v1/page/{page_name}/json")
    async def read_page_json(request: fastapi.Request, page_name: str):
        page = await store.load_page_definition_all_stores(page_name)
        if not page:
            return fastapi.responses.JSONResponse({"content": "Page not found"}, status_code=404)
        page.url = (
            f"{request.url.scheme}://{request.url.netloc}/api/v1/view/{page_name}"
        )
        return fastapi.responses.Response(
            content=json.dumps(page.to_dict()), media_type="application/json"
        )

    @app.post("/api/v1/page/{page_name}/json")
    async def save_page_json(request: fastapi.Request, page_name: str):
        data = await request.json()
        store_name = data.get("store", "default")
        path = f"{store_name}://{page_name}"
        page = Page.from_dict(data)
        await store.save_page_definition(path=path, data=page)
        return fastapi.responses.Response(content="OK", status_code=200)

    @app.get("/api/v1/widget/")
    async def list_known_widgets():
        widgets_dict = []
        for store_item in store.get_all_stores().values():
            for widget in await store_item.get_widget_list():
                eldef = widget.to_dict()
                eldef["store"] = store_item.config.name
                eldef["name"] = f"{store_item.config.name}://{widget.name}"
                widgets_dict.append(eldef)
        return fastapi.responses.Response(content=json.dumps(widgets_dict), media_type="application/json")

    @app.get("/api/v1/widget/{widget_name}/html")
    async def read_widget_html(request: fastapi.Request, widget_name: str):
        data = dict(request.query_params)

        widget_name = f"builtin://{widget_name}"

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

        widget_name = f"builtin://{widget_name}"

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
    async def render_page(request: fastapi.Request):
        if not config.server.render:
            return fastapi.responses.Response(content="Not enabled", status_code=500)

        data = await request.json()
        page = Page.from_dict(data)
        page = await renderer.render(page)
        return fastapi.responses.Response(
            content=str(page),
            media_type="text/html",
            status_code=page.response_code,
            headers=page.headers,
        )

    @app.get("/")
    def redirect_to_index():
        return fastapi.responses.RedirectResponse(url="/api/v1/view/index")

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
