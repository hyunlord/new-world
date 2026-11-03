# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate
"""Closed original gate output paths; shared mutable arm logs are not rebaselined."""
from pathlib import Path
from typing import Final

from generic_process import file_hash
from generic_whole_gate_journal import anchored, snapshot
from mux2_gate_inputs import Json, Node

OUTPUTS: Final = {
    9: ('smoke',), 10: ('identity',), 11: ('determinism_probe',), 12: ('perf',),
    17: ('m1-trf.log',), 18: ('colony_determinism',), 19: ('replay',),
    23: ('renderer_agreement',), 26: ('soak',), 27: ('nav_bench',),
    28: ('knowledge_f10', 'knowledge_f10_control'), 29: ('story',), 30: ('neighbours',),
    32: ('resolver_godot.json',),
    34: ('mart2_ground_test.out', 'mart2_people_test.out', 'mart21_trees_test.out'),
    35: ('steward',), 36: ('data_only_ext',), 42: ('core-registration-tests.log',),
    43: ('core-data-only-tests.log',), 37: ('mux2-defence',), 38: ('mux2-oracles',),
    39: ('mux2-visibility',), 40: ('mux2-why',), 41: ('mux2-stall',),
    46: ('mux21-toast.log', 'mux21-layout.log', 'mux21-why-en.log', 'mux21-why-ko.log'),
}


def retain(session: Node, arm: str, step: int) -> tuple[dict[str, Json], dict[str, str], dict[str, int]]:
    """Freeze named producer outputs before any collector callback; absent is not success."""
    root = Path(session.field('output').text())/arm
    anchors: dict[str, Json] = {}
    files: dict[str, str] = {}
    modes: dict[str, int] = {}
    for relative in OUTPUTS.get(step, ()):
        path = root/relative
        if not path.exists():
            anchors[str(path)] = {'kind': 'ABSENT'}
        elif path.is_dir():
            original = anchored(path)
            anchors[str(path)] = {'kind': 'DIRECTORY', **original}
            content, metadata = snapshot(path)
            files.update(content)
            modes.update(metadata)
        else:
            digest = file_hash(path)
            mode = path.lstat().st_mode
            anchors[str(path)] = {'kind': 'FILE', 'sha256': digest, 'mode': mode}
            files[str(path)] = digest
            modes[str(path)] = mode
    return anchors, files, modes
