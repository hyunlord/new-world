# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Exact original migration fixture selectors and their bounded external file closure."""
from pathlib import Path
from typing import Final

from generic_gate import require
from mux2_gate_fixtures import validate_fixtures

from generic_process import file_hash, real_path

FIXTURE_ENV: Final = frozenset({'WORLDSIM_MUX_FIXTURES', 'WORLDSIM_B5_LEGACY9',
                               'MUX2_V10_FIXTURES', 'MUX2_V10_MANIFEST'})
LEGACY_FILES: Final = (
    'historical/m55/tick-2000.save', 'historical/m55/tick-3000.save',
    'historical/mart2/tick-2000.save', 'historical/mart2/tick-3000.save',
    'base/on-tick-5000.save', 'base/on-tick-6000.save',
    'core8/on-tick-5000.save', 'core8/on-tick-6000.save',
    'b4/h-format9-fixture/tick4000.save', 'historical/acquisition-receipt.json',
    'historical/m55/source-commit.txt', 'historical/mart2/source-commit.txt',
    'historical/m55/continuation-2000-3000.json', 'historical/mart2/continuation-2000-3000.json',
    'base/provenance.json', 'core8/acquisition-receipt.json', 'b4/h-format9-fixture/receipt.json',
    'base/source-sha256.json', 'b4/BPTH/source-freeze.json')


def fixture_paths(environment: dict[str, str]) -> tuple[Path, ...]:
    require(FIXTURE_ENV <= environment.keys(), '/fixtures', 'incomplete original fixture selectors')
    legacy = Path(environment['WORLDSIM_MUX_FIXTURES'])
    v10 = Path(environment['MUX2_V10_FIXTURES'])
    for directory in (legacy, v10):
        real_path(directory, directory=True)
    require(environment['WORLDSIM_B5_LEGACY9'] == str(legacy/'b4/h-format9-fixture/tick4000.save'),
            '/fixtures', 'B5 input differs from named original inventory')
    manifest = Path(environment['MUX2_V10_MANIFEST'])
    real_path(manifest, directory=False)
    files = tuple(sorted(path for path in v10.iterdir() if path.suffix in {'.save', '.wsave'}))
    require(len(files) == 4, '/fixtures', 'original format10 producer requires four saves')
    return tuple(legacy/name for name in LEGACY_FILES) + (manifest, *files)


def validate_fixture_inputs(environment: dict[str, str], external: dict[str, str]) -> None:
    if not FIXTURE_ENV & environment.keys():
        return
    paths = fixture_paths(environment)
    for path in paths:
        require(str(path) in external and file_hash(path) == external[str(path)],
                '/fixtures', 'original fixture or provenance missing/changed: '+str(path))
    validate_fixtures(Path(environment['MUX2_V10_FIXTURES']), Path(environment['MUX2_V10_MANIFEST']))
    for path in paths:
        require(file_hash(path) == external[str(path)], '/fixtures', 'fixture changed during admission')
