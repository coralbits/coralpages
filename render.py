#!/usr/bin/env -S uv run --script

import argparse
from pathlib import Path
import sys
from contextlib import contextmanager
from typing import TextIO
import logging

import yaml

from pe.config import Config
from pe.renderer.renderer import Renderer
from pe.types import PageDefinition

logging.basicConfig(level=logging.DEBUG)


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=str)
    parser.add_argument("output", type=str, default="-")
    return parser.parse_args()


async def main():
    args = parse_args()

    with open_input(args.input) as finput, open_output(args.output) as foutput:
        await render(args, finput, foutput)


@contextmanager
def open_input(path: str) -> TextIO:
    """
    Open the input file.
    """
    if path == "-":
        yield sys.stdin
    else:
        with open(path, "rt", encoding="utf-8") as fd:
            yield fd


@contextmanager
def open_output(path: str) -> TextIO:
    """
    Open the output file.
    """
    if path == "-":
        yield sys.stdout
    else:
        with open(path, "wt", encoding="utf-8") as fd:
            yield fd


def prepare_config(input_path: str) -> Config:
    """
    Prepare the config for the renderer.
    """
    config = Config.read("config.yaml")
    config.page_path = Path(input_path).parent
    return config


async def render(args, finput: TextIO, foutput: TextIO):
    """
    Render the input file to the output file.
    """
    renderer = Renderer(prepare_config(args.input))

    page = PageDefinition.from_dict(yaml.safe_load(finput))
    output = await renderer.render(page)
    foutput.write(str(output))


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
