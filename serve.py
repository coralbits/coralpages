#!/usr/bin/env -S uv run --script
import argparse
import sys
from pathlib import Path

from pe.api import run_server


def parse_args():
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
    return parser.parse_args()


def validate_directory(directory: str) -> bool:
    """Validate that the directory exists and contains YAML files."""
    dir_path = Path(directory)

    if not dir_path.exists():
        print(f"Error: Directory '{directory}' does not exist", file=sys.stderr)
        return False

    if not dir_path.is_dir():
        print(f"Error: '{directory}' is not a directory", file=sys.stderr)
        return False

    # Check if there are any YAML files or index.yaml files
    yaml_files = list(dir_path.glob("*.yaml"))
    index_files = list(dir_path.glob("*/index.yaml"))

    if not yaml_files and not index_files:
        print(
            f"Warning: Directory '{directory}' contains no YAML files", file=sys.stderr
        )
        print("Expected files: *.yaml or */index.yaml", file=sys.stderr)

    return True


def main():
    args = parse_args()

    if not validate_directory(args.directory):
        sys.exit(1)

    try:
        run_server(base_directory=args.directory, host=args.host, port=args.port)
    except KeyboardInterrupt:
        print("\nServer stopped by user")
    except Exception as e:
        print(f"Error starting server: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
