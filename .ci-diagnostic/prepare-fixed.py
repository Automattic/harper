import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
EXPECTED_BASE = 'b31ee25e1c9bd94395a0c3449ad5185d18e48970'
OUTER_SHA = '7f10ac81f40f4b0936b42b83ee485f33109f7f6a2d9732073e9c47bf0fd39a34'
INNER_SHA = 'b31ef8e146809f6c3e81bd23936c18b71827b4acf351bcca48798a5594a82042'
archive = Path(os.environ['CI_DIAGNOSTIC_ARCHIVE']).read_bytes()
assert hashlib.sha256(archive).hexdigest() == OUTER_SHA
with zipfile.ZipFile(io.BytesIO(archive)) as outer:
    assert outer.namelist() == ['harper-chrome-plugin.zip']
    payload = outer.read('harper-chrome-plugin.zip')
assert hashlib.sha256(payload).hexdigest() == INNER_SHA
build = ROOT / 'packages/chrome-plugin/build'
build.mkdir(parents=True, exist_ok=False)
with zipfile.ZipFile(io.BytesIO(payload)) as inner:
    for name in inner.namelist():
        target = (build / name).resolve()
        assert target.is_relative_to(build.resolve()), name
    inner.extractall(build)
expected = json.loads((ROOT / '.ci-diagnostic/extension-hashes.json').read_text())
actual = {str(p.relative_to(build)): hashlib.sha256(p.read_bytes()).hexdigest() for p in build.rglob('*') if p.is_file()}
assert actual == expected
changed = subprocess.check_output(['git', 'diff', '--name-only', EXPECTED_BASE, 'HEAD'], cwd=ROOT, text=True).splitlines()
repairs = json.loads((ROOT / '.ci-diagnostic/repair-hashes.json').read_text())
allowed = set(repairs) | {'.github/workflows/browser_failure_diagnostic.yml', 'packages/chrome-plugin/ci-diagnostic.config.ts', 'packages/chrome-plugin/tests/fixtures.ts'}
assert all(p.startswith('.ci-diagnostic/') or p in allowed for p in changed), changed
for name, digest in repairs.items():
    assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest, name
for p in ['packages/chrome-plugin/tests/iframe_domain_inheritance.spec.ts', 'packages/chrome-plugin/tests/iframe_popup_dismiss.spec.ts', 'packages/chrome-plugin/tests/simple_textarea.spec.ts', 'packages/chrome-plugin/tests/prosemirror.spec.ts', 'packages/chrome-plugin/tests/lexical_webcomponent.spec.ts']:
    assert (ROOT / p).read_bytes() == subprocess.check_output(['git', 'show', f'{EXPECTED_BASE}:{p}'], cwd=ROOT)
output = ROOT / 'packages/chrome-plugin/diagnostic-output'
output.mkdir(parents=True, exist_ok=True)
(output / 'provenance.json').write_text(json.dumps({'source_base': EXPECTED_BASE, 'diagnostic_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'artifact_id': 14001621916, 'artifact_run': 43344688903, 'outer_sha256': OUTER_SHA, 'inner_sha256': INNER_SHA, 'extension_files': actual, 'diagnostic_changed_files': changed, 'repair_files': repairs}, indent=2))
print(f'Verified {len(actual)} extension files and unchanged production source/specs.')
