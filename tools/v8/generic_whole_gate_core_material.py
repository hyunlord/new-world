"""Two complete material-definition targets; no ordinary-world safety inference."""
import stat
from pathlib import Path
from typing import Final

from generic_gate import require, text
from generic_process import file_hash
from generic_sources import safe_path
from generic_whole_gate_core_material_pins import CONTRACTS, PINS
from mux2_gate_inputs import Json, Node

DATA: Final = 'rust/crates/sim-core/data/material/'
CATEGORIES: Final = ('stone', 'wood', 'animal', 'mineral', 'plant')


def classify_core_material(root: Path, inputs: dict[str, str], sections: list[Json]) -> list[Json]:
    """Authenticate every original case, defining source and dynamically loaded RON file."""
    expected = {name for name in PINS if name.startswith(DATA)}
    actual: set[str] = set()
    for category in CATEGORIES:
        directory = safe_path(root, DATA + category)
        require(stat.S_ISDIR(directory.lstat().st_mode), str(directory), 'material directory not regular')
        actual.update(path.relative_to(root).as_posix() for path in directory.iterdir()
                      if path.suffix == '.ron')
    require(actual == expected, '/material/data', 'material directory census changed')
    bindings: dict[str, Json] = {}
    for name, digest in PINS.items():
        path = safe_path(root, name)
        require(stat.S_ISREG(path.lstat().st_mode), name, 'material input not a regular file')
        require(inputs.get(name) == digest and file_hash(path) == digest,
                name, 'reviewed classification source changed or unbound')
        bindings[name] = digest
    result: list[Json] = []
    for target, source, names in CONTRACTS:
        matches = [Node(row) for row in sections if Node(row).field('package').value == 'sim-core'
                   and Node(row).field('target').value == target and Node(row).field('kind').value == 'test']
        require(len(matches) == 1, '/material/target', 'missing or duplicate reviewed target')
        row = matches[0]
        row.field('source').equals(source)
        cases = row.field('cases').array()
        observed = [text(case.field('name')) for case in cases]
        require(len(observed) == len(names) and set(observed) == set(names),
                '/material/cases', 'original reviewed case set differs')
        for case in cases:
            case.field('status').equals('ok')
            case.field('mode').equals(None)
        row.field('nested_passed').equals(0)
        result.append({'package': 'sim-core', 'target': target, 'kind': 'test', 'source': source,
                       'classification': 'MATERIAL_DEFINITIONS_NO_WORLD', 'cases': row.field('cases').value,
                       'source_pins': bindings, 'ordinary_safety': 'NOT_APPLICABLE_NO_WORLD',
                       'authority': 'SOURCE_QUALIFIED_ORIGINAL_AGGREGATE_ONLY', 'publication_authority': False})
    return result
