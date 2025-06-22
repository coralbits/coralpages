test:
	uv run -m unittest discover -s pe/tests -p "test_*.py"

serve:
	uv run serve.py test_pages --port 8001 --reload
