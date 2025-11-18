import re
from pathlib import Path

import pandas as pd

# https://gdgarcia.ca/psl.html
local_dir = Path(__file__)
harper_core_dir = Path.joinpath(local_dir.parent.parent, Path("harper-core"))
annotations_portuguese_file = Path.joinpath(
    harper_core_dir, Path("annotations-portuguese.json")
)
annotations_file = Path.joinpath(harper_core_dir, Path("annotations.json"))
dictionary_file = Path.joinpath(harper_core_dir, Path("dictionary.dict"))
new_dictionary_file = Path.joinpath(harper_core_dir, Path("new_dictionary.dict"))

emphasis_table = pd.read_csv("psl.csv")

dictionary = None

with open(dictionary_file) as f:
    dictionary = f.readlines()


with open(new_dictionary_file, "w") as new_dict:
    # word count stays the same
    new_dict.write(dictionary[0] + "\n")

    for entry in dictionary[1:]:
        # ignore comments
        comment = None
        try:
            split = re.split("#[ ]*", entry)
            entry = split[0]
            comment = split[1].strip()
        except Exception:
            pass

        if len(entry) == 0:
            if comment:
                new_dict.write(f"# {comment}\n")
            continue

        # tries getting the word and properties, if failed then continue the loop
        try:
            s = re.split("/| ", entry)
            word = s[0]
            properties = s[1].strip()
        except Exception as e:
            print(e)
            continue
        if len(word) == 0:
            continue

        result = emphasis_table[emphasis_table["word"] == word]["stressLoc"]
        word_stressloc = result.iloc[0] if not result.empty else None
        print(f"word row is for word {word} is: {word_stressloc}")

        emphasis = None
        match word_stressloc:
            # oxitona
            case None:
                emphasis = "-"
            # oxitona
            case "final":
                emphasis = "-"
            # paroxitona
            case "penult":
                emphasis = "="
                pass
            # proparoxitona
            case "antepenult":
                emphasis = "+"
                pass

        if comment:
            new_dict.write(f"{word}/{properties}{emphasis} # {comment}\n")
        else:
            new_dict.write(f"{word}/{properties}{emphasis}\n")
