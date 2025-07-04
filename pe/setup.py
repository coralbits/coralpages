"""
Setup logging for the application.
"""

import logging


def setup_logging():
    """
    Setup logging for the application.
    """
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
