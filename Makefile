test:
	uv run -m unittest discover -s pe/tests -p "test_*.py"

serve:
	uv run serve.py docs --port 8006 --reload --log-level=debug
