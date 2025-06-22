from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
from pathlib import Path
import uvicorn

from pe.services import PageService
from pe.adapters import FileSystemPageRepository, PePageRenderer


def create_app(page_service: PageService) -> FastAPI:
    """Create and configure the FastAPI application."""
    app = FastAPI(
        title="Page Editor Server",
        description="A hexagonal architecture server for serving pages",
        version="1.0.0",
    )

    @app.get("/api/list")
    async def list_root_pages():
        """List all pages in the root directory."""
        pages = page_service.list_pages()
        return {"pages": pages}

    @app.get("/api/list/{directory:path}")
    async def list_pages(directory: str):
        """List all pages in a directory."""
        clean_directory = directory.rstrip("/")
        pages = page_service.list_pages(clean_directory)
        return {"pages": pages}

    @app.get("/api/pages/{path:path}")
    async def get_page_data(path: str):
        """Get page data as JSON (for API access)."""
        clean_path = path.rstrip("/")

        page_data = page_service.get_page_data(clean_path)
        if page_data is None:
            raise HTTPException(status_code=404, detail=f"Page '{path}' not found")

        # Convert page data to dict for JSON response
        return {
            "title": page_data.title,
            "template": page_data.template,
            "data": page_data.data,
        }

    @app.get("/", response_class=HTMLResponse)
    async def serve_root():
        """Serve the root page (index.html)."""
        html_content = page_service.get_page_html("")
        if html_content is None:
            raise HTTPException(status_code=404, detail="Root page not found")
        return HTMLResponse(content=html_content)

    @app.get("/{path:path}", response_class=HTMLResponse)
    async def serve_page(path: str):
        """Serve a page by its path."""
        # Remove trailing slash for consistency
        clean_path = path.rstrip("/")

        html_content = page_service.get_page_html(clean_path)
        if html_content is None:
            raise HTTPException(status_code=404, detail=f"Page '{path}' not found")

        return HTMLResponse(content=html_content)

    return app


def create_page_service(base_directory: str) -> PageService:
    """Create a page service with file system storage."""
    page_repository = FileSystemPageRepository(base_directory)
    page_renderer = PePageRenderer()
    return PageService(page_repository, page_renderer)


def run_server(base_directory: str, host: str = "0.0.0.0", port: int = 8000):
    """Run the FastAPI server."""
    page_service = create_page_service(base_directory)
    app = create_app(page_service)

    print(f"Starting server on {host}:{port}")
    print(f"Serving pages from: {base_directory}")

    uvicorn.run(app, host=host, port=port)
