#!/usr/bin/env -S uv run --script

"""
Test script for async HTTP renderer.
"""

import asyncio
from pathlib import Path

import yaml
from pe.renderer.renderer import Renderer
from pe.types import ElementDefinition


async def test_async_http_renderer():
    """Test the async HTTP renderer."""

    # Load config
    with open("config.yaml", "rt", encoding="utf-8") as fd:
        config_dict = yaml.safe_load(fd)

    config = {
        "page_path": Path("docs"),
        "elements": {
            element["name"]: ElementDefinition.from_dict(element)
            for element in config_dict["elements"]
        },
    }

    # Create renderer
    renderer = Renderer(config)

    # Test async rendering
    print("Testing async HTTP renderer...")
    try:
        # This will test the async renderer with any HTTP elements
        result = await renderer.render_page("builtin://index.yaml")
        print("✅ Async rendering successful!")
        print(f"Result type: {type(result)}")
        print(f"Result title: {result.title}")
        print(f"Content length: {len(result.content)} characters")
        return True
    except Exception as e:
        print(f"❌ Async rendering failed: {e}")
        return False


if __name__ == "__main__":
    success = asyncio.run(test_async_http_renderer())
    exit(0 if success else 1)
