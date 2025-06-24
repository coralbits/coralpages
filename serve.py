#!/usr/bin/env -S uv run --script

import argparse
import json
import logging
from pathlib import Path
import traceback
from typing import Any
import yaml
from pe.config import Config
from pe.renderer.renderer import Renderer
from pe.types import BlockDefinition, ElementDefinition
import uvicorn
from fastapi import FastAPI
from fastapi.responses import RedirectResponse, Response
from fastapi.requests import Request


def create_app(args: argparse.Namespace):
    """
    Create the FastAPI app.
    """
    app = FastAPI()  # type: ignore

    def prepare_config() -> Config:
        """
        Prepare the config for the renderer.
        """
        config = Config.read("config.yaml")
        config.page_path = Path(args.directory)
        return config

    config = prepare_config()

    @app.get("/api/v1/view/{page_name}")
    async def read_page(request: Request, page_name: str):
        if page_name.startswith("_") or ".." in page_name:
            return Response(content="Forbidden", status_code=403)

        if page_name == "index":
            page_name = "index"
        else:
            page_name = page_name

        try:
            renderer = Renderer(config)
            page = await renderer.render_page(page_name, headers=request.headers)
            return Response(
                content=str(page),
                media_type="text/html",
                status_code=page.response_code,
                headers=page.headers,
            )
        except Exception:  # pylint: disable=broad-exception-caught
            traceback.print_exc()
            if config.debug:
                exception_str = traceback.format_exc()
                return Response(
                    content=f"Internal Server Error: {exception_str}",
                    status_code=500,
                )
            else:
                return Response(content="Internal Server Error", status_code=500)

    @app.get("/api/v1/element/")
    def list_known_elements():
        elements_dict = {
            name: element.to_dict() for name, element in config.elements.items()
        }
        return Response(
            content=json.dumps(elements_dict), media_type="application/json"
        )

    @app.get("/api/v1/element/{element_name}/html")
    async def read_element_html(request: Request, element_name: str):
        data = dict(request.query_params)

        element_name = f"builtin://{element_name}"

        block = BlockDefinition(
            type=element_name,
            data=data,
            children=[],
            style={},
        )
        renderer = Renderer(config)
        page = renderer.new_page()
        html, _ = await renderer.render_block(page, block)
        return Response(content=html, media_type="text/html")

    @app.get("/api/v1/element/{element_name}/css")
    async def read_element_css(request: Request, element_name: str):
        data = dict(request.query_params)

        element_name = f"builtin://{element_name}"

        block = BlockDefinition(
            type=element_name,
            data=data,
            children=[],
            style={},
        )
        renderer = Renderer(config)
        page = renderer.new_page()
        _, css = await renderer.render_block(page, block)
        return Response(content=css, media_type="text/css")

    @app.get("/")
    def redirect_to_index():
        return RedirectResponse(url="/api/v1/view/index")

    return app


def parse_args():
    """
    Parse the arguments.
    """
    parser = argparse.ArgumentParser(
        description="Serve pages using FastAPI with hexagonal architecture."
    )
    parser.add_argument("directory", help="Directory containing YAML pages to serve")
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
