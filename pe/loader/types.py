"""
Loaders for the page editor.
"""

from pe.types import PageDefinition


class LoaderBase:
    """
    Interface for a loader.
    """

    def load(self, path: str) -> PageDefinition:
        """
        Load a page
        """
        raise NotImplementedError("Not implemented")
