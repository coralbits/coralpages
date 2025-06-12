#!/usr/bin/env -S uv run --script
import argparse
from pe.page import YamlLoader
from pe.renderer import Renderer
from pe.loader import ElementLoader

def parse_args():
    parser = argparse.ArgumentParser(description='Process some integers.')
    parser.add_argument("input_yaml", help="Path to the YAML file")
    parser.add_argument("output_yaml", help="Path to the output HTML file")
    return parser.parse_args()

def main():
    args = parse_args()
    page = YamlLoader().open(args.input_yaml)
    element_loader = ElementLoader()

    html = Renderer(page, element_loader).render()
    if args.output_yaml == "-":
        print(html)
    else:
        with open(args.output_yaml, "wt", encoding="utf-8") as fd:
            fd.write(html)

if __name__ == "__main__":
    main()
