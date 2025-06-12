import yaml
from pe.types import Page

class YamlLoader:
    def open(self, path, mode="rt", encoding="utf-8") -> Page:
        with open(path, mode, encoding=encoding) as fd:
            data = yaml.safe_load(fd)
        return Page.__from_dict__(data)
