"""
Setup logging for the application.
"""

import contextvars
import logging


# Context variable to store trace_id across async calls
trace_id_var: contextvars.ContextVar[str | None] = contextvars.ContextVar(
    "trace_id", default=None
)


class TraceIdFilter(logging.Filter):
    """Filter to add trace_id to log records if available."""

    def filter(self, record):
        trace_id = trace_id_var.get(None)
        if trace_id:
            record.trace_id = trace_id
        return True


def setup_logging():
    """
    Setup logging for the application.
    """
    # make logging to log DEBUG in blue, warning in yellow, error in red
    logging.addLevelName(logging.DEBUG, "\033[94mDEBUG\033[0m")
    logging.addLevelName(logging.WARNING, "\033[93mWARN\033[0m")
    logging.addLevelName(logging.ERROR, "\033[91mERROR\033[0m")
    logging.basicConfig(
        format="\033[94m[%(levelname)s\t]\033[0m \033[92m[%(name)24s]\033[0m%(trace_id)s %(message)s",
        level=logging.DEBUG,
    )
    ALLOWED_NAME_PREFIX = ["pe.", "tests.", "server"]
    trace_filter = TraceIdFilter()

    for handler in logging.root.handlers:
        handler.addFilter(trace_filter)
        handler.addFilter(
            lambda record: any(
                record.name.startswith(prefix) for prefix in ALLOWED_NAME_PREFIX
            )
        )

    # Custom formatter to conditionally show trace_id
    class TraceFormatter(logging.Formatter):
        def format(self, record):
            if hasattr(record, "trace_id") and record.trace_id:
                record.trace_id = f" \033[96m[trace_id={record.trace_id}]\033[0m"
            else:
                record.trace_id = ""
            return super().format(record)

    formatter = TraceFormatter(
        "\033[94m[%(levelname)s\t]\033[0m \033[92m[%(name)24s]\033[0m%(trace_id)s %(message)s"
    )
    for handler in logging.root.handlers:
        handler.setFormatter(formatter)
