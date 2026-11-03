# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate
"""Source-owned sequential step publication and immutable returned output anchors."""
import os
import stat
from pathlib import Path
from typing import Final

from generic_gate import document, require
from mux2_gate_inputs import Json, Node

from generic_process import atomic_json, file_hash

ORDER: Final = (*range(1, 37), 42, 43, *range(37, 42), *range(44, 52))
ACCEPTANCE: Final = frozenset({25, 44, 45, 47, 48, 50})
REFERENCE: Final = tuple(n for n in ORDER if n not in ACCEPTANCE | {1, 6, 7, 8, 13, 14, 15, 16, 20, 22, 31, 32, 33, 51})


def snapshot(root: Path) -> tuple[dict[str, str], dict[str, int]]:
    """Freeze exact file bytes and every file/directory mode, rejecting links."""
    files: dict[str, str] = {}
    modes: dict[str, int] = {}
    if not root.exists():
        return files, modes
    for directory, children, names in os.walk(root, followlinks=False):
        current = Path(directory)
        modes[str(current)] = current.lstat().st_mode
        for name in children:
            require(not (current / name).is_symlink(), '/outputs', 'directory link unsupported')
        for name in names:
            path = current / name
            mode = path.lstat().st_mode
            require(stat.S_ISREG(mode), str(path), 'output is not a regular file')
            files[str(path)] = file_hash(path)
            modes[str(path)] = mode
    return files, modes


def anchored(root: Path) -> dict[str, Json]:
    files, modes = snapshot(root)
    return {'inventory': {k:v for k,v in files.items()}, 'modes': {k:v for k,v in modes.items()}}


def recheck(root: Path, anchor: Node) -> None:
    files, modes = snapshot(root)
    require(files == anchor.field('inventory').mapping() and modes == anchor.field('modes').mapping(),
            '/outputs', 'returned output bytes, modes or namespace changed')


def publish_terminal(root: Path, terminal: Path, payload: dict[str, Json], original: Node) -> None:
    """Publish intended bytes, then recheck the preexisting namespace without recapture."""
    atomic_json(terminal, payload, mode=0o600)
    require(document(terminal.read_text()).mapping() == payload, '/terminal', 'intended terminal bytes changed')
    files, modes = snapshot(root)
    require(files.pop(str(terminal), None) == file_hash(terminal), '/terminal', 'missing publication')
    require(modes.pop(str(terminal), None) == stat.S_IFREG | 0o600, '/terminal', 'publication mode changed')
    require(files == original.field('inventory').mapping() and modes == original.field('modes').mapping(),
            '/terminal', 'publication changed original evidence or namespace')


def check_auxiliaries(session: Node) -> None:
    """Validate original source-owned supplemental output anchors when already returned."""
    root = Path(session.field('output').text())
    for path in sorted((root/'auxiliary-index').glob('*.json')):
        require(path.name == 'smoke.json', '/auxiliary', 'unknown supplemental producer')
        node = document(path.read_text())
        node.field('root').equals(str(root/'auxiliaries/smoke'))
        recheck(root/'auxiliaries/smoke', node.field('anchor'))


def entries(session: Node, arm: str) -> tuple[Node, ...]:
    """Read original launch ordinal records; never sort step IDs into a passing order."""
    root = Path(session.field('output').text())/'journal'/arm
    result: list[Node] = []
    expected = ORDER if arm == 'candidate' else REFERENCE
    previous: str | None = None
    for index, path in enumerate(sorted(root.glob('*.json'))):
        require(index < len(expected) and path.name == f'{index:02d}.json', '/journal', 'foreign ordinal')
        node = document(path.read_text())
        node.field('ordinal').equals(index)
        node.field('step').equals(expected[index])
        node.field('arm').equals(arm)
        node.field('previous_sha256').equals(previous)
        terminal = Path(node.field('terminal').text())
        require(file_hash(terminal) == node.field('terminal_sha256').text(), '/journal', 'terminal changed')
        recheck(terminal.parent, node.field('anchor'))
        stage = document(terminal.read_text())
        for name, raw in stage.field('producer_anchors').mapping().items():
            bound = Node(raw)
            producer = Path(name)
            match bound.field('kind').text():
                case 'ABSENT': require(not producer.exists(), name, 'absent producer appeared')
                case 'DIRECTORY': recheck(producer, bound)
                case 'FILE':
                    require(file_hash(producer) == bound.field('sha256').text()
                            and producer.lstat().st_mode == bound.field('mode').number(),
                            name, 'original producer file changed')
                case _: require(False, name, 'unknown producer anchor')
        result.append(stage)
        previous = file_hash(path)
    return tuple(result)


def begin(session: Node, arm: str, step: int) -> tuple[int, str | None]:
    prior = entries(session, arm)
    order = ORDER if arm == 'candidate' else REFERENCE
    ordinal = len(prior)
    require(ordinal < len(order) and step == order[ordinal], '/journal', 'out-of-order actual step launch')
    root = Path(session.field('output').text())/'journal'/arm
    root.mkdir(parents=True, exist_ok=True)
    previous = None if ordinal == 0 else file_hash(root/f'{ordinal-1:02d}.json')
    return ordinal, previous


def publish(session: Node, arm: str, step: int, terminal: Path,
            ordinal: int, previous: str | None) -> None:
    require(begin(session, arm, step) == (ordinal, previous), '/journal', 'concurrent stage publication')
    atomic_json(Path(session.field('output').text())/'journal'/arm/f'{ordinal:02d}.json', {
        'ordinal': ordinal, 'step': step, 'arm': arm, 'previous_sha256': previous,
        'terminal': str(terminal), 'terminal_sha256': file_hash(terminal), 'anchor': anchored(terminal.parent)})
