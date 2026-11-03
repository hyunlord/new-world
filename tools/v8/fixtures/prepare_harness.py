"""Exact p96 fixture-only substitutions; never original-source qualification."""
import hashlib
from pathlib import Path
from typing import Literal, TypedDict

PREIMAGES = {
    'generic_setup_bridge.py': '8bba2fb8a34d0958909ec7bd5bb7ddc42b380eea639f193c530b31e7b2b23a46',
    'generic_setup_clippy.py': '9f1325bd031f4e615c09f41ef7599077d80df077151c0229a4cad01556cebe2a',
    'mux2_gate_lock.py': '3464e91509cd64af380181c178c738462f12ae7a15a7a635aa9d69bdca3ffbfd',
}
TOOL_NAMES = ('bash', 'cargo', 'rustc', 'cargo-clippy', 'clippy-driver')
ORIGINAL_TOOLS = {
    'bash': 'd59c6f2312e551e9c46497329d435b6ec74fbc46e9155b84170f377127ed004e',
    'cargo': '2be8a4e73901a99470d15b793e5ee814005c3b9ebbd3815ccbf9cf0f292238e7',
    'rustc': '34a8f7034eeb844189a3c36ebb3fe36c2193fdb47b60028eb7f9392cb1f62484',
    'cargo-clippy': '64209799ae8899219f187e5ed6b5d917598c3a7a65b02391c355074b6ed00b56',
    'clippy-driver': '1825b12644084b879d4cc43c20a95f750929796ad7b30a444cc54e570cf4de28',
}


class Replacement(TypedDict):
    old: str
    new: str
    count: int


class FileChange(TypedDict):
    before: str
    after: str
    replacements: list[Replacement]


class Transformation(TypedDict):
    qualification: Literal['TRANSFORMED_SOURCE_FIXTURE_ONLY']
    files: dict[str, FileChange]


def digest(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def transform(root: Path, tools: dict[str, Path], lock: Path) -> Transformation:
    """Validate all three preimages before changing any copied fixture file."""
    if root != root.resolve() or lock != lock.resolve() or lock.parent not in {root, root.parent}:
        raise ValueError('fixture root and private sibling lock must be canonical')
    if set(tools) != set(TOOL_NAMES):
        raise ValueError('exact five tool identities required')
    hashes: dict[str, str] = {}
    for name, path in tools.items():
        if path != path.resolve() or not path.is_file():
            raise ValueError('fixture tool must be a canonical actual file')
        hashes[name] = digest(path.read_bytes())
    changes: dict[str, FileChange] = {}
    outputs: dict[Path, bytes] = {}
    for name, expected in PREIMAGES.items():
        path = root/'tools/v8'/name
        if path != path.resolve():
            raise ValueError('copied helper must not be a symlink')
        raw = path.read_bytes()
        if digest(raw) != expected:
            raise ValueError('p96 helper preimage differs: '+name)
        pairs: list[tuple[str, str]] = []
        if name == 'mux2_gate_lock.py':
            pairs.append(("LOCK: Final = Path('/Users/rexxa/work/mux2-data/.mux2-gpu.lock')",
                          'LOCK: Final = Path('+repr(str(lock))+')'))
        else:
            selected = TOOL_NAMES[:3] if name == 'generic_setup_bridge.py' else TOOL_NAMES[3:]
            for tool in selected:
                pairs.append((repr(ORIGINAL_TOOLS[tool]), repr(hashes[tool])))
        text = raw.decode(); replacements: list[Replacement] = []
        for old, new in pairs:
            if text.count(old) != 1:
                raise ValueError('approved literal does not occur exactly once')
            text = text.replace(old, new)
            replacements.append({'old': old, 'new': new, 'count': 1})
        output = text.encode(); outputs[path] = output
        changes[name] = {'before': expected, 'after': digest(output), 'replacements': replacements}
    # All validation completed. This touches only three already copied fixture files.
    for path, output in outputs.items():
        _ = path.write_bytes(output)
    return {'qualification': 'TRANSFORMED_SOURCE_FIXTURE_ONLY', 'files': changes}
