import yaml
from pe.types import ElementDefinition

config = yaml.safe_load(open('config.yaml'))
element_definitions = {
    element["name"]:    ElementDefinition(**element)
    for element in config['elements']
    }
