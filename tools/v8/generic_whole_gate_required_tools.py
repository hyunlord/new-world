"""Original Cargo gate tools must be executable, explicitly bound and PATH-stable."""
import os
import shutil
import stat
from pathlib import Path
from typing import Final

from generic_gate import require
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

REQUIRED: Final = ('rustdoc', 'rustfmt', 'cargo-fmt', 'cargo-clippy', 'clippy-driver')


def validate_search_path(search_path: str) -> None:
    """Reject cwd-dependent lookup before resolving any selected executable."""
    require(all(part and Path(part).is_absolute() for part in search_path.split(os.pathsep)),
            '/environment/PATH', 'PATH components must be nonempty absolute paths')


def required_path_tools(search_path: str, external: dict[str, str]) -> dict[str, Json]:
    """Resolve selected PATH without executing a tool or assuming an installation layout."""
    validate_search_path(search_path)
    selected: dict[str, Json] = {}
    for command in REQUIRED:
        found = shutil.which(command, path=search_path)
        require(found is not None, command, 'missing required PATH tool')
        assert found is not None
        canonical = Path(found).resolve(strict=True)
        require(stat.S_ISREG(canonical.stat().st_mode) and os.access(canonical, os.X_OK),
                command, 'required PATH tool is not an executable regular file')
        digest = external.get(str(canonical))
        require(digest is not None, command, 'required PATH tool lacks canonical external input hash')
        require(file_hash(canonical) == digest, command, 'required PATH tool bytes differ from selected hash')
        selected[command] = str(canonical)
    return selected


def check_required_path_tools(session: Node) -> None:
    """Re-resolve aliases so an unchanged target hash cannot hide a retargeted PATH entry."""
    external = {name: Node(value).text() for name, value in session.field('external_inputs').mapping().items()}
    actual = required_path_tools(session.field('environment').field('PATH').text(), external)
    session.field('required_path_tools').equals(actual)
