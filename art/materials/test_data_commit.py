# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pydantic"]
# ///
# How to run: python3 -m pytest art/materials/test_data_commit.py -q.
"""Real temporary Git histories exercise valid and contaminated proof commits."""
import subprocess
from pathlib import Path

import pytest

from art.materials.check_data_commit import prove_data_commit
from art.materials.schema import StylesDocument


def git(root: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(root), *args], check=True,
                          text=True, capture_output=True).stdout.strip()


@pytest.fixture
def history(tmp_path: Path) -> Path:
    folder = tmp_path / "art/materials"
    folder.mkdir(parents=True)
    source = Path(__file__).with_name("construction_styles.json").read_text()
    _ = (folder / "construction_styles.json").write_text(source)
    _ = git(tmp_path, "init", "--quiet")
    _ = git(tmp_path, "config", "user.email", "test@example.invalid")
    _ = git(tmp_path, "config", "user.name", "Material proof test")
    _ = git(tmp_path, "add", "art/materials/construction_styles.json")
    _ = git(tmp_path, "commit", "--quiet", "-m", "Freeze generic style data")
    document = StylesDocument.model_validate_json(source)
    base = document.styles[0]
    addition = base.model_copy(update={"id":"test_new_era","sim_blueprints":()})
    changed = document.model_copy(update={"styles":(*document.styles,addition)})
    _ = (folder / "construction_styles.json").write_text(changed.model_dump_json(indent=2))
    for path in (base.textures.face,base.textures.top_ew,base.textures.top_ns,base.textures.end,
                 base.textures.junction,base.textures.door_closed,base.textures.door_open):
        assert path is not None
        destination = tmp_path / path
        destination.parent.mkdir(parents=True,exist_ok=True)
        _ = destination.write_bytes(b"fixture blob, pixel validation is a separate test")
        _ = git(tmp_path,"add",path)
    _ = git(tmp_path, "add", "art/materials/construction_styles.json")
    return tmp_path


def test_real_data_only_commit_is_accepted(history: Path) -> None:
    # Given a parent without the added style and new texture blobs.
    _ = git(history,"commit","--quiet","-m","Add an era through data")
    # When checking the actual Git diff and committed objects.
    proof = prove_data_commit(history,"HEAD","test_new_era")
    # Then the evidence is tied to an actual SHA and all seven assets.
    assert proof.passed
    assert len(proof.commit) == 40
    assert len(proof.texture_paths) == 7
    assert not any(path.endswith('.py') for path in proof.changed_files)


def test_python_file_in_same_commit_is_rejected(history: Path) -> None:
    # Given an apparently data-only addition with one code file slipped in.
    _ = (history / "art/materials/special_case.py").write_text("SPECIAL_CASE = True\n")
    _ = git(history,"add","art/materials/special_case.py")
    _ = git(history,"commit","--quiet","-m","Contaminated era addition")
    # When checking that exact commit, then code contamination fails.
    with pytest.raises(ValueError,match="no .py"):
        _ = prove_data_commit(history,"HEAD","test_new_era")


def test_published_style_commit_touches_no_python() -> None:
    # Given the committed proof record produced after the isolated data commit.
    from pydantic import TypeAdapter

    from art.materials.check_data_commit import CommitProof
    root = Path(__file__).resolve().parents[2]
    evidence = root / "docs/v8/artpolish2/DATA_ONLY_PROOF.json"
    if not evidence.exists():
        pytest.skip("Data-only style commit has not been recorded yet; mandatory final gate")
    registered = TypeAdapter(CommitProof).validate_json(evidence.read_bytes())
    # When independently recomputing its Git diff and object membership.
    actual = prove_data_commit(root,registered.commit,registered.style_id)
    # Then the committed evidence and no-code invariant both hold.
    assert actual == registered
    assert not any(Path(path).suffix == ".py" for path in actual.changed_files)
