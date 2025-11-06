import json
import re
from pathlib import Path

local_dir = Path(__file__)
harper_core_dir = Path.joinpath(local_dir.parent.parent, Path("harper-core"))
annotations_portuguese_file = Path.joinpath(
    harper_core_dir, Path("annotations-portuguese.json")
)
annotations_file = Path.joinpath(harper_core_dir, Path("annotations.json"))
dictionary_file = Path.joinpath(harper_core_dir, Path("dictionary.dict"))


backup = None
dictionary = None

new = {"affixes": {}, "properties": {}}
with open(annotations_file) as f:
    backup = json.load(f)

with open(dictionary_file) as f:
    dictionary = f.readlines()

for entry in dictionary:
    entry = entry.split("#")[0]
    if len(entry) == 0:
        continue

    try:
        properties = re.split("/| ", entry)[1]
    except Exception as e:
        print(e)
        continue
    print(f"properties are {properties}")
    for property in properties:
        result_properties = backup["properties"].get(property)
        result_affixes = backup["affixes"].get(property)

        if result_properties:
            new["properties"][property] = result_properties
        elif result_affixes:
            new["affixes"][property] = result_affixes
        else:
            print("could not find property ", property)

with open(annotations_portuguese_file, "w") as f:
    json.dump(new, f)
