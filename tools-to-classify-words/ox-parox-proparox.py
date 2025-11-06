from pathlib import Path

# https://gdgarcia.ca/psl.html
local_dir = Path(__file__)
harper_core_dir = Path.joinpath(local_dir.parent.parent, Path("harper-core"))
