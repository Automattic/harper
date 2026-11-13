"""Remove only the Lexical synchronization changes in the selected arm."""
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
BASE = 'b31ee25e1c9bd94395a0c3449ad5185d18e48970'
mode = os.environ['CI_DIAGNOSTIC_MODE']
selection = os.environ['CI_DIAGNOSTIC_SELECTION']
assert mode in ('traced', 'untraced')
assert selection in ('synchronized', 'original')
repairs = json.loads((ROOT / '.ci-diagnostic/repair-hashes.json').read_text())
for name, digest in repairs.items():
    assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest


def baseline(name):
    return subprocess.check_output(['git', 'show', f'{BASE}:{name}'], cwd=ROOT)


selection_files = {
    'packages/chrome-plugin/tests/lexical.spec.ts',
    'packages/chrome-plugin/tests/testUtils.ts',
    'packages/chrome-plugin/tests/pages/lexical.html',
}
fixture = 'packages/chrome-plugin/tests/fixtures.ts'
expected = dict(repairs)
if selection == 'original':
    for name in sorted(selection_files):
        data = baseline(name)
        (ROOT / name).write_bytes(data)
        expected[name] = hashlib.sha256(data).hexdigest()
if mode == 'untraced':
    (ROOT / fixture).write_bytes(baseline(fixture))
expected[fixture] = hashlib.sha256((ROOT / fixture).read_bytes()).hexdigest()
if mode == 'untraced':
    assert (ROOT / fixture).read_bytes() == baseline(fixture)
else:
    committed = subprocess.check_output(['git', 'show', f'HEAD:{fixture}'], cwd=ROOT)
    assert (ROOT / fixture).read_bytes() == committed
for name, digest in expected.items():
    assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest
actual_changes = subprocess.check_output(['git', 'diff', '--name-only'], cwd=ROOT, text=True).splitlines()
allowed_changes = (selection_files if selection == 'original' else set()) | ({fixture} if mode == 'untraced' else set())
assert set(actual_changes) == allowed_changes
out = ROOT / 'packages/chrome-plugin/diagnostic-output/runtime-selection.json'
out.write_text(json.dumps({'selection': selection, 'mode': mode, 'workers': 1, 'files': expected, 'runtime_changed_files': actual_changes, 'cpu_count': os.cpu_count()}, indent=2))
print(f'Verified runtime mode: {selection}/{mode}; single-worker repair retained in all arms.')
