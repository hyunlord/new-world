# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: python3 -m art.materials.check_data_commit COMMIT STYLE_ID.
"""Verify an added construction style is a real data-only Git commit."""
from __future__ import annotations

import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from art.materials.schema import StylesDocument

STYLES_PATH: Final = "art/materials/construction_styles.json"


@dataclass(frozen=True, slots=True)
class CommitProof:
    commit: str
    parent: str
    style_id: str
    changed_files: tuple[str, ...]
    texture_paths: tuple[str, ...]
    passed: bool


def _git(root: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(root), *args], check=True,
                          text=True, capture_output=True).stdout.strip()


def prove_data_commit(root: Path, commit: str, style_id: str) -> CommitProof:
    """Check actual parent/commit content and every named texture object."""
    sha = _git(root, "rev-parse", "--verify", f"{commit}^{{commit}}")
    parent = _git(root, "rev-parse", f"{sha}^")
    files = tuple(_git(root, "diff-tree", "--no-commit-id", "--name-only", "-r", sha).splitlines())
    if not files or STYLES_PATH not in files or any(Path(path).suffix == ".py" for path in files):
        message = "Style-addition commit must change style JSON and no .py file"
        raise ValueError(message)
    if any(not path.startswith(("art/", "docs/v8/artpolish2/")) for path in files):
        message = "Style-addition commit crosses the authorized path boundary"
        raise ValueError(message)
    before = StylesDocument.model_validate_json(_git(root, "show", f"{parent}:{STYLES_PATH}"))
    after = StylesDocument.model_validate_json(_git(root, "show", f"{sha}:{STYLES_PATH}"))
    if any(style.id == style_id for style in before.styles):
        message = f"Style already existed before {sha}: {style_id}"
        raise ValueError(message)
    style = next((row for row in after.styles if row.id == style_id), None)
    if style is None:
        message = f"Style was not added by {sha}: {style_id}"
        raise ValueError(message)
    paths = style.textures
    textures = tuple(path for path in (paths.face, paths.top_ew, paths.top_ns, paths.end,
                     paths.junction, paths.door_closed, paths.door_open, paths.floor) if path is not None)
    for path in textures:
        _ = _git(root, "cat-file", "-e", f"{sha}:{path}")
    return CommitProof(sha, parent, style_id, files, textures, True)


def main() -> int:
    import json
    if len(sys.argv) != 3:
        print("Usage: python3 -m art.materials.check_data_commit COMMIT STYLE_ID", file=sys.stderr)
        return 2
    result = prove_data_commit(Path(__file__).resolve().parents[2], sys.argv[1], sys.argv[2])
    print(json.dumps(asdict(result), indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
