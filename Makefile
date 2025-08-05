.PHONY: help test test-watch serve

help:
	@echo "Usage:"
	@echo "       make serve"
	@echo "       make fmt"
	@echo "       make test [TEST=test_name]"
	@echo "       make test-watch [TEST=test_name]"

# can set TEST=test_name to run a single test
test:
	uv run -m unittest $(TEST)


test-watch:
	while true; do \
		uv run -m unittest $(TEST); \
		sleep 1; \
		inotifywait -e modify -e create -e delete -e move -r .; \
	done

serve:
	uv run serve.py docs --port 8006 --reload --log-level=debug

fmt:
	uvx ruff format .
