#!/usr/bin/env -S uv run --script
import argparse
import sys
import os
from pathlib import Path
import uvicorn


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
    parser.add_argument(
        "--reload", action="store_true", help="Enable auto-reload on file changes"
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
        # Set the base directory as an environment variable
        os.environ["PAGE_EDITOR_BASE_DIR"] = args.directory

        print(f"Starting server on {args.host}:{args.port}")
        print(f"Serving pages from: {args.directory}")
        if args.reload:
            print("Auto-reload enabled - server will restart on file changes")

        # Use the server module directly
        uvicorn.run("pe.server:app", host=args.host, port=args.port, reload=args.reload)
    except KeyboardInterrupt:
        print("\nServer stopped by user")
    except Exception as e:
        print(f"Error starting server: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
