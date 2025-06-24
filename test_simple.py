#!/usr/bin/env python3
"""
Simple test to verify configuration loading works.
"""

from pe.config import Config
from pe.loader.factory import LoaderRoot


def test_config():
    """
    Test configuration loading.
    """
    print("Testing configuration loading...")

    # Load configuration
    config = Config.read("config.yaml")
    print(f"✅ Loaded config with {len(config.stores)} stores")

    # Test loader creation
    loader = LoaderRoot(config)
    print("✅ Created LoaderRoot")

    # Test element loading
    try:
        element = loader.load_element("text")
        print(f"✅ Loaded element 'text' from store '{element.store}'")
        print(f"   Tags: {element.tags}")
    except Exception as e:
        print(f"❌ Error loading element: {e}")
        return False

    # Test page loading
    try:
        page = loader.load("index")
        print(f"✅ Loaded page '{page.title}'")
        return True
    except Exception as e:
        print(f"❌ Error loading page: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = test_config()
    if success:
        print("✅ Configuration test passed!")
    else:
        print("❌ Configuration test failed!")
        exit(1)
