#!/usr/bin/env python3
"""
Simple test script to verify the new store-based system works.
"""

import asyncio
from pe.config import Config
from pe.renderer.renderer import Renderer


async def test_system():
    """
    Test the new store-based system.
    """
    print("Testing the new store-based system...")

    # Load configuration
    config = Config.read("config.yaml")
    print(f"Loaded config with {len(config.stores)} stores:")
    for store_name, store_config in config.stores.items():
        print(f"  - {store_name}: {store_config.type} ({', '.join(store_config.tags)})")

    print(f"Loaded {len(config.elements)} elements:")
    for element_name, element in config.elements.items():
        print(f"  - {element_name}: store={element.store}, tags={element.tags}")

    # Create renderer
    renderer = Renderer(config)

    # Test loading a page
    try:
        page = await renderer.render_page("index")
        print(f"Successfully rendered page: {page.title}")
        print(f"Page has {len(page.content)} characters")
        return True
    except Exception as e:
        print(f"Error rendering page: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = asyncio.run(test_system())
    if success:
        print("✅ System test passed!")
    else:
        print("❌ System test failed!")
        exit(1)
