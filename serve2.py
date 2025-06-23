#!/usr/bin/env -S uv run --script

import argparse
import json
from pathlib import Path
from typing import Any
import yaml
from pe2.renderer.renderer import Renderer
from pe2.types import BlockDefinition, ElementDefinition
import uvicorn
from fastapi import FastAPI
from fastapi.responses import RedirectResponse, Response
from fastapi.requests import Request


def create_app(args: argparse.Namespace):
    """
    Create the FastAPI app.
    """
    app = FastAPI()  # type: ignore

    def prepare_config() -> dict[str, Any]:
        """
        Prepare the config for the renderer.
        """
        with open("config.yaml", "rt", encoding="utf-8") as fd:
            config_dict = yaml.safe_load(fd)

        return {
            "page_path": Path(args.directory),
            "elements": {
                element["name"]: ElementDefinition.from_dict(element)
                for element in config_dict["elements"]
            },
        }

    config = prepare_config()

    @app.get("/api/v1/view/{page_name}")
    async def read_root(page_name: str):
        if page_name == "index":
            page_name = "builtin://index.yaml"
        else:
            page_name = f"builtin://{page_name}.yaml"

        renderer = Renderer(config)
        page = await renderer.render_page(page_name)
        return Response(content=str(page), media_type="text/html")

    @app.get("/api/v1/element/")
    def list_known_elements():
        elements_dict = {
            name: element.to_dict() for name, element in config["elements"].items()
        }
        return Response(
            content=json.dumps(elements_dict), media_type="application/json"
        )

    @app.get("/api/v1/element/{element_name}/html")
    async def read_element_html(request: Request, element_name: str):
        data = request.query_params

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
        data = request.query_params

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
    return parser.parse_args()


app = create_app(parse_args())


def main():
    opts = parse_args()
    uvicorn.run("serve2:app", host=opts.host, port=opts.port, reload=opts.reload)


if __name__ == "__main__":
    main()
