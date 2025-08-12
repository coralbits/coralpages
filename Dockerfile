FROM ghcr.io/astral-sh/uv:bookworm-slim

WORKDIR /app


# Copy dependency files
COPY . .

# Remove venv if it exists
RUN rm -rf .venv

# Install Python dependencies
RUN uv sync

# Expose the port the app runs on
EXPOSE 8006

# Command to run the application
CMD ["./run.sh"]
